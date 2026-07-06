use leptos::*;
use leptos_axum::extract;
use pi_kiosk_core::DetectionEvent;
use std::sync::Arc;

use crate::state::AppState;

#[server(GetRecentEvents, "/api")]
pub async fn get_recent_events(limit: u32) -> Result<Vec<DetectionEvent>, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    let events = state
        .db
        .with_writer(|conn| pi_kiosk_db::events::recent_events(conn, limit))
        .await
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    Ok(events)
}

#[server(GetDetectionState, "/api")]
pub async fn get_detection_state() -> Result<DetectionState, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    let config = state.config.read().await.clone();
    Ok(DetectionState {
        motion_enabled: config.detection.motion_enabled,
        person_enabled: config.detection.person_enabled,
        tamper_enabled: config.detection.tamper_enabled,
        motion_threshold: config.detection.motion_threshold,
        person_confidence: config.detection.person_confidence,
        cooldown_seconds: config.detection.cooldown_seconds,
        capture_interval_ms: config.detection.capture_interval_ms,
    })
}

#[server(UpdateDetectionConfig, "/api")]
pub async fn update_detection_config(
    motion_enabled: Option<bool>,
    person_enabled: Option<bool>,
    tamper_enabled: Option<bool>,
    motion_threshold: Option<f32>,
    person_confidence: Option<f32>,
    cooldown_seconds: Option<u32>,
    capture_interval_ms: Option<u32>,
) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    {
        let mut config = state.config.write().await;
        if let Some(v) = motion_enabled { config.detection.motion_enabled = v; }
        if let Some(v) = person_enabled { config.detection.person_enabled = v; }
        if let Some(v) = tamper_enabled { config.detection.tamper_enabled = v; }
        if let Some(v) = motion_threshold { config.detection.motion_threshold = v; }
        if let Some(v) = person_confidence { config.detection.person_confidence = v; }
        if let Some(v) = cooldown_seconds { config.detection.cooldown_seconds = v; }
        if let Some(v) = capture_interval_ms { config.detection.capture_interval_ms = v; }
    }

    state
        .save_settings()
        .await
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    Ok(())
}

#[server(DeleteEvent, "/api")]
pub async fn delete_event(event_id: String) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    state
        .db
        .with_writer(|conn| {
            pi_kiosk_db::events::delete_event(conn, &event_id)
        })
        .await
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    Ok(())
}

#[server(ClearAllEvents, "/api")]
pub async fn clear_all_events() -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    state
        .db
        .with_writer(|conn| {
            pi_kiosk_db::events::clear_all_events(conn)
        })
        .await
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DetectionState {
    pub motion_enabled: bool,
    pub person_enabled: bool,
    pub tamper_enabled: bool,
    pub motion_threshold: f32,
    pub person_confidence: f32,
    pub cooldown_seconds: u32,
    pub capture_interval_ms: u32,
}
