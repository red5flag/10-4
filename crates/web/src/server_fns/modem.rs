use leptos::*;
use leptos_axum::extract;
use pi_kiosk_core::ModemStatus;
use pi_kiosk_modem::at::SmsMessage;
use std::sync::Arc;

use crate::state::AppState;

#[server(GetModemStatus, "/api")]
pub async fn get_modem_status() -> Result<ModemStatus, ServerFnError> {
    let senders = crate::state::get_live_senders()
        .ok_or_else(|| ServerFnError::<std::convert::Infallible>::ServerError("no live state".into()))?;
    let status = senders.modem.borrow().clone();
    Ok(status)
}

#[server(SendSms, "/api")]
pub async fn send_sms(number: String, text: String) -> Result<(), ServerFnError> {
    let _state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    let mut manager = pi_kiosk_modem::ModemManager::new();
    if !manager.detect() {
        return Err(ServerFnError::ServerError("no modem detected".into()));
    }

    manager.send_sms(&number, &text)
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    Ok(())
}

#[server(GetSmsInbox, "/api")]
pub async fn get_sms_inbox() -> Result<Vec<SmsMessage>, ServerFnError> {
    let _state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    let mut manager = pi_kiosk_modem::ModemManager::new();
    if !manager.detect() {
        return Ok(Vec::new());
    }

    match manager.read_sms() {
        Ok(msgs) => Ok(msgs),
        Err(e) => {
            tracing::warn!("failed to read SMS: {e}");
            Ok(Vec::new())
        }
    }
}

#[server(GetModemDiagnostics, "/api")]
pub async fn get_modem_diagnostics() -> Result<ModemDiagnostics, ServerFnError> {
    let _state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    let senders = crate::state::get_live_senders()
        .ok_or_else(|| ServerFnError::<std::convert::Infallible>::ServerError("no live state".into()))?;
    let status = senders.modem.borrow().clone();

    let mut diag = ModemDiagnostics {
        status: status.clone(),
        serial_ports: pi_kiosk_modem::at::detect_serial_ports(),
        modemmanager_available: std::process::Command::new("mmcli")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false),
        at_response: None,
    };

    if status.present {
        let mut manager = pi_kiosk_modem::ModemManager::new();
        if manager.detect() {
            if let Ok(resp) = manager.get_at_diagnostics() {
                diag.at_response = Some(resp);
            }
        }
    }

    Ok(diag)
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ModemDiagnostics {
    pub status: ModemStatus,
    pub serial_ports: Vec<String>,
    pub modemmanager_available: bool,
    pub at_response: Option<String>,
}
