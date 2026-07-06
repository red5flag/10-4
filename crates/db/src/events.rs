use pi_kiosk_core::{DetectionEvent, DetectionKind, Error, Result};
use rusqlite::Connection;

pub fn insert_event(conn: &Connection, event: &DetectionEvent) -> Result<()> {
    let kind_str = match event.kind {
        DetectionKind::Motion => "motion",
        DetectionKind::Person => "person",
        DetectionKind::Tamper => "tamper",
    };
    conn.execute(
        "INSERT INTO detection_events (id, kind, timestamp, confidence, thumbnail_path, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            event.id,
            kind_str,
            event.timestamp.to_rfc3339(),
            event.confidence,
            event.thumbnail_path,
            event.metadata.to_string(),
        ],
    )
    .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}

pub fn recent_events(conn: &Connection, limit: u32) -> Result<Vec<DetectionEvent>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, kind, timestamp, confidence, thumbnail_path, metadata
             FROM detection_events ORDER BY timestamp DESC LIMIT ?1",
        )
        .map_err(|e| Error::Database(e.to_string()))?;

    let rows = stmt
        .query_map(rusqlite::params![limit], |row| {
            let kind_str: String = row.get(1)?;
            let kind = match kind_str.as_str() {
                "motion" => DetectionKind::Motion,
                "person" => DetectionKind::Person,
                "tamper" => DetectionKind::Tamper,
                _ => DetectionKind::Motion,
            };
            let ts: String = row.get(2)?;
            let meta_str: String = row.get(5)?;
            Ok(DetectionEvent {
                id: row.get(0)?,
                kind,
                timestamp: chrono::DateTime::parse_from_rfc3339(&ts)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?
                    .with_timezone(&chrono::Utc),
                confidence: row.get(3)?,
                thumbnail_path: row.get(4)?,
                metadata: serde_json::from_str(&meta_str).unwrap_or(serde_json::Value::Null),
            })
        })
        .map_err(|e| Error::Database(e.to_string()))?;

    let mut events = Vec::new();
    for row in rows {
        events.push(row.map_err(|e| Error::Database(e.to_string()))?);
    }
    Ok(events)
}

pub fn delete_event(conn: &Connection, event_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM detection_events WHERE id = ?1",
        rusqlite::params![event_id],
    )
    .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}

pub fn clear_all_events(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM detection_events", [])
        .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}
