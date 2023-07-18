//! Sqlite database integration and setup.

#[allow(missing_docs, unused_imports)]
pub mod schema;

use crate::{
    settings,
    workflow::{self, StoredReceipt},
    Receipt,
};
use anyhow::Result;
use byte_unit::{AdjustedByte, Byte, ByteUnit};
use diesel::{
    prelude::*,
    r2d2::{self, CustomizeConnection, ManageConnection},
    BelongingToDsl, Connection as SingleConnection, RunQueryDsl, SqliteConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use homestar_core::workflow::Pointer;
use libipld::Cid;
use std::{env, sync::Arc, time::Duration};
use tokio::fs;
use tracing::info;

const ENV: &str = "DATABASE_URL";
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");
const PRAGMAS: &str = "
PRAGMA journal_mode = WAL;          -- better write-concurrency
PRAGMA synchronous = NORMAL;        -- fsync only in critical moments
PRAGMA wal_autocheckpoint = 1000;   -- write WAL changes back every 1000 pages, for an in average 1MB WAL file. May affect readers if number is increased
PRAGMA busy_timeout = 1000;         -- sleep if the database is busy
PRAGMA foreign_keys = ON;           -- enforce foreign keys
";

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
pub struct Db(Arc<Pool>);

impl Clone for Db {
    fn clone(&self) -> Self {
        Db(Arc::clone(&self.0))
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

    /// Get database url.
    ///
    /// Contains a minimal side-effect to set the env if not already set.
    pub fn set_url(database_url: Option<String>) -> Option<String> {
        database_url.map_or_else(
            || dotenv().ok().and_then(|_| env::var(ENV).ok()),
            |url| {
                env::set_var(ENV, &url);
                Some(url)
            },
        )
    }
}

/// Database trait for working with different Sqlite connection pool and
/// connection configurations.
pub trait Database: Send + Sync + Clone {
    /// Test a Sqlite connection to the database and run pending migrations.
    fn setup(url: &str) -> Result<SqliteConnection> {
        info!("Using database at {:?}", url);
        let mut connection = SqliteConnection::establish(url)?;
        let _ = connection.run_pending_migrations(MIGRATIONS);

        Ok(connection)
    }

    /// Establish a pooled connection to Sqlite database.
    fn setup_connection_pool(settings: &settings::Node) -> Result<Self>
    where
        Self: Sized;
    /// Get a pooled connection for the database.
    fn conn(&self) -> Result<Connection>;
    /// Store receipt given a connection to the database pool.
    ///
    /// On conflicts, do nothing.
    fn store_receipt(receipt: Receipt, conn: &mut Connection) -> Result<Receipt> {
        diesel::insert_into(schema::receipts::table)
            .values(&receipt)
            .on_conflict(schema::receipts::cid)
            .do_nothing()
            .get_result(conn)
            .map_err(Into::into)
    }

    /// Store receipts given a connection to the Database pool.
    fn store_receipts(receipts: Vec<Receipt>, conn: &mut Connection) -> Result<usize> {
        diesel::insert_into(schema::receipts::table)
            .values(&receipts)
            .execute(conn)
            .map_err(Into::into)
    }

    /// Find receipt for a given [Instruction] [Pointer], which is indexed.
    ///
    /// This *should* always return one receipt, but sometimes it's nicer to
    /// work across vecs/arrays.
    ///
    /// [Instruction]: homestar_core::workflow::Instruction
    fn find_instructions(pointers: &Vec<Pointer>, conn: &mut Connection) -> Result<Vec<Receipt>> {
        let found_receipts = schema::receipts::dsl::receipts
            .filter(schema::receipts::instruction.eq_any(pointers))
            .load(conn)?;
        Ok(found_receipts)
    }

    /// Find receipt for a given [Instruction] [Pointer], which is indexed.
    ///
    /// [Instruction]: homestar_core::workflow::Instruction
    fn find_instruction(pointer: Pointer, conn: &mut Connection) -> Result<Receipt> {
        let found_receipt = schema::receipts::dsl::receipts
            .filter(schema::receipts::instruction.eq(pointer))
            .first(conn)?;
        Ok(found_receipt)
    }

    /// Store localized workflow cid and information, e.g. number of tasks.
    fn store_workflow(workflow: workflow::Stored, conn: &mut Connection) -> Result<usize> {
        diesel::insert_into(schema::workflows::table)
            .values(&workflow)
            .on_conflict(schema::workflows::cid)
            .do_nothing()
            .execute(conn)
            .map_err(Into::into)
    }

    /// Store workflow [Cid] and [Receipt] [Cid] in the database for inner join.
    fn store_workflow_receipt(
        workflow_cid: Cid,
        receipt_cid: Cid,
        conn: &mut Connection,
    ) -> Result<usize> {
        let value = StoredReceipt::new(Pointer::new(workflow_cid), Pointer::new(receipt_cid));
        diesel::insert_into(schema::workflows_receipts::table)
            .values(&value)
            .on_conflict((
                schema::workflows_receipts::workflow_cid,
                schema::workflows_receipts::receipt_cid,
            ))
            .do_nothing()
            .execute(conn)
            .map_err(Into::into)
    }

    /// Store series of receipts for a workflow [Cid] in the
    /// [schema::workflows_receipts] table.
    ///
    /// NOTE: We cannot do batch inserts with `on_conflict`, so we add
    /// each one 1-by-1:
    /// <https://github.com/diesel-rs/diesel/issues/3114>
    fn store_workflow_receipts(
        workflow_cid: Cid,
        receipts: &[Cid],
        conn: &mut Connection,
    ) -> Result<usize> {
        receipts.iter().try_fold(0, |acc, receipt| {
            let res = Self::store_workflow_receipt(workflow_cid, *receipt, conn)?;
            Ok::<_, anyhow::Error>(acc + res)
        })
    }

    /// Select workflow given a [Cid] to the workflow.
    fn select_workflow(cid: Cid, conn: &mut Connection) -> Result<workflow::Stored> {
        let wf = schema::workflows::dsl::workflows
            .filter(schema::workflows::cid.eq(Pointer::new(cid)))
            .select(workflow::Stored::as_select())
            .get_result(conn)?;
        Ok(wf)
    }

    /// Return workflow information with number of receipts emitted.
    fn get_workflow_info(workflow_cid: Cid, conn: &mut Connection) -> Result<workflow::Info> {
        let wf = Self::select_workflow(workflow_cid, conn)?;
        let associated_receipts = workflow::StoredReceipt::belonging_to(&wf)
            .select(schema::workflows_receipts::receipt_cid)
            .load(conn)?;

        let cids = associated_receipts
            .into_iter()
            .map(|pointer: Pointer| pointer.cid())
            .collect();

        Ok(workflow::Info::new(workflow_cid, cids, wf.num_tasks as u32))
    }
}

impl Database for Db {
    fn setup_connection_pool(settings: &settings::Node) -> Result<Self> {
        let database_url = env::var(ENV)?;
        Self::setup(&database_url)?;
        let manager = r2d2::ConnectionManager::<SqliteConnection>::new(database_url);

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
        Ok(Db(Arc::new(pool)))
    }

    fn conn(&self) -> Result<Connection> {
        let conn = self.0.get()?;
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
    use crate::{settings::Settings, test_utils};

    #[tokio::test]
    async fn check_pragmas_memory_db() {
        let db = test_utils::db::MemoryDb::setup_connection_pool(Settings::load().unwrap().node())
            .unwrap();
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
