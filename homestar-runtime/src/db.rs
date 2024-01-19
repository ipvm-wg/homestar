//! (Default) sqlite database integration and setup.

use crate::{
    settings,
    workflow::{self, StoredReceipt},
    Receipt,
};
use anyhow::Result;
use byte_unit::{AdjustedByte, Byte, ByteUnit};
use diesel::{
    dsl::now,
    prelude::*,
    r2d2::{self, CustomizeConnection, ManageConnection},
    BelongingToDsl, Connection as SingleConnection, RunQueryDsl, SqliteConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use homestar_invocation::Pointer;
use libipld::Cid;
use std::{env, sync::Arc, time::Duration};
use tokio::fs;
use tracing::info;

#[allow(missing_docs, unused_imports)]
pub mod schema;
pub(crate) mod utils;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");
const PRAGMAS: &str = "
PRAGMA journal_mode = WAL;          -- better write-concurrency
PRAGMA synchronous = NORMAL;        -- fsync only in critical moments
PRAGMA wal_autocheckpoint = 1000;   -- write WAL changes back every 1000 pages, for an in average 1MB WAL file. May affect readers if number is increased
PRAGMA busy_timeout = 1000;         -- sleep if the database is busy
PRAGMA foreign_keys = ON;           -- enforce foreign keys
";

/// Database environment variable.
pub(crate) const ENV: &str = "DATABASE_URL";

/// A Sqlite connection [pool].
///
/// [pool]: r2d2::Pool
pub(crate) type Pool = r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>;
/// A [connection] from the Sqlite connection [pool].
///
/// [connection]: r2d2::PooledConnection
/// [pool]: r2d2::Pool
pub(crate) type Connection =
    r2d2::PooledConnection<r2d2::ConnectionManager<diesel::SqliteConnection>>;

/// The database object, which wraps an inner [Arc] to the connection pool.
#[derive(Debug)]
pub struct Db {
    /// The [Arc]'ed connection pool.
    pub(crate) pool: Arc<Pool>,
    /// The database URL.
    pub(crate) url: String,
}

impl Clone for Db {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
            url: self.url.clone(),
        }
    }
}

impl Db {
    /// Get size of SQlite file in megabytes (via async call).
    pub async fn size() -> Result<AdjustedByte> {
        let url = env::var(ENV)?;
        let len = fs::metadata(url).await?.len();
        let byte = Byte::from_bytes(len);
        let byte_unit = byte.get_adjusted_unit(ByteUnit::MB);
        Ok(byte_unit)
    }
}

/// Database trait for working with different Sqlite connection pool and
/// connection configurations.
pub trait Database: Send + Sync + Clone {
    /// Establish a pooled connection to Sqlite database.
    fn setup_connection_pool(
        settings: &settings::Node,
        database_url: Option<String>,
    ) -> Result<Self>
    where
        Self: Sized;
    /// Get a pooled connection for the database.
    fn conn(&self) -> Result<Connection>;

    /// Set database url.
    ///
    /// Contains a minimal side-effect to set the env if not already set.
    fn set_url(database_url: Option<String>) -> Option<String> {
        database_url.map_or_else(
            || dotenv().ok().and_then(|_| env::var(ENV).ok()),
            |url| {
                env::set_var(ENV, &url);
                Some(url)
            },
        )
    }

    /// Get database url.
    fn url() -> Result<String> {
        Ok(env::var(ENV)?)
    }

    /// Test a Sqlite connection to the database and run pending migrations.
    fn setup(url: &str) -> Result<SqliteConnection> {
        info!(
            subject = "database",
            category = "homestar.init",
            "setting up database at {}, running migrations if needed",
            url
        );
        let mut connection = SqliteConnection::establish(url)?;
        let _ = connection.run_pending_migrations(MIGRATIONS);

        Ok(connection)
    }

    /// Check if the database is up.
    fn health_check(conn: &mut Connection) -> Result<(), diesel::result::Error> {
        diesel::sql_query("SELECT 1").execute(conn)?;
        Ok(())
    }

    /// Commit a receipt to the database, updating two tables
    /// within a transaction.
    fn commit_receipt(
        workflow_cid: Cid,
        receipt: Receipt,
        conn: &mut Connection,
    ) -> Result<Receipt, diesel::result::Error> {
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            if let Some(returned) = Self::store_receipt(receipt.clone(), conn)? {
                Self::store_workflow_receipt(workflow_cid, returned.cid(), conn)?;
                Ok(returned)
            } else {
                Self::store_workflow_receipt(workflow_cid, receipt.cid(), conn)?;
                Ok(receipt)
            }
        })
    }

    /// Store receipt given a connection to the database pool.
    ///
    /// On conflicts, do nothing.
    fn store_receipt(
        receipt: Receipt,
        conn: &mut Connection,
    ) -> Result<Option<Receipt>, diesel::result::Error> {
        diesel::insert_into(schema::receipts::table)
            .values(&receipt)
            .on_conflict(schema::receipts::cid)
            .do_nothing()
            .get_result(conn)
            .optional()
    }

    /// Store receipts given a connection to the Database pool.
    fn store_receipts(
        receipts: Vec<Receipt>,
        conn: &mut Connection,
    ) -> Result<usize, diesel::result::Error> {
        receipts.iter().try_fold(0, |acc, receipt| {
            if let Some(res) = diesel::insert_into(schema::receipts::table)
                .values(receipt)
                .on_conflict(schema::receipts::cid)
                .do_nothing()
                .execute(conn)
                .optional()?
            {
                Ok::<_, diesel::result::Error>(acc + res)
            } else {
                Ok(acc)
            }
        })
    }

    /// Find receipts given a set of [Instruction] [Pointer]s, which is indexed.
    ///
    /// [Instruction]: homestar_invocation::task::Instruction
    fn find_instruction_pointers(
        pointers: &Vec<Pointer>,
        conn: &mut Connection,
    ) -> Result<Vec<Receipt>, diesel::result::Error> {
        schema::receipts::dsl::receipts
            .filter(schema::receipts::instruction.eq_any(pointers))
            .load(conn)
    }

    /// Find receipt for a given [Instruction] Cid, which is indexed.
    ///
    /// [Instruction]: homestar_invocation::task::Instruction
    fn find_instruction_by_cid(
        cid: Cid,
        conn: &mut Connection,
    ) -> Result<Receipt, diesel::result::Error> {
        schema::receipts::dsl::receipts
            .filter(schema::receipts::instruction.eq(Pointer::new(cid)))
            .first(conn)
    }

    /// Find a receipt for a given Cid.
    fn find_receipt_by_cid(
        cid: Cid,
        conn: &mut Connection,
    ) -> Result<Receipt, diesel::result::Error> {
        schema::receipts::dsl::receipts
            .filter(schema::receipts::cid.eq(Pointer::new(cid)))
            .select(Receipt::as_select())
            .get_result(conn)
    }

    /// Find receipts given a batch of [Receipt] [Pointer]s.
    fn find_receipt_pointers(
        pointers: &Vec<Pointer>,
        conn: &mut Connection,
    ) -> Result<Vec<Receipt>, diesel::result::Error> {
        schema::receipts::dsl::receipts
            .filter(schema::receipts::cid.eq_any(pointers))
            .load(conn)
    }

    /// Store localized workflow cid and information, e.g. number of tasks.
    ///
    /// On conflicts, do nothing.
    /// Otherwise, return the stored workflow.
    fn store_workflow(
        workflow: workflow::Stored,
        conn: &mut Connection,
    ) -> Result<workflow::Stored, diesel::result::Error> {
        if let Some(stored) = diesel::insert_into(schema::workflows::table)
            .values(&workflow)
            .on_conflict(schema::workflows::cid)
            .do_nothing()
            .get_result(conn)
            .optional()?
        {
            Ok(stored)
        } else {
            Ok(workflow)
        }
    }

    /// Store workflow Cid and [Receipt] Cid in the database for inner join.
    fn store_workflow_receipt(
        workflow_cid: Cid,
        receipt_cid: Cid,
        conn: &mut Connection,
    ) -> Result<Option<usize>, diesel::result::Error> {
        let value = StoredReceipt::new(Pointer::new(workflow_cid), Pointer::new(receipt_cid));
        diesel::insert_into(schema::workflows_receipts::table)
            .values(&value)
            .on_conflict((
                schema::workflows_receipts::workflow_cid,
                schema::workflows_receipts::receipt_cid,
            ))
            .do_nothing()
            .execute(conn)
            .optional()
    }

    /// Store series of receipts for a workflow Cid in the
    /// [schema::workflows_receipts] table.
    ///
    /// NOTE: We cannot do batch inserts with `on_conflict`, so we add
    /// each one 1-by-1:
    /// <https://github.com/diesel-rs/diesel/issues/3114>
    fn store_workflow_receipts(
        workflow_cid: Cid,
        receipts: &[Cid],
        conn: &mut Connection,
    ) -> Result<usize, diesel::result::Error> {
        receipts.iter().try_fold(0, |acc, receipt| {
            if let Some(res) = Self::store_workflow_receipt(workflow_cid, *receipt, conn)? {
                Ok::<_, diesel::result::Error>(acc + res)
            } else {
                Ok(acc)
            }
        })
    }

    /// Select workflow given a Cid to the workflow.
    fn select_workflow(
        cid: Cid,
        conn: &mut Connection,
    ) -> Result<workflow::Stored, diesel::result::Error> {
        schema::workflows::dsl::workflows
            .filter(schema::workflows::cid.eq(Pointer::new(cid)))
            .select(workflow::Stored::as_select())
            .get_result(conn)
    }

    /// Return workflow information with number of receipts emitted.
    fn get_workflow_info(
        workflow_cid: Cid,
        conn: &mut Connection,
    ) -> Result<(Option<String>, workflow::Info), diesel::result::Error> {
        let workflow = Self::select_workflow(workflow_cid, conn)?;
        let associated_receipts = workflow::StoredReceipt::belonging_to(&workflow)
            .select(schema::workflows_receipts::receipt_cid)
            .load(conn)?;

        let cids = associated_receipts
            .into_iter()
            .map(|pointer: Pointer| pointer.cid())
            .collect();

        let name = workflow.name.clone();
        let info = workflow::Info::new(workflow, cids);

        Ok((name, info))
    }

    /// Update the local (view) name of a workflow.
    fn update_local_name(name: &str, conn: &mut Connection) -> Result<(), diesel::result::Error> {
        diesel::update(schema::workflows::dsl::workflows)
            .filter(schema::workflows::created_at.lt(now))
            .set(schema::workflows::name.eq(name))
            .execute(conn)?;

        Ok(())
    }
}

impl Database for Db {
    fn setup_connection_pool(
        settings: &settings::Node,
        database_url: Option<String>,
    ) -> Result<Self> {
        let database_url = Self::set_url(database_url).unwrap_or_else(|| {
            settings
                .db
                .url
                .as_ref()
                .map_or_else(|| "homestar.db".to_string(), |url| url.to_string())
        });

        Self::setup(&database_url)?;
        let manager = r2d2::ConnectionManager::<SqliteConnection>::new(database_url.clone());

        // setup PRAGMAs
        manager
            .connect()
            .and_then(|mut conn| ConnectionCustomizer.on_acquire(&mut conn))?;

        let pool = r2d2::Pool::builder()
            // Max number of conns.
            .max_size(settings.db.max_pool_size)
            // Never maintain idle connections
            .min_idle(Some(0))
            // Close connections after 30 seconds of idle time
            .idle_timeout(Some(Duration::from_secs(30)))
            .connection_customizer(Box::new(ConnectionCustomizer))
            .build(manager)
            .expect("DATABASE_URL must be set to an SQLite DB file");

        Ok(Db {
            pool: Arc::new(pool),
            url: database_url,
        })
    }

    fn conn(&self) -> Result<Connection> {
        let conn = self.pool.get()?;
        Ok(conn)
    }
}

/// Database connection options.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ConnectionCustomizer;

impl<C> CustomizeConnection<C, r2d2::Error> for ConnectionCustomizer
where
    C: diesel::Connection,
{
    fn on_acquire(&self, conn: &mut C) -> Result<(), r2d2::Error> {
        conn.batch_execute(PRAGMAS).map_err(r2d2::Error::QueryError)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::db::MemoryDb;

    #[homestar_runtime_proc_macro::db_async_test]
    fn check_pragmas_memory_db() {
        let settings = TestSettings::load();

        let db = MemoryDb::setup_connection_pool(settings.node(), None).unwrap();
        let mut conn = db.conn().unwrap();

        let journal_mode = diesel::dsl::sql::<diesel::sql_types::Text>("PRAGMA journal_mode")
            .load::<String>(&mut conn)
            .unwrap();

        assert_eq!(journal_mode, vec!["memory".to_string()]);

        let fk_mode = diesel::dsl::sql::<diesel::sql_types::Text>("PRAGMA foreign_keys")
            .load::<String>(&mut conn)
            .unwrap();

        assert_eq!(fk_mode, vec!["1".to_string()]);

        let busy_timeout = diesel::dsl::sql::<diesel::sql_types::Text>("PRAGMA busy_timeout")
            .load::<String>(&mut conn)
            .unwrap();

        assert_eq!(busy_timeout, vec!["1000".to_string()]);
    }
}
