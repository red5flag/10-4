use crate::types::{HealthCheckType, TorConfig, WanSource};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub device_name: String,
    pub listen_addr: String,
    pub db_path: String,
    pub camera: CameraConfig,
    pub detection: DetectionConfig,
    pub failover: FailoverConfig,
    pub storage: StorageConfig,
    pub network: NetworkConfig,
    pub cellular: CellularConfig,
    pub tor: TorConfig,
    pub tls: TlsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    pub device: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub night_vision_auto: bool,
    pub ir_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionConfig {
    pub motion_enabled: bool,
    pub person_enabled: bool,
    pub tamper_enabled: bool,
    pub capture_interval_ms: u32,
    pub motion_threshold: f32,
    pub person_confidence: f32,
    pub cooldown_seconds: u32,
    pub model_path: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FailoverConfig {
    pub enabled: bool,
    pub priority: Vec<WanSource>,
    pub health_check_interval_s: u32,
    pub health_check_target: String,
    pub health_check_type: HealthCheckType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub clip_dir: String,
    pub max_retention_days: u32,
    pub max_disk_usage_pct: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub wifi_mode: WifiMode,
    pub ssid: String,
    pub password: String,
    pub channel: u8,
    pub band: WifiBand,
    pub encryption: Encryption,
    pub country_code: String,
    pub hidden: bool,
    pub ap_interface: String,
    pub dhcp_range_start: String,
    pub dhcp_range_end: String,
    pub lease_time: String,
    pub dns_servers: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WifiMode {
    Ap,
    Client,
    Repeater,
    Hotspot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WifiBand {
    Band24G,
    Band5G,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Encryption {
    Open,
    Wpa2Psk,
    Wpa3Sae,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellularConfig {
    pub apn: String,
    pub interface: String,
    pub auto_connect: bool,
}

impl Default for CellularConfig {
    fn default() -> Self {
        Self {
            apn: "internet".into(),
            interface: "wwan0".into(),
            auto_connect: false,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            device_name: "pi-kiosk".into(),
            listen_addr: "127.0.0.1:3000".into(),
            db_path: "/var/lib/pi-kiosk/kiosk.db".into(),
            camera: CameraConfig {
                device: "/dev/video0".into(),
                width: 1280,
                height: 720,
                fps: 15,
                night_vision_auto: true,
                ir_enabled: false,
            },
            detection: DetectionConfig {
                motion_enabled: true,
                person_enabled: false,
                tamper_enabled: true,
                capture_interval_ms: 500,
                motion_threshold: 0.05,
                person_confidence: 0.5,
                cooldown_seconds: 30,
                model_path: None,
            },
            failover: FailoverConfig {
                enabled: true,
                priority: vec![WanSource::Ethernet, WanSource::Wifi, WanSource::Cellular],
                health_check_interval_s: 10,
                health_check_target: "1.1.1.1".into(),
                health_check_type: HealthCheckType::Ping,
            },
            storage: StorageConfig {
                clip_dir: "/var/lib/pi-kiosk/clips".into(),
                max_retention_days: 7,
                max_disk_usage_pct: 85,
            },
            network: NetworkConfig {
                wifi_mode: WifiMode::Ap,
                ssid: "pi-kiosk".into(),
                password: String::new(),
                channel: 6,
                band: WifiBand::Band24G,
                encryption: Encryption::Wpa2Psk,
                country_code: "US".into(),
                hidden: false,
                ap_interface: "wlan0".into(),
                dhcp_range_start: "10.0.0.100".into(),
                dhcp_range_end: "10.0.0.200".into(),
                lease_time: "12h".into(),
                dns_servers: vec!["8.8.8.8".into(), "1.1.1.1".into()],
            },
            tor: TorConfig {
                enabled: false,
                socks_port: 9050,
                control_port: 9051,
                use_bridges: false,
                bridge_lines: Vec::new(),
            },
            cellular: CellularConfig::default(),
            tls: TlsConfig::default(),
        }
    }
}
