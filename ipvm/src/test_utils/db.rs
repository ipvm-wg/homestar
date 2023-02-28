use diesel::{connection::SimpleConnection, Connection, SqliteConnection};

pub fn setup() -> anyhow::Result<SqliteConnection> {
    let mut conn = diesel::sqlite::SqliteConnection::establish(":memory:").unwrap();
    let source = diesel_migrations::FileBasedMigrations::find_migrations_directory()?;
    let _ = diesel_migrations::MigrationHarness::run_pending_migrations(&mut conn, source).unwrap();
    begin(&mut conn)?;
    Ok(conn)
}

pub fn begin(conn: &mut SqliteConnection) -> Result<(), diesel::result::Error> {
    conn.batch_execute("BEGIN;")
}
