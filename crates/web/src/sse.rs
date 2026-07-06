use axum::response::sse::{Event, KeepAlive, Sse};
use futures::stream::Stream;
use std::convert::Infallible;
use std::time::Duration;

/// SSE endpoint for live dashboard updates.
/// Broadcasts system stats every 3 seconds, plus heartbeat.
pub async fn events_sse() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = futures::stream::unfold((), |()| async move {
        tokio::time::sleep(Duration::from_secs(3)).await;

        let event = if let Some(senders) = crate::state::get_live_senders() {
            let stats = senders.system_stats.borrow().clone();
            let wan = senders.wan.borrow().clone();
            let camera = senders.camera.borrow().clone();
            let modem = senders.modem.borrow().clone();
            let recording = *senders.recording.borrow();
            let motion = *senders.motion_active.borrow();
            let person = *senders.person_detected.borrow();

            let payload = serde_json::json!({
                "system": {
                    "cpu_usage": stats.cpu_usage,
                    "cpu_temp_c": stats.cpu_temp_c,
                    "mem_used_mb": stats.mem_used_mb,
                    "mem_total_mb": stats.mem_total_mb,
                    "disk_used_gb": stats.disk_used_gb,
                    "disk_total_gb": stats.disk_total_gb,
                    "net_rx_bytes": stats.net_rx_bytes,
                    "net_tx_bytes": stats.net_tx_bytes,
                },
                "wan": {
                    "active_source": format!("{:?}", wan.active_source),
                    "online": wan.online,
                    "ip_address": wan.ip_address,
                    "gateway": wan.gateway,
                },
                "camera": {
                    "online": camera.online,
                    "recording": recording,
                    "night_vision": camera.night_vision,
                },
                "modem": {
                    "present": modem.present,
                    "connected": modem.connected,
                    "operator": modem.operator,
                    "signal_strength": modem.signal_strength,
                },
                "detection": {
                    "motion_active": motion,
                    "person_detected": person,
                },
            });

            Event::default()
                .event("stats")
                .data(payload.to_string())
        } else {
            Event::default()
                .event("heartbeat")
                .data(chrono::Utc::now().to_rfc3339())
        };

        Some((Ok(event), ()))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
