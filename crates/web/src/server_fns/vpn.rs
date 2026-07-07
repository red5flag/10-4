use leptos::*;
use leptos_axum::extract;
use pi_kiosk_core::{TorConfig, VpnConfig, VpnMode, VpnStatus, VpnType, WireguardConfig};
use std::convert::Infallible;
use std::sync::Arc;

use crate::state::AppState;

type FnError = ServerFnError<Infallible>;

#[server(GetVpnStatus, "/api")]
pub async fn get_vpn_status() -> Result<VpnStatus, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    let mut status = VpnStatus::default();

    let wg_output = tokio::process::Command::new("wg")
        .args(["show"])
        .output()
        .await;

    if let Ok(o) = wg_output {
        if o.status.success() {
            let text = String::from_utf8_lossy(&o.stdout);
            if !text.trim().is_empty() {
                status.wireguard_active = true;
                status.wireguard_interface = text
                    .lines()
                    .next()
                    .map(|l| l.split(':').next().unwrap_or("").trim().to_string());
            }
        }
    }

    let ovpn_output = tokio::process::Command::new("systemctl")
        .args(["is-active", "openvpn@pi-kiosk"])
        .output()
        .await;

    if let Ok(o) = ovpn_output {
        let text = String::from_utf8_lossy(&o.stdout);
        status.openvpn_active = text.trim() == "active";
    }

    let tor_config = state.config.read().await.tor.clone();
    if tor_config.enabled {
        let tor_check = tokio::process::Command::new("pgrep")
            .args(["-x", "tor"])
            .output()
            .await;
        if let Ok(o) = tor_check {
            if o.status.success() {
                status.tor_active = true;
                status.tor_socks_port = Some(tor_config.socks_port);
            }
        }
    }

    let nft_output = tokio::process::Command::new("nft")
        .args(["list", "table", "inet", "pi-kiosk-killswitch"])
        .output()
        .await;

    if let Ok(o) = nft_output {
        status.kill_switch_active = o.status.success();
    }

    Ok(status)
}

#[server(GetVpnConfigs, "/api")]
pub async fn get_vpn_configs() -> Result<Vec<VpnConfig>, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    let configs = state
        .db
        .with_writer(|conn| pi_kiosk_db::vpn::list_vpn_configs(conn))
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    Ok(configs)
}

#[server(AddVpnConfig, "/api")]
pub async fn add_vpn_config(
    name: String,
    vpn_type: VpnType,
    mode: VpnMode,
    config_path: String,
    kill_switch: bool,
) -> Result<VpnConfig, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    let config = VpnConfig {
        id: pi_kiosk_db::vpn::new_vpn_id(),
        name,
        vpn_type,
        mode,
        config_path,
        enabled: false,
        kill_switch,
        created_at: chrono::Utc::now(),
    };

    state
        .db
        .with_writer(|conn| pi_kiosk_db::vpn::insert_vpn_config(conn, &config))
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    Ok(config)
}

#[server(DeleteVpnConfig, "/api")]
pub async fn delete_vpn_config(id: String) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    state
        .db
        .with_writer(|conn| pi_kiosk_db::vpn::delete_vpn_config(conn, &id))
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    Ok(())
}

#[server(ToggleVpnConfig, "/api")]
pub async fn toggle_vpn_config(id: String, enabled: bool) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    state
        .db
        .with_writer(|conn| pi_kiosk_db::vpn::set_vpn_enabled(conn, &id, enabled))
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    Ok(())
}

#[server(GenerateWireguardConfig, "/api")]
pub async fn generate_wireguard_config(config: WireguardConfig) -> Result<String, ServerFnError> {
    let mut conf = String::new();

    conf.push_str("[Interface]\n");
    conf.push_str(&format!("PrivateKey = {}\n", config.private_key));
    conf.push_str(&format!("Address = {}\n", config.address));

    if let Some(port) = config.listen_port {
        conf.push_str(&format!("ListenPort = {}\n", port));
    }

    if !config.dns.is_empty() {
        conf.push_str(&format!("DNS = {}\n", config.dns.join(", ")));
    }

    for peer in &config.peers {
        conf.push_str("\n[Peer]\n");
        conf.push_str(&format!("PublicKey = {}\n", peer.public_key));
        conf.push_str(&format!("AllowedIPs = {}\n", peer.allowed_ips));

        if let Some(ep) = &peer.endpoint {
            conf.push_str(&format!("Endpoint = {}\n", ep));
        }

        if let Some(psk) = &peer.preshared_key {
            conf.push_str(&format!("PresharedKey = {}\n", psk));
        }

        if let Some(ka) = peer.persistent_keepalive {
            conf.push_str(&format!("PersistentKeepalive = {}\n", ka));
        }
    }

    Ok(conf)
}

#[server(ImportVpnConfig, "/api")]
pub async fn import_vpn_config(name: String, config_text: String, vpn_type: VpnType) -> Result<VpnConfig, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    let config_dir = "/etc/pi-kiosk/vpn";
    let _ = std::fs::create_dir_all(config_dir);

    let id = pi_kiosk_db::vpn::new_vpn_id();
    let ext = match vpn_type {
        VpnType::Wireguard => "conf",
        VpnType::Openvpn => "ovpn",
    };

    let config_path = format!("{}/{}.{}", config_dir, id, ext);

    std::fs::write(&config_path, &config_text)
        .map_err(|e| FnError::ServerError(format!("failed to write config: {e}")))?;

    let config = VpnConfig {
        id: id.clone(),
        name,
        vpn_type,
        mode: VpnMode::Client,
        config_path,
        enabled: false,
        kill_switch: false,
        created_at: chrono::Utc::now(),
    };

    state
        .db
        .with_writer(|conn| pi_kiosk_db::vpn::insert_vpn_config(conn, &config))
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    Ok(config)
}

#[server(ExportVpnConfig, "/api")]
pub async fn export_vpn_config(id: String) -> Result<String, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    let configs = state
        .db
        .with_writer(|conn| pi_kiosk_db::vpn::list_vpn_configs(conn))
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    let config = configs
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| FnError::ServerError("config not found".into()))?;

    std::fs::read_to_string(&config.config_path)
        .map_err(|e| FnError::ServerError(format!("failed to read config: {e}")))
        .map_err(|e| e.into())
}

#[server(GetTorConfig, "/api")]
pub async fn get_tor_config() -> Result<TorConfig, ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;
    let tor = state.config.read().await.tor.clone();
    Ok(tor)
}

#[server(SaveTorConfig, "/api")]
pub async fn save_tor_config(config: TorConfig) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    {
        let mut cfg = state.config.write().await;
        cfg.tor = config;
    }

    state
        .save_settings()
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    Ok(())
}

#[server(StartTor, "/api")]
pub async fn start_tor() -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    let tor_config = state.config.read().await.tor.clone();

    let torrc_path = "/etc/pi-kiosk/tor/torrc";
    let _ = std::fs::create_dir_all("/etc/pi-kiosk/tor");

    let mut torrc = String::new();
    torrc.push_str(&format!("SocksPort {}\n", tor_config.socks_port));
    torrc.push_str(&format!("ControlPort {}\n", tor_config.control_port));
    torrc.push_str("DataDirectory /var/lib/pi-kiosk/tor\n");
    torrc.push_str("RunAsDaemon 1\n");

    if tor_config.use_bridges && !tor_config.bridge_lines.is_empty() {
        torrc.push_str("UseBridges 1\n");
        for line in &tor_config.bridge_lines {
            torrc.push_str(&format!("Bridge {}\n", line));
        }
    }

    std::fs::write(torrc_path, &torrc)
        .map_err(|e| FnError::ServerError(format!("failed to write torrc: {e}")))?;

    crate::priv_client::send_request_ok(
        &pi_kiosk_privileged::proto::PrivRequest::TorStart {
            config_path: torrc_path.to_string(),
        },
    )
    .await
    .map_err(|e| FnError::ServerError(e).into())
}

#[server(StopTor, "/api")]
pub async fn stop_tor() -> Result<(), ServerFnError> {
    crate::priv_client::send_request_ok(
        &pi_kiosk_privileged::proto::PrivRequest::TorStop,
    )
    .await
    .map_err(|e| FnError::ServerError(e).into())
}

#[server(StartVpn, "/api")]
pub async fn start_vpn(id: String) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    let configs = state
        .db
        .with_writer(|conn| pi_kiosk_db::vpn::list_vpn_configs(conn))
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    let config = configs
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| FnError::ServerError("config not found".into()))?;

    match config.vpn_type {
        VpnType::Wireguard => {
            crate::priv_client::send_request_ok(
                &pi_kiosk_privileged::proto::PrivRequest::WireguardUp {
                    config_path: config.config_path,
                },
            )
            .await
            .map_err(|e| FnError::ServerError(e).into())
        }
        VpnType::Openvpn => {
            crate::priv_client::send_request_ok(
                &pi_kiosk_privileged::proto::PrivRequest::OpenvpnUp {
                    config_path: config.config_path,
                },
            )
            .await
            .map_err(|e| FnError::ServerError(e).into())
        }
    }
}

#[server(StopVpn, "/api")]
pub async fn stop_vpn(id: String) -> Result<(), ServerFnError> {
    let state: axum::Extension<Arc<AppState>> =
        extract().await.map_err(|e| FnError::ServerError(e.to_string()))?;

    let configs = state
        .db
        .with_writer(|conn| pi_kiosk_db::vpn::list_vpn_configs(conn))
        .await
        .map_err(|e| FnError::ServerError(e.to_string()))?;

    let config = configs
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| FnError::ServerError("config not found".into()))?;

    match config.vpn_type {
        VpnType::Wireguard => {
            let iface = config
                .config_path
                .split('/')
                .last()
                .unwrap_or("wg0")
                .strip_suffix(".conf")
                .unwrap_or("wg0");

            crate::priv_client::send_request_ok(
                &pi_kiosk_privileged::proto::PrivRequest::WireguardDown {
                    interface: iface.to_string(),
                },
            )
            .await
            .map_err(|e| FnError::ServerError(e).into())
        }
        VpnType::Openvpn => {
            crate::priv_client::send_request_ok(
                &pi_kiosk_privileged::proto::PrivRequest::OpenvpnDown {
                    interface: config.config_path,
                },
            )
            .await
            .map_err(|e| FnError::ServerError(e).into())
        }
    }
}
