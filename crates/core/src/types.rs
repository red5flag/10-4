use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WanSource {
    Ethernet,
    Wifi,
    Cellular,
}

impl std::fmt::Display for WanSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WanSource::Ethernet => write!(f, "ethernet"),
            WanSource::Wifi => write!(f, "wifi"),
            WanSource::Cellular => write!(f, "cellular"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DetectionKind {
    Motion,
    Person,
    Tamper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TamperKind {
    SuddenDarkness,
    Obstruction,
    CameraMoved,
    FeedLoss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionEvent {
    pub id: String,
    pub kind: DetectionKind,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub confidence: Option<f32>,
    pub thumbnail_path: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub cpu_temp_c: f32,
    pub mem_used_mb: u64,
    pub mem_total_mb: u64,
    pub disk_used_gb: f32,
    pub disk_total_gb: f32,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraStatus {
    pub online: bool,
    pub recording: bool,
    pub night_vision: bool,
    pub resolution: (u32, u32),
    pub fps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WanStatus {
    pub active_source: WanSource,
    pub online: bool,
    pub ethernet_up: bool,
    pub wifi_up: bool,
    pub cellular_up: bool,
    pub ip_address: Option<String>,
    pub gateway: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModemStatus {
    pub present: bool,
    pub sim_ready: bool,
    pub operator: Option<String>,
    pub signal_strength: Option<i32>,
    pub access_technology: Option<String>,
    pub registered: bool,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedClient {
    pub hostname: String,
    pub mac: String,
    pub ip: String,
    pub interface: String,
    pub connected_since: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub wan: WanStatus,
    pub camera: CameraStatus,
    pub system: SystemStats,
    pub modem: ModemStatus,
    pub connected_clients: Vec<ConnectedClient>,
    pub recent_events: Vec<DetectionEvent>,
    pub recording: bool,
    pub motion_active: bool,
    pub person_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverEvent {
    pub id: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub from_source: Option<WanSource>,
    pub to_source: WanSource,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthCheckType {
    #[default]
    Ping,
    Dns,
    Http,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VpnType {
    Wireguard,
    Openvpn,
}

impl std::fmt::Display for VpnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VpnType::Wireguard => write!(f, "wireguard"),
            VpnType::Openvpn => write!(f, "openvpn"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VpnMode {
    Client,
    Server,
}

impl std::fmt::Display for VpnMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VpnMode::Client => write!(f, "client"),
            VpnMode::Server => write!(f, "server"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnConfig {
    pub id: String,
    pub name: String,
    pub vpn_type: VpnType,
    pub mode: VpnMode,
    pub config_path: String,
    pub enabled: bool,
    pub kill_switch: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VpnStatus {
    pub wireguard_active: bool,
    pub wireguard_interface: Option<String>,
    pub openvpn_active: bool,
    pub tor_active: bool,
    pub tor_socks_port: Option<u16>,
    pub kill_switch_active: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TorConfig {
    pub enabled: bool,
    pub socks_port: u16,
    pub control_port: u16,
    pub use_bridges: bool,
    pub bridge_lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireguardPeer {
    pub public_key: String,
    pub allowed_ips: String,
    pub endpoint: Option<String>,
    pub preshared_key: Option<String>,
    pub persistent_keepalive: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireguardConfig {
    pub interface_name: String,
    pub private_key: String,
    pub address: String,
    pub listen_port: Option<u16>,
    pub dns: Vec<String>,
    pub peers: Vec<WireguardPeer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertAction {
    Ui,
    Sms,
    Webhook,
}

impl std::fmt::Display for AlertAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertAction::Ui => write!(f, "ui"),
            AlertAction::Sms => write!(f, "sms"),
            AlertAction::Webhook => write!(f, "webhook"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub event_type: String,
    pub action: AlertAction,
    pub action_target: Option<String>,
    pub cooldown_s: u32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertLogEntry {
    pub id: i64,
    pub rule_id: Option<String>,
    pub event_type: String,
    pub action: String,
    pub status: String,
    pub message: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
