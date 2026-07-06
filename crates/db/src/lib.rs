pub mod auth;
pub mod events;
pub mod failover;
pub mod network;
pub mod notifications;
pub mod schema;
pub mod settings;
pub mod vpn;

use pi_kiosk_core::{Error, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Database {
    writer: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open(path: &str) -> Result<Self> {
        let dir = Path::new(path).parent();
        if let Some(d) = dir {
            std::fs::create_dir_all(d).map_err(|e| Error::Database(e.to_string()))?;
        }

        let conn = Connection::open(path).map_err(|e| Error::Database(e.to_string()))?;

        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| Error::Database(e.to_string()))?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(|e| Error::Database(e.to_string()))?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| Error::Database(e.to_string()))?;

        schema::run_migrations(&conn)?;

        Ok(Self {
            writer: Arc::new(Mutex::new(conn)),
        })
    }

    pub async fn writer(&self) -> tokio::sync::MutexGuard<'_, Connection> {
        self.writer.lock().await
    }

    pub async fn with_writer<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self.writer.lock().await;
        f(&conn)
    }
}
