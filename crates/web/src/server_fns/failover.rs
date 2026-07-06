use leptos::*;
use leptos_axum::extract;
use pi_kiosk_core::{FailoverConfig, FailoverEvent, WanSource};
use std::convert::Infallible;
use std::sync::Arc;

use crate::state::AppState;

type FnError = ServerFnError<Infallible>;

#[server(GetFailoverConfig, "/api")]
pub async fn get_failover_config() -> Result<FailoverConfig, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;
    let config = state.config.read().await;
    Ok(config.failover.clone())
}

#[server(SaveFailoverConfig, "/api")]
pub async fn save_failover_config(config: FailoverConfig) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    {
        let mut cfg = state.config.write().await;
        cfg.failover = config;
    }

    state
        .save_settings()
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    Ok(())
}

#[server(GetFailoverEvents, "/api")]
pub async fn get_failover_events(limit: u32) -> Result<Vec<FailoverEvent>, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    let events = state
        .db
        .with_writer(|conn| pi_kiosk_db::failover::list_failover_events(conn, limit))
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    Ok(events)
}

#[server(ClearFailoverEvents, "/api")]
pub async fn clear_failover_events() -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    state
        .db
        .with_writer(|conn| pi_kiosk_db::failover::clear_failover_events(conn))
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    Ok(())
}

#[server(TriggerFailover, "/api")]
pub async fn trigger_failover(to_source: WanSource) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    let senders = crate::state::get_live_senders()
        .ok_or_else(|| FnError::ServerError("no live state".into()))?;

    let wan = senders.wan.borrow().clone();
    let current = wan.active_source;

    let reason = format!("manual failover from {:?} to {:?}", current, to_source);

    state
        .db
        .with_writer(|conn| {
            pi_kiosk_db::failover::insert_failover_event(conn, Some(current), to_source, &reason)
        })
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    let new_wan = pi_kiosk_core::WanStatus {
        active_source: to_source,
        online: true,
        ..wan
    };
    let _ = senders.wan.send(new_wan);

    Ok(())
}
