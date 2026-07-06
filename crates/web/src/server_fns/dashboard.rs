use leptos::*;
use leptos_axum::extract;
use pi_kiosk_core::DashboardSnapshot;
use std::sync::Arc;

use crate::state::AppState;

#[server(GetDashboardSnapshot, "/api")]
pub async fn get_dashboard_snapshot() -> Result<DashboardSnapshot, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;
    Ok(state.0.snapshot())
}
