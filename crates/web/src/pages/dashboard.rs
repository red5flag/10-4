use leptos::*;

use crate::server_fns::dashboard::get_dashboard_snapshot;

#[component]
pub fn DashboardPage() -> impl IntoView {
    let snapshot = create_resource(|| (), |_| async { get_dashboard_snapshot().await });

    view! {
        <div class="page">
            <h1>"Dashboard"</h1>
            <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                {move || {
                    snapshot.get().map(|s| match s {
                        Ok(snap) => view! {
                            <div class="dashboard-grid">
                                <StatusCard title="Internet" status=if snap.wan.online { "Online" } else { "Offline" }.to_string() ok=snap.wan.online>
                                    <div class="card-body">
                                        <div class="stat-row">
                                            <span class="stat-label">"WAN"</span>
                                            <span class="stat-value">{format!("{:?}", snap.wan.active_source)}</span>
                                        </div>
                                        <div class="stat-row">
                                            <span class="stat-label">"IP"</span>
                                            <span class="stat-value">{snap.wan.ip_address.clone().unwrap_or("—".into())}</span>
                                        </div>
                                        <div class="stat-row">
                                            <span class="stat-label">"Gateway"</span>
                                            <span class="stat-value">{snap.wan.gateway.clone().unwrap_or("—".into())}</span>
                                        </div>
                                    </div>
                                </StatusCard>
                                <StatusCard title="Camera" status=if snap.camera.online { "Online" } else { "Offline" }.to_string() ok=snap.camera.online>
                                    <div class="card-body">
                                        <div class="stat-row">
                                            <span class="stat-label">"Recording"</span>
                                            <span class="stat-value">{if snap.recording { "● REC" } else { "Idle" }}</span>
                                        </div>
                                        <div class="stat-row">
                                            <span class="stat-label">"Night Vision"</span>
                                            <span class="stat-value">{if snap.camera.night_vision { "On" } else { "Off" }}</span>
                                        </div>
                                        <div class="stat-row">
                                            <span class="stat-label">"Resolution"</span>
                                            <span class="stat-value">{format!("{}×{}", snap.camera.resolution.0, snap.camera.resolution.1)}</span>
                                        </div>
                                    </div>
                                </StatusCard>
                                <StatusCard title="Detection" status=if snap.motion_active || snap.person_detected { "Active" } else { "Idle" }.to_string() ok=!snap.motion_active>
                                    <div class="card-body">
                                        <div class="stat-row">
                                            <span class="stat-label">"Motion"</span>
                                            <span class="stat-value">{if snap.motion_active { "⚠ Detected" } else { "Clear" }}</span>
                                        </div>
                                        <div class="stat-row">
                                            <span class="stat-label">"Person"</span>
                                            <span class="stat-value">{if snap.person_detected { "⚠ Detected" } else { "Clear" }}</span>
                                        </div>
                                    </div>
                                </StatusCard>
                                <StatusCard title="System" status="OK".to_string() ok=true>
                                    <div class="card-body">
                                        <ProgressBar label="CPU" value=snap.system.cpu_usage max=100.0 unit="%"/>
                                        <div class="stat-row">
                                            <span class="stat-label">"Temp"</span>
                                            <span class="stat-value">{format!("{:.1}°C", snap.system.cpu_temp_c)}</span>
                                        </div>
                                        <ProgressBar
                                            label="RAM"
                                            value=snap.system.mem_used_mb as f32
                                            max=snap.system.mem_total_mb as f32
                                            unit=" MB"
                                        />
                                        <ProgressBar
                                            label="Disk"
                                            value=snap.system.disk_used_gb
                                            max=snap.system.disk_total_gb
                                            unit=" GB"
                                        />
                                        <div class="stat-row">
                                            <span class="stat-label">"Net ↓↑"</span>
                                            <span class="stat-value">{format!("{} / {}", format_bytes(snap.system.net_rx_bytes), format_bytes(snap.system.net_tx_bytes))}</span>
                                        </div>
                                    </div>
                                </StatusCard>
                                <StatusCard title="Modem" status=if snap.modem.present { if snap.modem.connected { "Connected" } else { "Present" } } else { "Not detected" }.to_string() ok=snap.modem.present>
                                    <div class="card-body">
                                        <div class="stat-row">
                                            <span class="stat-label">"Operator"</span>
                                            <span class="stat-value">{snap.modem.operator.clone().unwrap_or("—".into())}</span>
                                        </div>
                                        <div class="stat-row">
                                            <span class="stat-label">"Signal"</span>
                                            <span class="stat-value">{snap.modem.signal_strength.map(|s| format!("{} dBm", s)).unwrap_or("—".into())}</span>
                                        </div>
                                        <div class="stat-row">
                                            <span class="stat-label">"Tech"</span>
                                            <span class="stat-value">{snap.modem.access_technology.clone().unwrap_or("—".into())}</span>
                                        </div>
                                    </div>
                                </StatusCard>
                                <div class="card">
                                    <div class="card-header">
                                        <span>{format!("Clients ({})", snap.connected_clients.len())}</span>
                                    </div>
                                    <div class="card-body">
                                        {if snap.connected_clients.is_empty() {
                                            view! { <div class="empty-state">"No connected clients"</div> }.into_view()
                                        } else {
                                            snap.connected_clients.iter().map(|c| view! {
                                                <div class="client-row">
                                                    <span class="client-name">{c.hostname.clone()}</span>
                                                    <span class="client-ip">{c.ip.clone()}</span>
                                                    <span class="client-mac">{c.mac.clone()}</span>
                                                </div>
                                            }).collect_view()
                                        }}
                                    </div>
                                </div>
                                <div class="card">
                                    <div class="card-header">"Recent Events"</div>
                                    <div class="card-body">
                                        {if snap.recent_events.is_empty() {
                                            view! { <div class="empty-state">"No recent events"</div> }.into_view()
                                        } else {
                                            snap.recent_events.iter().map(|e| view! {
                                                <div class="event-row">
                                                    <span class="event-kind" class:event-motion=e.kind == pi_kiosk_core::DetectionKind::Motion class:event-person=e.kind == pi_kiosk_core::DetectionKind::Person class:event-tamper=e.kind == pi_kiosk_core::DetectionKind::Tamper>
                                                        {format!("{:?}", e.kind)}
                                                    </span>
                                                    <span class="event-time">{e.timestamp.format("%H:%M:%S").to_string()}</span>
                                                </div>
                                            }).collect_view()
                                        }}
                                    </div>
                                </div>
                            </div>
                        },
                        Err(e) => view! { <div class="error">{format!("Error: {e}")}</div> },
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn StatusCard(title: &'static str, status: String, ok: bool, children: Children) -> impl IntoView {
    view! {
        <div class="card status-card" class:status-ok=ok class:status-warn=!ok>
            <div class="card-header">
                <span>{title}</span>
                <span class="status-badge" class:badge-ok=ok class:badge-warn=!ok>{status}</span>
            </div>
            {children()}
        </div>
    }
}

#[component]
fn ProgressBar(label: &'static str, value: f32, max: f32, unit: &'static str) -> impl IntoView {
    let pct = if max > 0.0 { (value / max * 100.0).min(100.0) } else { 0.0 };
    view! {
        <div class="progress-row">
            <div class="stat-row">
                <span class="stat-label">{label}</span>
                <span class="stat-value">{format!("{:.1} / {:.1}{}", value, max, unit)}</span>
            </div>
            <div class="progress-bar">
                <div class="progress-fill" style=format!("width: {}%", pct)></div>
            </div>
        </div>
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1_048_576 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1_073_741_824 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    }
}
