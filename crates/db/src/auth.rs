use pi_kiosk_core::{Error, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub username: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

const SESSION_DURATION_HOURS: i64 = 24;

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| Error::InvalidInput(e.to_string()))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

pub fn create_user(conn: &Connection, username: &str, password: &str) -> Result<User> {
    let hash = hash_password(password)?;
    conn.execute(
        "INSERT INTO users (username, password_hash, role) VALUES (?1, ?2, 'admin')",
        rusqlite::params![username, hash],
    )
    .map_err(|e| Error::Database(e.to_string()))?;

    let id = conn.last_insert_rowid();
    Ok(User {
        id,
        username: username.to_string(),
        role: "admin".to_string(),
    })
}

pub fn get_user_by_username(conn: &Connection, username: &str) -> Result<Option<User>> {
    let result = conn
        .query_row(
            "SELECT id, username, role FROM users WHERE username = ?1",
            rusqlite::params![username],
            |r| {
                Ok(User {
                    id: r.get(0)?,
                    username: r.get(1)?,
                    role: r.get(2)?,
                })
            },
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })
        .map_err(|e| Error::Database(e.to_string()))?;

    Ok(result)
}

pub fn get_password_hash(conn: &Connection, username: &str) -> Result<Option<String>> {
    let result = conn
        .query_row(
            "SELECT password_hash FROM users WHERE username = ?1",
            rusqlite::params![username],
            |r| r.get::<_, String>(0),
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })
        .map_err(|e| Error::Database(e.to_string()))?;

    Ok(result)
}

pub fn authenticate(conn: &Connection, username: &str, password: &str) -> Result<Option<User>> {
    let hash = match get_password_hash(conn, username)? {
        Some(h) => h,
        None => return Ok(None),
    };
    if !verify_password(password, &hash) {
        return Ok(None);
    }
    get_user_by_username(conn, username)
}

pub fn create_session(conn: &Connection, username: &str) -> Result<Session> {
    let token = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();
    let expires = now + chrono::Duration::hours(SESSION_DURATION_HOURS);

    conn.execute(
        "INSERT INTO auth_sessions (token, username, expires_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![token, username, expires.to_rfc3339()],
    )
    .map_err(|e| Error::Database(e.to_string()))?;

    Ok(Session {
        token,
        username: username.to_string(),
        expires_at: expires,
    })
}

pub fn validate_session(conn: &Connection, token: &str) -> Result<Option<Session>> {
    let result = conn
        .query_row(
            "SELECT token, username, expires_at FROM auth_sessions WHERE token = ?1 AND expires_at > datetime('now')",
            rusqlite::params![token],
            |r| {
                let expires_str: String = r.get(2)?;
                let expires = chrono::DateTime::parse_from_rfc3339(&expires_str)
                    .map(|d| d.with_timezone(&chrono::Utc))
                    .unwrap_or(chrono::Utc::now());
                Ok(Session {
                    token: r.get(0)?,
                    username: r.get(1)?,
                    expires_at: expires,
                })
            },
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })
        .map_err(|e| Error::Database(e.to_string()))?;

    Ok(result)
}

pub fn delete_session(conn: &Connection, token: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM auth_sessions WHERE token = ?1",
        rusqlite::params![token],
    )
    .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}

pub fn cleanup_expired_sessions(conn: &Connection) -> Result<()> {
    conn.execute(
        "DELETE FROM auth_sessions WHERE expires_at <= datetime('now')",
        [],
    )
    .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}

pub fn user_count(conn: &Connection) -> Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))
        .map_err(|e| Error::Database(e.to_string()))
}
