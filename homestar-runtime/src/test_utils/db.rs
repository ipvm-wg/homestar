use crate::{
    db::{Connection, Database, Pool},
    settings,
};
use diesel::r2d2;
use std::sync::Arc;

/// Sqlite in-memory [Database] [Pool].
#[derive(Debug)]
pub struct MemoryDb(Arc<Pool>);

impl Database for MemoryDb {
    fn setup_connection_pool(_settings: &settings::Node) -> Self {
        let manager = r2d2::ConnectionManager::<diesel::SqliteConnection>::new(":memory:");
        let pool = r2d2::Pool::builder()
            .max_size(1)
            .build(manager)
            .expect("DATABASE_URL must be set to an SQLite DB file");

        let source = diesel_migrations::FileBasedMigrations::find_migrations_directory().unwrap();
        let _ = diesel_migrations::MigrationHarness::run_pending_migrations(
            &mut pool.get().unwrap(),
            source,
        )
        .unwrap();
        MemoryDb(Arc::new(pool))
    }

    fn conn(&self) -> anyhow::Result<Connection> {
        let conn = self.0.get()?;
        Ok(conn)
    }
}
