//! Sqlite database integration and setup.

#[allow(missing_docs, unused_imports)]
pub mod schema;

use crate::{settings, Receipt};
use anyhow::Result;
use byte_unit::{AdjustedByte, Byte, ByteUnit};
use diesel::{prelude::*, r2d2};
use dotenvy::dotenv;
use homestar_core::workflow::Pointer;
use std::{env, sync::Arc, time::Duration};
use tokio::fs;

/// A Sqlite connection [pool].
///
/// [pool]: r2d2::Pool
pub type Pool = r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>;
/// A [connection] from the Sqlite connection [pool].
///
/// [connection]: r2d2::PooledConnection
/// [pool]: r2d2::Pool
pub type Connection = r2d2::PooledConnection<r2d2::ConnectionManager<diesel::SqliteConnection>>;

/// The database object, which wraps an inner [Arc] to the connection pool.
#[derive(Debug)]
pub struct Db(Arc<Pool>);

impl Clone for Db {
    fn clone(&self) -> Self {
        Db(Arc::clone(&self.0))
    }
}

impl Db {
    fn url() -> String {
        dotenv().ok();
        env::var("DATABASE_URL").expect("DATABASE_URL must be set")
    }

    /// Get size of SQlite file in megabytes (via async call).
    pub async fn size() -> Result<AdjustedByte> {
        let len = fs::metadata(Db::url()).await?.len();
        let byte = Byte::from_bytes(len);
        let byte_unit = byte.get_adjusted_unit(ByteUnit::MB);
        Ok(byte_unit)
    }
}

/// Database trait for working with different Sqlite [pool] and [connection]
/// configurations.
///
/// [pool]: Pool
/// [connection]: Connection
pub trait Database {
    /// Establish a pooled connection to Sqlite database.
    fn setup_connection_pool(settings: &settings::Node) -> Self;
    /// Get a [pooled connection] for the database.
    ///
    /// [pooled connection]: Connection
    fn conn(&self) -> Result<Connection>;
    /// Store receipt given a [Connection] to the DB [Pool].
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

    /// Store receipts given a [Connection] to the DB [Pool].
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
    fn find_instructions(pointers: Vec<Pointer>, conn: &mut Connection) -> Result<Vec<Receipt>> {
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
}

impl Database for Db {
    fn setup_connection_pool(settings: &settings::Node) -> Self {
        let manager = r2d2::ConnectionManager::<SqliteConnection>::new(Db::url());

        let pool = r2d2::Pool::builder()
            // Max number of conns.
            .max_size(settings.db.max_pool_size)
            // Never maintain idle connections
            .min_idle(Some(0))
            // Close connections after 30 seconds of idle time
            .idle_timeout(Some(Duration::from_secs(30)))
            .build(manager)
            .expect("DATABASE_URL must be set to an SQLite DB file");
        Db(Arc::new(pool))
    }

    fn conn(&self) -> Result<Connection> {
        let conn = self.0.get()?;
        Ok(conn)
    }
}
