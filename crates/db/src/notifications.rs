use pi_kiosk_core::{AlertAction, AlertLogEntry, AlertRule, Error, Result};
use rusqlite::Connection;
use uuid::Uuid;

pub fn insert_alert_rule(conn: &Connection, rule: &AlertRule) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO alert_rules \
         (id, event_type, action, action_target, cooldown_s, enabled, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            rule.id,
            rule.event_type,
            rule.action.to_string(),
            rule.action_target,
            rule.cooldown_s,
            rule.enabled as i32,
            rule.created_at.to_rfc3339(),
        ],
    )
    .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}

pub fn list_alert_rules(conn: &Connection) -> Result<Vec<AlertRule>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, event_type, action, action_target, cooldown_s, enabled, created_at \
             FROM alert_rules ORDER BY created_at DESC",
        )
        .map_err(|e| Error::Database(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let event_type: String = row.get(1)?;
            let action_str: String = row.get(2)?;
            let action_target: Option<String> = row.get(3)?;
            let cooldown_s: u32 = row.get(4)?;
            let enabled: i32 = row.get(5)?;
            let created_str: String = row.get(6)?;

            let action = match action_str.as_str() {
                "sms" => AlertAction::Sms,
                "webhook" => AlertAction::Webhook,
                _ => AlertAction::Ui,
            };

            let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            Ok(AlertRule {
                id,
                event_type,
                action,
                action_target,
                cooldown_s,
                enabled: enabled != 0,
                created_at,
            })
        })
        .map_err(|e| Error::Database(e.to_string()))?;

    let mut rules = Vec::new();
    for row in rows {
        rules.push(row.map_err(|e| Error::Database(e.to_string()))?);
    }
    Ok(rules)
}

pub fn delete_alert_rule(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM alert_rules WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}

pub fn set_alert_rule_enabled(conn: &Connection, id: &str, enabled: bool) -> Result<()> {
    conn.execute(
        "UPDATE alert_rules SET enabled = ?1 WHERE id = ?2",
        rusqlite::params![enabled as i32, id],
    )
    .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}

pub fn insert_alert_log(
    conn: &Connection,
    rule_id: Option<&str>,
    event_type: &str,
    action: &str,
    status: &str,
    message: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO alert_log (rule_id, event_type, action, status, message, timestamp) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            rule_id,
            event_type,
            action,
            status,
            message,
            chrono::Utc::now().to_rfc3339(),
        ],
    )
    .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}

pub fn list_alert_log(conn: &Connection, limit: u32) -> Result<Vec<AlertLogEntry>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, rule_id, event_type, action, status, message, timestamp \
             FROM alert_log ORDER BY timestamp DESC LIMIT ?1",
        )
        .map_err(|e| Error::Database(e.to_string()))?;

    let rows = stmt
        .query_map(rusqlite::params![limit], |row| {
            let id: i64 = row.get(0)?;
            let rule_id: Option<String> = row.get(1)?;
            let event_type: String = row.get(2)?;
            let action: String = row.get(3)?;
            let status: String = row.get(4)?;
            let message: Option<String> = row.get(5)?;
            let ts_str: String = row.get(6)?;

            let timestamp = chrono::DateTime::parse_from_rfc3339(&ts_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            Ok(AlertLogEntry {
                id,
                rule_id,
                event_type,
                action,
                status,
                message,
                timestamp,
            })
        })
        .map_err(|e| Error::Database(e.to_string()))?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|e| Error::Database(e.to_string()))?);
    }
    Ok(entries)
}

pub fn clear_alert_log(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM alert_log", [])
        .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}

pub fn last_alert_time(conn: &Connection, rule_id: &str) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
    let mut stmt = conn
        .prepare("SELECT timestamp FROM alert_log WHERE rule_id = ?1 ORDER BY timestamp DESC LIMIT 1")
        .map_err(|e| Error::Database(e.to_string()))?;

    let result: Option<String> = stmt
        .query_row(rusqlite::params![rule_id], |row| row.get(0))
        .ok();

    match result {
        Some(ts_str) => {
            let dt = chrono::DateTime::parse_from_rfc3339(&ts_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            Ok(Some(dt))
        }
        None => Ok(None),
    }
}

pub fn new_rule_id() -> String {
    Uuid::new_v4().to_string()
}
