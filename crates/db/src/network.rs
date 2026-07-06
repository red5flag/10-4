use pi_kiosk_core::{Error, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkProfile {
    pub id: String,
    pub name: String,
    pub mode: String,
    pub ssid: String,
    pub password: Option<String>,
    pub band: String,
    pub channel: Option<i32>,
    pub encryption: String,
    pub country_code: String,
    pub interface: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub id: String,
    pub name: String,
    pub proto: String,
    pub src_port: Option<String>,
    pub dest_ip: String,
    pub dest_port: String,
    pub enabled: bool,
    pub created_at: String,
}

pub fn insert_network_profile(conn: &Connection, profile: &NetworkProfile) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO network_profiles (id, name, mode, ssid, password, band, channel, encryption, country_code, interface, is_active, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            profile.id,
            profile.name,
            profile.mode,
            profile.ssid,
            profile.password,
            profile.band,
            profile.channel,
            profile.encryption,
            profile.country_code,
            profile.interface,
            profile.is_active as i32,
            profile.created_at,
        ],
    )
    .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}

pub fn list_network_profiles(conn: &Connection) -> Result<Vec<NetworkProfile>> {
    let mut stmt = conn
        .prepare("SELECT id, name, mode, ssid, password, band, channel, encryption, country_code, interface, is_active, created_at FROM network_profiles ORDER BY is_active DESC, name ASC")
        .map_err(|e| Error::Database(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(NetworkProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                mode: row.get(2)?,
                ssid: row.get(3)?,
                password: row.get(4)?,
                band: row.get(5)?,
                channel: row.get(6)?,
                encryption: row.get(7)?,
                country_code: row.get(8)?,
                interface: row.get(9)?,
                is_active: row.get::<_, i32>(10)? != 0,
                created_at: row.get(11)?,
            })
        })
        .map_err(|e| Error::Database(e.to_string()))?;

    let mut profiles = Vec::new();
    for row in rows {
        profiles.push(row.map_err(|e| Error::Database(e.to_string()))?);
    }
    Ok(profiles)
}

pub fn delete_network_profile(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM network_profiles WHERE id = ?1",
        params![id],
    )
    .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}

pub fn set_active_network_profile(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("UPDATE network_profiles SET is_active = 0", [])
        .map_err(|e| Error::Database(e.to_string()))?;
    conn.execute(
        "UPDATE network_profiles SET is_active = 1 WHERE id = ?1",
        params![id],
    )
    .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}

pub fn insert_firewall_rule(conn: &Connection, rule: &FirewallRule) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO firewall_rules (id, name, proto, src_port, dest_ip, dest_port, enabled, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            rule.id,
            rule.name,
            rule.proto,
            rule.src_port,
            rule.dest_ip,
            rule.dest_port,
            rule.enabled as i32,
            rule.created_at,
        ],
    )
    .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}

pub fn list_firewall_rules(conn: &Connection) -> Result<Vec<FirewallRule>> {
    let mut stmt = conn
        .prepare("SELECT id, name, proto, src_port, dest_ip, dest_port, enabled, created_at FROM firewall_rules ORDER BY created_at DESC")
        .map_err(|e| Error::Database(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(FirewallRule {
                id: row.get(0)?,
                name: row.get(1)?,
                proto: row.get(2)?,
                src_port: row.get(3)?,
                dest_ip: row.get(4)?,
                dest_port: row.get(5)?,
                enabled: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| Error::Database(e.to_string()))?;

    let mut rules = Vec::new();
    for row in rows {
        rules.push(row.map_err(|e| Error::Database(e.to_string()))?);
    }
    Ok(rules)
}

pub fn delete_firewall_rule(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM firewall_rules WHERE id = ?1",
        params![id],
    )
    .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}

pub fn toggle_firewall_rule(conn: &Connection, id: &str, enabled: bool) -> Result<()> {
    conn.execute(
        "UPDATE firewall_rules SET enabled = ?1 WHERE id = ?2",
        params![enabled as i32, id],
    )
    .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}
