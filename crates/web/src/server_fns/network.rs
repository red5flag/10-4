use leptos::*;
use leptos_axum::extract;
use pi_kiosk_core::{ConnectedClient, NetworkConfig, WanStatus};
use pi_kiosk_db::network::FirewallRule;
use pi_kiosk_network::InterfaceInfo;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state::AppState;

#[server(GetWanStatus, "/api")]
pub async fn get_wan_status() -> Result<WanStatus, ServerFnError> {
    let senders = crate::state::get_live_senders()
        .ok_or_else(|| ServerFnError::<std::convert::Infallible>::ServerError("no live state".into()))?;
    let status = senders.wan.borrow().clone();
    Ok(status)
}

#[server(GetConnectedClients, "/api")]
pub async fn get_connected_clients() -> Result<Vec<ConnectedClient>, ServerFnError> {
    let senders = crate::state::get_live_senders()
        .ok_or_else(|| ServerFnError::<std::convert::Infallible>::ServerError("no live state".into()))?;
    let clients = senders.connected_clients.borrow().clone();
    Ok(clients)
}

#[server(GetInterfaces, "/api")]
pub async fn get_interfaces() -> Result<Vec<InterfaceInfo>, ServerFnError> {
    let _state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    let monitor = pi_kiosk_network::NetworkMonitor::new();
    let snapshot = monitor.snapshot();
    Ok(snapshot.interfaces)
}

#[server(GetNetworkConfig, "/api")]
pub async fn get_network_config() -> Result<NetworkConfig, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;
    let config = state.config.read().await;
    Ok(config.network.clone())
}

#[server(SaveNetworkConfig, "/api")]
pub async fn save_network_config(config: NetworkConfig) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    {
        let mut cfg = state.config.write().await;
        cfg.network = config;
    }

    state.save_settings().await
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    Ok(())
}

#[server(GetFirewallRules, "/api")]
pub async fn get_firewall_rules() -> Result<Vec<FirewallRule>, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    let rules = state
        .db
        .with_writer(|conn| pi_kiosk_db::network::list_firewall_rules(conn))
        .await
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    Ok(rules)
}

#[server(AddFirewallRule, "/api")]
pub async fn add_firewall_rule(
    name: String,
    proto: String,
    src_port: Option<String>,
    dest_ip: String,
    dest_port: String,
) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    let rule = FirewallRule {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        proto,
        src_port,
        dest_ip,
        dest_port,
        enabled: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    state
        .db
        .with_writer(|conn| pi_kiosk_db::network::insert_firewall_rule(conn, &rule))
        .await
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    Ok(())
}

#[server(DeleteFirewallRule, "/api")]
pub async fn delete_firewall_rule(id: String) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    state
        .db
        .with_writer(|conn| pi_kiosk_db::network::delete_firewall_rule(conn, &id))
        .await
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    Ok(())
}

#[server(ToggleFirewallRule, "/api")]
pub async fn toggle_firewall_rule(id: String, enabled: bool) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    state
        .db
        .with_writer(|conn| pi_kiosk_db::network::toggle_firewall_rule(conn, &id, enabled))
        .await
        .map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbWifiAdapter {
    pub interface: String,
    pub vendor: String,
    pub product: String,
    pub driver: String,
    pub supports_5g: bool,
    pub supports_ap: bool,
}

#[server(GetUsbWifiAdapters, "/api")]
pub async fn get_usb_wifi_adapters() -> Result<Vec<UsbWifiAdapter>, ServerFnError> {
    let _state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| ServerFnError::<std::convert::Infallible>::ServerError(e.to_string()))?;

    let mut adapters = Vec::new();

    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("wl") {
                continue;
            }

            let device_path = format!("/sys/class/net/{}/device", name);
            if !std::path::Path::new(&device_path).exists() {
                continue;
            }

            let vendor = std::fs::read_to_string(format!("{}/vendor", device_path))
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            let product = std::fs::read_to_string(format!("{}/product", device_path))
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            let driver = std::fs::read_to_string(format!("{}/driver", device_path))
                .ok()
                .and_then(|s| {
                    std::path::Path::new(&s.trim())
                        .file_name()
                        .map(|f| f.to_string_lossy().to_string())
                })
                .unwrap_or_default();

            let supports_5g = check_5g_support(&name);
            let supports_ap = check_ap_support(&name);

            adapters.push(UsbWifiAdapter {
                interface: name,
                vendor,
                product,
                driver,
                supports_5g,
                supports_ap,
            });
        }
    }

    Ok(adapters)
}

fn check_5g_support(iface: &str) -> bool {
    std::process::Command::new("iw")
        .args(["dev", iface, "info"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.contains("5180") || stdout.contains("5200") || stdout.contains("5745")
        })
        .unwrap_or(false)
}

fn check_ap_support(iface: &str) -> bool {
    std::process::Command::new("iw")
        .args(["dev", iface, "info"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.contains("AP")
        })
        .unwrap_or(false)
}
