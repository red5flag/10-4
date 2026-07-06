use leptos::*;
use leptos_axum::extract;
use pi_kiosk_camera::ClipInfo;
use std::sync::Arc;

use crate::state::AppState;

#[server(StartRecording, "/api")]
pub async fn start_recording() -> Result<String, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    let config = state.config.read().await.clone();
    let clip_dir = config.storage.clip_dir.clone();

    let senders = crate::state::get_live_senders()
        .ok_or_else(|| ServerFnError::<std::convert::Infallible>::ServerError("no live state".into()))?;

    let filename = senders
        .camera_controller
        .start_recording(&clip_dir)
        .await
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    Ok(filename)
}

#[server(StopRecording, "/api")]
pub async fn stop_recording() -> Result<(), ServerFnError> {
    let senders = crate::state::get_live_senders()
        .ok_or_else(|| ServerFnError::<std::convert::Infallible>::ServerError("no live state".into()))?;

    senders
        .camera_controller
        .stop_recording()
        .await
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    Ok(())
}

#[server(TakeSnapshot, "/api")]
pub async fn take_snapshot() -> Result<String, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    let config = state.config.read().await.clone();
    let clip_dir = config.storage.clip_dir.clone();

    let senders = crate::state::get_live_senders()
        .ok_or_else(|| ServerFnError::<std::convert::Infallible>::ServerError("no live state".into()))?;

    let filename = senders
        .camera_controller
        .snapshot(&clip_dir)
        .await
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    Ok(filename)
}

#[server(SetNightVision, "/api")]
pub async fn set_night_vision(enabled: bool) -> Result<(), ServerFnError> {
    let senders = crate::state::get_live_senders()
        .ok_or_else(|| ServerFnError::<std::convert::Infallible>::ServerError("no live state".into()))?;

    senders
        .camera_controller
        .set_night_vision(enabled)
        .await
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    Ok(())
}

#[server(IsRecording, "/api")]
pub async fn is_recording() -> Result<bool, ServerFnError> {
    let senders = crate::state::get_live_senders()
        .ok_or_else(|| ServerFnError::<std::convert::Infallible>::ServerError("no live state".into()))?;

    Ok(senders.camera_controller.is_recording().await)
}

#[server(ListClips, "/api")]
pub async fn list_clips() -> Result<Vec<ClipInfo>, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    let config = state.config.read().await.clone();
    let mgr = pi_kiosk_camera::ClipManager::new(
        &config.storage.clip_dir,
        config.storage.max_retention_days,
        config.storage.max_disk_usage_pct,
    );

    Ok(mgr.list_clips())
}

#[server(ListSnapshots, "/api")]
pub async fn list_snapshots() -> Result<Vec<ClipInfo>, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    let config = state.config.read().await.clone();
    let mgr = pi_kiosk_camera::ClipManager::new(
        &config.storage.clip_dir,
        config.storage.max_retention_days,
        config.storage.max_disk_usage_pct,
    );

    Ok(mgr.list_snapshots())
}

#[server(DeleteClip, "/api")]
pub async fn delete_clip(filename: String) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    let config = state.config.read().await.clone();
    let mgr = pi_kiosk_camera::ClipManager::new(
        &config.storage.clip_dir,
        config.storage.max_retention_days,
        config.storage.max_disk_usage_pct,
    );

    mgr.delete_clip(&filename)
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    Ok(())
}
