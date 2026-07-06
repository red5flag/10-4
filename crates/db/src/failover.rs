use pi_kiosk_core::{FailoverEvent, Result, WanSource};
use rusqlite::Connection;

pub fn insert_failover_event(
    conn: &Connection,
    from_source: Option<WanSource>,
    to_source: WanSource,
    reason: &str,
) -> Result<()> {
    let from_str = from_source.map(|s| s.to_string());
    conn.execute(
        "INSERT INTO failover_log (from_source, to_source, reason) VALUES (?1, ?2, ?3)",
        rusqlite::params![from_str, to_source.to_string(), reason],
    )
    .map_err(|e| pi_kiosk_core::Error::Database(e.to_string()))?;
    Ok(())
}

pub fn list_failover_events(conn: &Connection, limit: u32) -> Result<Vec<FailoverEvent>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, from_source, to_source, reason \
             FROM failover_log ORDER BY id DESC LIMIT ?1",
        )
        .map_err(|e| pi_kiosk_core::Error::Database(e.to_string()))?;

    let rows = stmt
        .query_map(rusqlite::params![limit], |row| {
            let id: i64 = row.get(0)?;
            let ts: String = row.get(1)?;
            let from: Option<String> = row.get(2)?;
            let to: String = row.get(3)?;
            let reason: String = row.get(4)?;

            let from_source = from.and_then(|s| parse_wan_source(&s));
            let to_source = parse_wan_source(&to).unwrap_or(WanSource::Ethernet);

            Ok(FailoverEvent {
                id,
                timestamp: chrono::DateTime::parse_from_rfc3339(&ts)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                from_source,
                to_source,
                reason,
            })
        })
        .map_err(|e| pi_kiosk_core::Error::Database(e.to_string()))?;

    let mut events = Vec::new();
    for row in rows {
        events.push(row.map_err(|e| pi_kiosk_core::Error::Database(e.to_string()))?);
    }
    Ok(events)
}

pub fn clear_failover_events(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM failover_log", [])
        .map_err(|e| pi_kiosk_core::Error::Database(e.to_string()))?;
    Ok(())
}

fn parse_wan_source(s: &str) -> Option<WanSource> {
    match s.trim() {
        "ethernet" => Some(WanSource::Ethernet),
        "wifi" => Some(WanSource::Wifi),
        "cellular" => Some(WanSource::Cellular),
        _ => None,
    }
}
