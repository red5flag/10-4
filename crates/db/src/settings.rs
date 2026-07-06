use pi_kiosk_core::{Error, Result};
use rusqlite::Connection;
use serde::{de::DeserializeOwned, Serialize};

pub fn get<T: DeserializeOwned>(conn: &Connection, key: &str) -> Result<Option<T>> {
    let row: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            rusqlite::params![key],
            |r| r.get(0),
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })
        .map_err(|e| Error::Database(e.to_string()))?;

    match row {
        Some(json) => {
            let val = serde_json::from_str(&json).map_err(|e| Error::Database(e.to_string()))?;
            Ok(Some(val))
        }
        None => Ok(None),
    }
}

pub fn set<T: Serialize>(conn: &Connection, key: &str, value: &T) -> Result<()> {
    let json = serde_json::to_string(value)?;
    conn.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
        rusqlite::params![key, json],
    )
    .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}

pub fn delete(conn: &Connection, key: &str) -> Result<()> {
    conn.execute("DELETE FROM settings WHERE key = ?1", rusqlite::params![key])
        .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}
