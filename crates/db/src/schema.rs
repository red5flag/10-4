use pi_kiosk_core::{Error, Result};
use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> Result<()> {
    let sql = include_str!("../../../migrations/001_init.sql");
    conn.execute_batch(sql)
        .map_err(|e| Error::Database(format!("migration failed: {e}")))?;
    tracing::info!("database migrations applied");
    Ok(())
}
