use leptos::*;
use leptos_axum::extract;
use pi_kiosk_core::AppConfig;
use std::convert::Infallible;
use std::sync::Arc;

use crate::state::AppState;

type FnError = ServerFnError<Infallible>;

#[server(GetSettings, "/api")]
pub async fn get_settings() -> Result<AppConfig, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;
    let cfg = state.config.read().await.clone();
    Ok(cfg)
}

#[server(SaveSettings, "/api")]
pub async fn save_settings(config: AppConfig) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    {
        let mut cfg = state.config.write().await;
        *cfg = config;
    }

    state
        .save_settings()
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    Ok(())
}

#[server(ExportConfig, "/api")]
pub async fn export_config() -> Result<String, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    let cfg = state.config.read().await.clone();
    serde_json::to_string_pretty(&cfg)
        .map_err(|e| FnError::ServerError(format!("failed to serialize config: {e}")))
        .map_err(|e| e.into())
}

#[server(ImportConfig, "/api")]
pub async fn import_config(json: String) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    let config: AppConfig = serde_json::from_str(&json)
        .map_err(|e| FnError::ServerError(format!("invalid config JSON: {e}")))?;

    {
        let mut cfg = state.config.write().await;
        *cfg = config;
    }

    state
        .save_settings()
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    Ok(())
}

#[server(BackupDatabase, "/api")]
pub async fn backup_database() -> Result<String, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    let db_path = state.config.read().await.db_path.clone();
    let backup_path = format!(
        "/var/lib/pi-kiosk/backups/kiosk-backup-{}.db",
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    );

    std::fs::create_dir_all("/var/lib/pi-kiosk/backups")
        .map_err(|e| FnError::ServerError(format!("failed to create backup dir: {e}")))?;

    std::fs::copy(&db_path, &backup_path)
        .map_err(|e| FnError::ServerError(format!("failed to copy database: {e}")))?;

    tracing::info!("database backed up to {}", backup_path);
    Ok(backup_path)
}

#[server(ListBackups, "/api")]
pub async fn list_backups() -> Result<Vec<String>, ServerFnError> {
    let backup_dir = "/var/lib/pi-kiosk/backups";

    if !std::path::Path::new(backup_dir).exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    let entries = std::fs::read_dir(backup_dir)
        .map_err(|e| FnError::ServerError(format!("failed to read backup dir: {e}")))?;

    for entry in entries {
        if let Ok(entry) = entry {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".db") {
                    backups.push(name.to_string());
                }
            }
        }
    }

    backups.sort();
    backups.reverse();
    Ok(backups)
}

#[server(RestoreDatabase, "/api")]
pub async fn restore_database(backup_name: String) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    let db_path = state.config.read().await.db_path.clone();
    let backup_path = format!("/var/lib/pi-kiosk/backups/{}", backup_name);

    if !std::path::Path::new(&backup_path).exists() {
        return Err(FnError::ServerError("backup file not found".into()).into());
    }

    std::fs::copy(&backup_path, &db_path)
        .map_err(|e| FnError::ServerError(format!("failed to restore database: {e}")))?;

    tracing::info!("database restored from {}", backup_path);
    Ok(())
}

#[server(GenerateSelfSignedCert, "/api")]
pub async fn generate_self_signed_cert() -> Result<String, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    let cert_dir = "/etc/pi-kiosk/tls";
    std::fs::create_dir_all(cert_dir)
        .map_err(|e| FnError::ServerError(format!("failed to create cert dir: {e}")))?;

    let cert_path = format!("{}/pi-kiosk.crt", cert_dir);
    let key_path = format!("{}/pi-kiosk.key", cert_dir);
    let device_name = state.config.read().await.device_name.clone();

    let result = tokio::process::Command::new("openssl")
        .args([
            "req", "-x509", "-newkey", "rsa:2048",
            "-keyout", &key_path,
            "-out", &cert_path,
            "-days", "365",
            "-nodes",
            "-subj", &format!("/CN={}", device_name),
        ])
        .output()
        .await;

    match result {
        Ok(o) if o.status.success() => {
            tracing::info!("self-signed certificate generated at {}", cert_path);

            {
                let mut cfg = state.config.write().await;
                cfg.tls.enabled = true;
                cfg.tls.cert_path = cert_path.clone();
                cfg.tls.key_path = key_path;
            }
            state.save_settings().await
                .map_err(|e| FnError::ServerError(e.to_string()))?;

            Ok(cert_path)
        }
        Ok(o) => Err(FnError::ServerError(format!(
            "openssl failed: {}",
            String::from_utf8_lossy(&o.stderr)
        )).into()),
        Err(e) => Err(FnError::ServerError(format!("failed to run openssl: {e}")).into()),
    }
}
