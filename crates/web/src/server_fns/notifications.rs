use leptos::*;
use leptos_axum::extract;
use pi_kiosk_core::{AlertAction, AlertLogEntry, AlertRule};
use std::convert::Infallible;
use std::sync::Arc;

use crate::state::AppState;

type FnError = ServerFnError<Infallible>;

#[server(GetAlertRules, "/api")]
pub async fn get_alert_rules() -> Result<Vec<AlertRule>, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    state
        .db
        .with_writer(|conn| pi_kiosk_db::notifications::list_alert_rules(conn))
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))
        .map_err(|e| e.into())
}

#[server(AddAlertRule, "/api")]
pub async fn add_alert_rule(
    event_type: String,
    action: AlertAction,
    action_target: Option<String>,
    cooldown_s: u32,
) -> Result<AlertRule, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    let rule = AlertRule {
        id: pi_kiosk_db::notifications::new_rule_id(),
        event_type,
        action,
        action_target,
        cooldown_s,
        enabled: true,
        created_at: chrono::Utc::now(),
    };

    state
        .db
        .with_writer(|conn| pi_kiosk_db::notifications::insert_alert_rule(conn, &rule))
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    Ok(rule)
}

#[server(DeleteAlertRule, "/api")]
pub async fn delete_alert_rule(id: String) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    state
        .db
        .with_writer(|conn| pi_kiosk_db::notifications::delete_alert_rule(conn, &id))
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))
        .map_err(|e| e.into())
}

#[server(ToggleAlertRule, "/api")]
pub async fn toggle_alert_rule(id: String, enabled: bool) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    state
        .db
        .with_writer(|conn| {
            pi_kiosk_db::notifications::set_alert_rule_enabled(conn, &id, enabled)
        })
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))
        .map_err(|e| e.into())
}

#[server(GetAlertLog, "/api")]
pub async fn get_alert_log(limit: u32) -> Result<Vec<AlertLogEntry>, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    state
        .db
        .with_writer(|conn| pi_kiosk_db::notifications::list_alert_log(conn, limit))
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))
        .map_err(|e| e.into())
}

#[server(ClearAlertLog, "/api")]
pub async fn clear_alert_log() -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    state
        .db
        .with_writer(|conn| pi_kiosk_db::notifications::clear_alert_log(conn))
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))
        .map_err(|e| e.into())
}

#[server(TestAlertRule, "/api")]
pub async fn test_alert_rule(id: String) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    let rules = state
        .db
        .with_writer(|conn| pi_kiosk_db::notifications::list_alert_rules(conn))
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    let rule = rules
        .into_iter()
        .find(|r| r.id == id)
        .ok_or_else(|| FnError::ServerError("rule not found".into()))?;

    let message = format!("Test notification for rule: {}", rule.event_type);

    dispatch_alert(&state, &rule, &message)
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    Ok(())
}

pub async fn dispatch_alert(
    state: &AppState,
    rule: &AlertRule,
    message: &str,
) -> anyhow::Result<()> {
    let status = match rule.action {
        AlertAction::Ui => {
            tracing::info!("UI notification: [{}] {}", rule.event_type, message);
            "sent"
        }
        AlertAction::Sms => {
            if let Some(number) = &rule.action_target {
                tracing::info!("SMS notification to {}: {}", number, message);
                "sent"
            } else {
                "failed"
            }
        }
        AlertAction::Webhook => {
            if let Some(url) = &rule.action_target {
                let client = reqwest::Client::new();
                let payload = serde_json::json!({
                    "event_type": rule.event_type,
                    "message": message,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                });

                match client
                    .post(url)
                    .json(&payload)
                    .timeout(std::time::Duration::from_secs(10))
                    .send()
                    .await
                {
                    Ok(resp) if resp.status().is_success() => {
                        tracing::info!("webhook sent to {}: {}", url, resp.status());
                        "sent"
                    }
                    Ok(resp) => {
                        tracing::warn!("webhook failed: {} status {}", url, resp.status());
                        "failed"
                    }
                    Err(e) => {
                        tracing::warn!("webhook error: {}", e);
                        "failed"
                    }
                }
            } else {
                "failed"
            }
        }
    };

    state
        .db
        .with_writer(|conn| {
            pi_kiosk_db::notifications::insert_alert_log(
                conn,
                Some(&rule.id),
                &rule.event_type,
                &rule.action.to_string(),
                status,
                Some(message),
            )
        })
        .await?;

    Ok(())
}
