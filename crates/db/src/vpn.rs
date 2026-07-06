use pi_kiosk_core::{Result, VpnConfig, VpnMode, VpnType};
use rusqlite::Connection;
use uuid::Uuid;

pub fn insert_vpn_config(conn: &Connection, config: &VpnConfig) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO vpn_configs \
         (id, name, type, mode, config_path, enabled, kill_switch, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            config.id,
            config.name,
            config.vpn_type.to_string(),
            config.mode.to_string(),
            config.config_path,
            config.enabled as i32,
            config.kill_switch as i32,
            config.created_at.to_rfc3339(),
        ],
    )
    .map_err(|e| pi_kiosk_core::Error::Database(e.to_string()))?;
    Ok(())
}

pub fn list_vpn_configs(conn: &Connection) -> Result<Vec<VpnConfig>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, type, mode, config_path, enabled, kill_switch, created_at \
             FROM vpn_configs ORDER BY created_at DESC",
        )
        .map_err(|e| pi_kiosk_core::Error::Database(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let type_str: String = row.get(2)?;
            let mode_str: String = row.get(3)?;
            let config_path: String = row.get(4)?;
            let enabled: i32 = row.get(5)?;
            let kill_switch: i32 = row.get(6)?;
            let created_str: String = row.get(7)?;

            let vpn_type = match type_str.as_str() {
                "openvpn" => VpnType::Openvpn,
                _ => VpnType::Wireguard,
            };

            let mode = match mode_str.as_str() {
                "server" => VpnMode::Server,
                _ => VpnMode::Client,
            };

            let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            Ok(VpnConfig {
                id,
                name,
                vpn_type,
                mode,
                config_path,
                enabled: enabled != 0,
                kill_switch: kill_switch != 0,
                created_at,
            })
        })
        .map_err(|e| pi_kiosk_core::Error::Database(e.to_string()))?;

    let mut configs = Vec::new();
    for row in rows {
        configs.push(row.map_err(|e| pi_kiosk_core::Error::Database(e.to_string()))?);
    }
    Ok(configs)
}

pub fn delete_vpn_config(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM vpn_configs WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| pi_kiosk_core::Error::Database(e.to_string()))?;
    Ok(())
}

pub fn set_vpn_enabled(conn: &Connection, id: &str, enabled: bool) -> Result<()> {
    conn.execute(
        "UPDATE vpn_configs SET enabled = ?1 WHERE id = ?2",
        rusqlite::params![enabled as i32, id],
    )
    .map_err(|e| pi_kiosk_core::Error::Database(e.to_string()))?;
    Ok(())
}

pub fn new_vpn_id() -> String {
    Uuid::new_v4().to_string()
}
