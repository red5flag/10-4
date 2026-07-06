use pi_kiosk_privileged::proto::{
    DnsmasqConfig, Encryption, HostapdConfig, NftRule, PrivRequest, PrivResponse, WifiBand,
};
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tracing::{error, info};

const SOCKET_PATH: &str = "/run/pi-kiosk/priv.sock";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pi_kiosk_privileged=info".into()),
        )
        .init();

    let socket_dir = Path::new(SOCKET_PATH).parent().unwrap();
    tokio::fs::create_dir_all(socket_dir).await.ok();

    if Path::new(SOCKET_PATH).exists() {
        tokio::fs::remove_file(SOCKET_PATH).await?;
    }

    let listener = UnixListener::bind(SOCKET_PATH)?;
    info!("privileged helper listening on {}", SOCKET_PATH);

    // Set permissions: root:pikiosk 0660
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o660);
        std::fs::set_permissions(SOCKET_PATH, perms)?;
    }

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(handle_connection(stream));
            }
            Err(e) => error!("accept error: {e}"),
        }
    }
}

async fn handle_connection(mut stream: tokio::net::UnixStream) {
    // Read length-prefixed JSON
    let len = match stream.read_u32().await {
        Ok(n) => n as usize,
        Err(_) => return,
    };
    let mut buf = vec![0u8; len];
    if stream.read_exact(&mut buf).await.is_err() {
        return;
    }

    let request: PrivRequest = match serde_json::from_slice(&buf) {
        Ok(r) => r,
        Err(e) => {
            let resp = PrivResponse::Error(format!("invalid request: {e}"));
            send_response(&mut stream, &resp).await;
            return;
        }
    };

    info!("received request: {:?}", request);

    let response = dispatch(request).await;
    send_response(&mut stream, &response).await;
}

async fn dispatch(request: PrivRequest) -> PrivResponse {
    match request {
        PrivRequest::Ping => PrivResponse::Pong,
        PrivRequest::HostapdStart { config } => {
            match write_hostapd_config(&config) {
                Ok(()) => {
                    match tokio::process::Command::new("systemctl")
                        .args(["restart", "hostapd"])
                        .output()
                        .await
                    {
                        Ok(o) if o.status.success() => {
                            info!("hostapd started (ssid={})", config.ssid);
                            PrivResponse::Ok
                        }
                        Ok(o) => PrivResponse::Error(format!(
                            "hostapd start failed: {}",
                            String::from_utf8_lossy(&o.stderr)
                        )),
                        Err(e) => PrivResponse::Error(format!("failed to run systemctl: {e}")),
                    }
                }
                Err(e) => PrivResponse::Error(format!("failed to write hostapd config: {e}")),
            }
        }
        PrivRequest::HostapdStop => {
            match tokio::process::Command::new("systemctl")
                .args(["stop", "hostapd"])
                .output()
                .await
            {
                Ok(o) if o.status.success() => {
                    info!("hostapd stopped");
                    PrivResponse::Ok
                }
                Ok(o) => PrivResponse::Error(format!(
                    "hostapd stop failed: {}",
                    String::from_utf8_lossy(&o.stderr)
                )),
                Err(e) => PrivResponse::Error(format!("failed to run systemctl: {e}")),
            }
        }
        PrivRequest::DnsmasqStart { config } => {
            match write_dnsmasq_config(&config) {
                Ok(()) => {
                    match tokio::process::Command::new("systemctl")
                        .args(["restart", "dnsmasq"])
                        .output()
                        .await
                    {
                        Ok(o) if o.status.success() => {
                            info!("dnsmasq started (iface={})", config.interface);
                            PrivResponse::Ok
                        }
                        Ok(o) => PrivResponse::Error(format!(
                            "dnsmasq start failed: {}",
                            String::from_utf8_lossy(&o.stderr)
                        )),
                        Err(e) => PrivResponse::Error(format!("failed to run systemctl: {e}")),
                    }
                }
                Err(e) => PrivResponse::Error(format!("failed to write dnsmasq config: {e}")),
            }
        }
        PrivRequest::DnsmasqStop => {
            match tokio::process::Command::new("systemctl")
                .args(["stop", "dnsmasq"])
                .output()
                .await
            {
                Ok(o) if o.status.success() => {
                    info!("dnsmasq stopped");
                    PrivResponse::Ok
                }
                Ok(o) => PrivResponse::Error(format!(
                    "dnsmasq stop failed: {}",
                    String::from_utf8_lossy(&o.stderr)
                )),
                Err(e) => PrivResponse::Error(format!("failed to run systemctl: {e}")),
            }
        }
        PrivRequest::NftablesApply { rules } => {
            match write_nftables_rules(&rules) {
                Ok(()) => {
                    match tokio::process::Command::new("nft")
                        .args(["-f", "/etc/nftables.d/pi-kiosk.nft"])
                        .output()
                        .await
                    {
                        Ok(o) if o.status.success() => {
                            info!("nftables applied ({} rules)", rules.len());
                            PrivResponse::Ok
                        }
                        Ok(o) => PrivResponse::Error(format!(
                            "nft apply failed: {}",
                            String::from_utf8_lossy(&o.stderr)
                        )),
                        Err(e) => PrivResponse::Error(format!("failed to run nft: {e}")),
                    }
                }
                Err(e) => PrivResponse::Error(format!("failed to write nftables rules: {e}")),
            }
        }
        PrivRequest::WireguardUp { config_path } => {
            match tokio::process::Command::new("wg-quick")
                .args(["up", &config_path])
                .output()
                .await
            {
                Ok(o) if o.status.success() => {
                    info!("wireguard up ({})", config_path);
                    PrivResponse::Ok
                }
                Ok(o) => PrivResponse::Error(format!(
                    "wg-quick up failed: {}",
                    String::from_utf8_lossy(&o.stderr)
                )),
                Err(e) => PrivResponse::Error(format!("failed to run wg-quick: {e}")),
            }
        }
        PrivRequest::WireguardDown { interface } => {
            match tokio::process::Command::new("wg-quick")
                .args(["down", &interface])
                .output()
                .await
            {
                Ok(o) if o.status.success() => {
                    info!("wireguard down ({})", interface);
                    PrivResponse::Ok
                }
                Ok(o) => PrivResponse::Error(format!(
                    "wg-quick down failed: {}",
                    String::from_utf8_lossy(&o.stderr)
                )),
                Err(e) => PrivResponse::Error(format!("failed to run wg-quick: {e}")),
            }
        }
        PrivRequest::OpenvpnUp { config_path } => {
            match tokio::process::Command::new("systemctl")
                .args(["start", "openvpn@pi-kiosk"])
                .output()
                .await
            {
                Ok(o) if o.status.success() => {
                    info!("openvpn started ({})", config_path);
                    PrivResponse::Ok
                }
                Ok(o) => PrivResponse::Error(format!(
                    "openvpn start failed: {}",
                    String::from_utf8_lossy(&o.stderr)
                )),
                Err(e) => PrivResponse::Error(format!("failed to run systemctl: {e}")),
            }
        }
        PrivRequest::OpenvpnDown { interface: _ } => {
            match tokio::process::Command::new("systemctl")
                .args(["stop", "openvpn@pi-kiosk"])
                .output()
                .await
            {
                Ok(o) if o.status.success() => {
                    info!("openvpn stopped");
                    PrivResponse::Ok
                }
                Ok(o) => PrivResponse::Error(format!(
                    "openvpn stop failed: {}",
                    String::from_utf8_lossy(&o.stderr)
                )),
                Err(e) => PrivResponse::Error(format!("failed to run systemctl: {e}")),
            }
        }
        PrivRequest::TorStart { config_path } => {
            match tokio::process::Command::new("tor")
                .args(["-f", &config_path])
                .spawn()
            {
                Ok(child) => {
                    info!("tor started (pid={:?}, config={})", child.id(), config_path);
                    PrivResponse::Ok
                }
                Err(e) => PrivResponse::Error(format!("failed to start tor: {e}")),
            }
        }
        PrivRequest::TorStop => {
            let kill = tokio::process::Command::new("pkill")
                .args(["-x", "tor"])
                .output()
                .await;
            match kill {
                Ok(o) if o.status.success() => {
                    info!("tor stopped");
                    PrivResponse::Ok
                }
                Ok(_) => {
                    info!("tor not running");
                    PrivResponse::Ok
                }
                Err(e) => PrivResponse::Error(format!("failed to kill tor: {e}")),
            }
        }
        PrivRequest::KillSwitchEnable { interface } => {
            let rules = format!(
                "table inet pi-kiosk-killswitch {{\n\
                 \x20 chain killswitch {{\n\
                 \x20   meta oifname != \"{iface}\" drop;\n\
                 \x20   ip daddr 127.0.0.0/8 accept;\n\
                 \x20 }}\n\
                 }}\n",
                iface = interface
            );
            let path = "/etc/nftables.d/pi-kiosk-killswitch.nft";
            if let Err(e) = std::fs::write(path, &rules) {
                PrivResponse::Error(format!("failed to write kill-switch rules: {e}"))
            } else {
                match tokio::process::Command::new("nft")
                    .args(["-f", path])
                    .output()
                    .await
                {
                    Ok(o) if o.status.success() => {
                        info!("kill-switch enabled (interface={})", interface);
                        PrivResponse::Ok
                    }
                    Ok(o) => PrivResponse::Error(format!(
                        "nft apply failed: {}",
                        String::from_utf8_lossy(&o.stderr)
                    )),
                    Err(e) => PrivResponse::Error(format!("failed to run nft: {e}")),
                }
            }
        }
        PrivRequest::KillSwitchDisable => {
            match tokio::process::Command::new("nft")
                .args(["delete", "table", "inet", "pi-kiosk-killswitch"])
                .output()
                .await
            {
                Ok(o) if o.status.success() => {
                    info!("kill-switch disabled");
                    PrivResponse::Ok
                }
                Ok(_) => {
                    info!("kill-switch table not present");
                    PrivResponse::Ok
                }
                Err(e) => PrivResponse::Error(format!("failed to run nft: {e}")),
            }
        }
        PrivRequest::InterfaceUp { interface } => {
            match tokio::process::Command::new("ip")
                .args(["link", "set", &interface, "up"])
                .output()
                .await
            {
                Ok(o) if o.status.success() => {
                    info!("interface {} up", interface);
                    PrivResponse::Ok
                }
                Ok(o) => PrivResponse::Error(format!(
                    "interface up failed: {}",
                    String::from_utf8_lossy(&o.stderr)
                )),
                Err(e) => PrivResponse::Error(format!("failed to run ip: {e}")),
            }
        }
        PrivRequest::InterfaceDown { interface } => {
            match tokio::process::Command::new("ip")
                .args(["link", "set", &interface, "down"])
                .output()
                .await
            {
                Ok(o) if o.status.success() => {
                    info!("interface {} down", interface);
                    PrivResponse::Ok
                }
                Ok(o) => PrivResponse::Error(format!(
                    "interface down failed: {}",
                    String::from_utf8_lossy(&o.stderr)
                )),
                Err(e) => PrivResponse::Error(format!("failed to run ip: {e}")),
            }
        }
        PrivRequest::SetDefaultRoute { interface, gateway } => {
            let del = tokio::process::Command::new("ip")
                .args(["route", "del", "default"])
                .output()
                .await;
            let _ = del;

            match tokio::process::Command::new("ip")
                .args(["route", "add", "default", "via", &gateway, "dev", &interface])
                .output()
                .await
            {
                Ok(o) if o.status.success() => {
                    info!("default route set via {} dev {}", gateway, interface);
                    PrivResponse::Ok
                }
                Ok(o) => PrivResponse::Error(format!(
                    "set default route failed: {}",
                    String::from_utf8_lossy(&o.stderr)
                )),
                Err(e) => PrivResponse::Error(format!("failed to run ip: {e}")),
            }
        }
    }
}

fn write_hostapd_config(config: &HostapdConfig) -> anyhow::Result<()> {
    let band_str = match config.band {
        WifiBand::Band24G => "g",
        WifiBand::Band5G => "a",
    };

    let mut conf = format!(
        "interface={}\ndriver=nl80211\nssid={}\nhw_mode={}\nchannel={}\ncountry_code={}\n",
        config.interface, config.ssid, band_str, config.channel, config.country_code
    );

    if config.hidden {
        conf.push_str("ignore_broadcast_ssid=1\n");
    }

    match config.encryption {
        Encryption::Open => {
            conf.push_str("auth_algs=1\n");
        }
        Encryption::Wpa2Psk => {
            conf.push_str(&format!(
                "wpa=2\nwpa_passphrase={}\nwpa_key_mgmt=WPA-PSK\nrsn_pairwise=CCMP\n",
                config.password
            ));
        }
        Encryption::Wpa3Sae => {
            conf.push_str(&format!(
                "wpa=2\nwpa_passphrase={}\nwpa_key_mgmt=WPA-PSK SAE\nrsn_pairwise=CCMP\nsae_require_mfp=1\nieee80211w=2\n",
                config.password
            ));
        }
    }

    let dir = "/etc/hostapd";
    std::fs::create_dir_all(dir)?;
    std::fs::write(format!("{}/hostapd.conf", dir), conf)?;
    Ok(())
}

fn write_dnsmasq_config(config: &DnsmasqConfig) -> anyhow::Result<()> {
    let mut conf = format!(
        "interface={}\nbind-interfaces\ndhcp-range={},{},{}\n",
        config.interface, config.dhcp_range_start, config.dhcp_range_end, config.lease_time
    );

    for dns in &config.dns_servers {
        conf.push_str(&format!("server={}\n", dns));
    }

    let dir = "/etc/dnsmasq.d";
    std::fs::create_dir_all(dir)?;
    std::fs::write(format!("{}/pi-kiosk.conf", dir), conf)?;
    Ok(())
}

fn write_nftables_rules(rules: &[NftRule]) -> anyhow::Result<()> {
    let mut conf = String::new();
    for rule in rules {
        conf.push_str(&format!(
            "table {} {{ chain {} {{ {} }} }}\n",
            rule.table, rule.chain, rule.rule
        ));
    }

    let dir = "/etc/nftables.d";
    std::fs::create_dir_all(dir)?;
    std::fs::write(format!("{}/pi-kiosk.nft", dir), conf)?;
    Ok(())
}

async fn send_response(stream: &mut tokio::net::UnixStream, resp: &PrivResponse) {
    let data = match serde_json::to_vec(resp) {
        Ok(d) => d,
        Err(e) => {
            error!("failed to serialize response: {e}");
            return;
        }
    };
    if stream.write_u32(data.len() as u32).await.is_err() {
        return;
    }
    let _ = stream.write_all(&data).await;
}
