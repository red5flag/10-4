use pi_kiosk_core::{ConnectedClient, WanSource, WanStatus};
use std::collections::HashMap;
use std::net::IpAddr;
use std::process::Command;

pub struct NetworkMonitor;

#[derive(Debug, Clone)]
pub struct NetworkSnapshot {
    pub wan: WanStatus,
    pub connected_clients: Vec<ConnectedClient>,
    pub interfaces: Vec<InterfaceInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InterfaceInfo {
    pub name: String,
    pub state: String,
    pub mac: String,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub link_type: String,
    pub speed_mbps: Option<u32>,
}

impl NetworkMonitor {
    pub fn new() -> Self {
        Self
    }

    pub fn snapshot(&self) -> NetworkSnapshot {
        let interfaces = self.read_interfaces();
        let wan = self.determine_wan_status(&interfaces);
        let connected_clients = self.read_connected_clients();

        NetworkSnapshot {
            wan,
            connected_clients,
            interfaces,
        }
    }

    fn read_interfaces(&self) -> Vec<InterfaceInfo> {
        let output = Command::new("ip")
            .args(["-j", "-d", "addr", "show"])
            .output();

        let mut interfaces = Vec::new();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Ok(json) = serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
                    for iface in json {
                        let name = iface.get("ifname").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        if name.is_empty() || name == "lo" {
                            continue;
                        }

                        let state = iface.get("operstate").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                        let mac = iface.get("address").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let link_type = iface.get("link_type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();

                        let mut ipv4 = None;
                        let mut ipv6 = None;

                        if let Some(addr_info) = iface.get("addr_info").and_then(|v| v.as_array()) {
                            for addr in addr_info {
                                let family = addr.get("family").and_then(|v| v.as_str()).unwrap_or("");
                                let local = addr.get("local").and_then(|v| v.as_str()).unwrap_or("");
                                if family == "inet" && !local.is_empty() {
                                    ipv4 = Some(local.to_string());
                                } else if family == "inet6" && !local.is_empty() && !local.starts_with("fe80") {
                                    ipv6 = Some(local.to_string());
                                }
                            }
                        }

                        let speed_mbps = if state == "UP" && (link_type == "ether" || name.starts_with("eth")) {
                            self.read_link_speed(&name)
                        } else {
                            None
                        };

                        interfaces.push(InterfaceInfo {
                            name,
                            state,
                            mac,
                            ipv4,
                            ipv6,
                            link_type,
                            speed_mbps,
                        });
                    }
                }
            }
            _ => {
                tracing::debug!("ip addr show failed, using fallback");
                interfaces = self.fallback_interfaces();
            }
        }

        interfaces
    }

    fn read_link_speed(&self, iface: &str) -> Option<u32> {
        let path = format!("/sys/class/net/{}/speed", iface);
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
    }

    fn fallback_interfaces(&self) -> Vec<InterfaceInfo> {
        let mut interfaces = Vec::new();

        if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name == "lo" {
                    continue;
                }

                let state_path = format!("/sys/class/net/{}/operstate", name);
                let state = std::fs::read_to_string(&state_path)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|_| "unknown".to_string());

                let mac = std::fs::read_to_string(format!("/sys/class/net/{}/address", name))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();

                let link_type = if name.starts_with("eth") || name.starts_with("en") {
                    "ether".to_string()
                } else if name.starts_with("wlan") || name.starts_with("wl") {
                    "wifi".to_string()
                } else {
                    "unknown".to_string()
                };

                interfaces.push(InterfaceInfo {
                    name,
                    state,
                    mac,
                    ipv4: None,
                    ipv6: None,
                    link_type,
                    speed_mbps: None,
                });
            }
        }

        interfaces
    }

    fn determine_wan_status(&self, interfaces: &[InterfaceInfo]) -> WanStatus {
        let ethernet_up = interfaces.iter().any(|i| {
            (i.name.starts_with("eth") || i.name.starts_with("en"))
                && i.state == "up"
                && i.ipv4.is_some()
        });

        let wifi_up = interfaces.iter().any(|i| {
            (i.name.starts_with("wlan") || i.name.starts_with("wl"))
                && i.state == "up"
                && i.ipv4.is_some()
        });

        let cellular_up = interfaces.iter().any(|i| {
            (i.name.starts_with("wwan") || i.name.starts_with("usb") || i.name.contains("cellular"))
                && i.state == "up"
                && i.ipv4.is_some()
        });

        let active_source = if ethernet_up {
            WanSource::Ethernet
        } else if wifi_up {
            WanSource::Wifi
        } else if cellular_up {
            WanSource::Cellular
        } else {
            WanSource::Ethernet
        };

        let online = ethernet_up || wifi_up || cellular_up;

        let (ip_address, gateway) = self.read_default_route();

        WanStatus {
            active_source,
            online,
            ethernet_up,
            wifi_up,
            cellular_up,
            ip_address,
            gateway,
        }
    }

    fn read_default_route(&self) -> (Option<String>, Option<String>) {
        let output = Command::new("ip")
            .args(["-j", "route", "show", "default"])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Ok(json) = serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
                    if let Some(route) = json.first() {
                        let gateway = route
                            .get("gateway")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let ip_address = self.get_wan_ip();
                        return (ip_address, gateway);
                    }
                }
            }
        }

        (None, None)
    }

    fn get_wan_ip(&self) -> Option<String> {
        let output = Command::new("ip")
            .args(["-j", "addr", "show"])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Ok(json) = serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
                    for iface in &json {
                        let ifname = iface.get("ifname").and_then(|v| v.as_str()).unwrap_or("");
                        if ifname.starts_with("eth") || ifname.starts_with("en") || ifname.starts_with("wlan") || ifname.starts_with("wl") {
                            if let Some(addr_info) = iface.get("addr_info").and_then(|v| v.as_array()) {
                                for addr in addr_info {
                                    if addr.get("family").and_then(|v| v.as_str()) == Some("inet") {
                                        if let Some(local) = addr.get("local").and_then(|v| v.as_str()) {
                                            return Some(local.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn read_connected_clients(&self) -> Vec<ConnectedClient> {
        let mut clients = Vec::new();

        let leases = self.read_dhcp_leases();
        clients.extend(leases);

        let arp_clients = self.read_arp_table();
        for arp in arp_clients {
            if !clients.iter().any(|c| c.ip == arp.ip) {
                clients.push(arp);
            }
        }

        clients
    }

    fn read_dhcp_leases(&self) -> Vec<ConnectedClient> {
        let mut clients = Vec::new();

        let paths = ["/var/lib/misc/dnsmasq.leases", "/tmp/dhcp.leases", "/var/lib/dhcp/dhcpd.leases"];

        for path in &paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let mac = parts[1].to_string();
                        let ip = parts[2].to_string();
                        let hostname = if parts[3] == "*" {
                            "unknown".to_string()
                        } else {
                            parts[3].to_string()
                        };

                        clients.push(ConnectedClient {
                            hostname,
                            mac,
                            ip,
                            interface: "wlan0".to_string(),
                            connected_since: chrono::Utc::now(),
                        });
                    }
                }
                break;
            }
        }

        clients
    }

    fn read_arp_table(&self) -> Vec<ConnectedClient> {
        let mut clients = Vec::new();

        let output = Command::new("ip").args(["-j", "neigh", "show"]).output();

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Ok(json) = serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
                    for entry in json {
                        let ip = entry.get("dst").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let mac = entry.get("lladdr").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let iface = entry.get("dev").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let state = entry.get("state").and_then(|v| v.as_array())
                            .and_then(|a| a.first())
                            .and_then(|v| v.as_str())
                            .unwrap_or("");

                        if !ip.is_empty() && !mac.is_empty() && mac != "00:00:00:00:00:00" {
                            let hostname = self.reverse_lookup(&ip).unwrap_or_else(|| "unknown".to_string());
                            clients.push(ConnectedClient {
                                hostname,
                                mac,
                                ip,
                                interface: iface,
                                connected_since: chrono::Utc::now(),
                            });
                        }
                    }
                }
            }
        }

        clients
    }

    fn reverse_lookup(&self, ip: &str) -> Option<String> {
        let output = Command::new("getent")
            .args(["hosts", ip])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let parts: Vec<&str> = stdout.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Some(parts[1].to_string());
                }
            }
        }

        None
    }
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        Self::new()
    }
}

fn _parse_ip(s: &str) -> Option<IpAddr> {
    s.parse().ok()
}

fn _build_iface_map(interfaces: &[InterfaceInfo]) -> HashMap<String, &InterfaceInfo> {
    interfaces.iter().map(|i| (i.name.clone(), i)).collect()
}
