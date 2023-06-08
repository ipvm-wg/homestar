use crate::{
    db::{Connection, Database, Pool},
    settings,
};
use anyhow::Result;
use diesel::r2d2::{self, CustomizeConnection, ManageConnection};
use std::sync::Arc;

const PRAGMAS: &str = "
PRAGMA busy_timeout = 1000;         -- sleep if the database is busy
PRAGMA foreign_keys = ON;           -- enforce foreign keys
";

/// Database connection options.
#[derive(Debug, Clone)]
struct ConnectionCustomizer;

impl<C> CustomizeConnection<C, r2d2::Error> for ConnectionCustomizer
where
    C: diesel::Connection,
{
    fn on_acquire(&self, conn: &mut C) -> Result<(), r2d2::Error> {
        conn.batch_execute(PRAGMAS).map_err(r2d2::Error::QueryError)
    }
}

/// Sqlite in-memory [Database] [Pool].
#[derive(Debug)]
pub struct MemoryDb(Arc<Pool>);

impl Clone for MemoryDb {
    fn clone(&self) -> Self {
        MemoryDb(Arc::clone(&self.0))
    }
}

impl Database for MemoryDb {
    fn setup_connection_pool(_settings: &settings::Node) -> Result<Self> {
        let manager = r2d2::ConnectionManager::<diesel::SqliteConnection>::new(":memory:");

        // setup PRAGMAs
        manager
            .connect()
            .and_then(|mut conn| ConnectionCustomizer.on_acquire(&mut conn))?;

        let pool = r2d2::Pool::builder()
            .max_size(1)
            .connection_customizer(Box::new(ConnectionCustomizer))
            .build(manager)
            .expect("DATABASE_URL must be set to an SQLite DB file");

        let source = diesel_migrations::FileBasedMigrations::find_migrations_directory().unwrap();
        let _ = diesel_migrations::MigrationHarness::run_pending_migrations(
            &mut pool.get().unwrap(),
            source,
        )
        .unwrap();
        Ok(MemoryDb(Arc::new(pool)))
    }

    fn conn(&self) -> anyhow::Result<Connection> {
        let conn = self.0.get()?;
        Ok(conn)
    }
}
