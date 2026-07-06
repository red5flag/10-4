use serde::{Deserialize, Serialize};

/// Typed requests accepted by the privileged helper.
/// The helper NEVER accepts arbitrary commands — only these variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum PrivRequest {
    Ping,
    HostapdStart { config: HostapdConfig },
    HostapdStop,
    DnsmasqStart { config: DnsmasqConfig },
    DnsmasqStop,
    NftablesApply { rules: Vec<NftRule> },
    WireguardUp { config_path: String },
    WireguardDown { interface: String },
    OpenvpnUp { config_path: String },
    OpenvpnDown { interface: String },
    TorStart { config_path: String },
    TorStop,
    KillSwitchEnable { interface: String },
    KillSwitchDisable,
    InterfaceUp { interface: String },
    InterfaceDown { interface: String },
    SetDefaultRoute { interface: String, gateway: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostapdConfig {
    pub interface: String,
    pub ssid: String,
    pub password: String,
    pub channel: u8,
    pub band: WifiBand,
    pub encryption: Encryption,
    pub country_code: String,
    pub hidden: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WifiBand {
    Band24G,
    Band5G,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Encryption {
    Wpa2Psk,
    Wpa3Sae,
    Open,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsmasqConfig {
    pub interface: String,
    pub dhcp_range_start: String,
    pub dhcp_range_end: String,
    pub lease_time: String,
    pub dns_servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftRule {
    pub table: String,
    pub chain: String,
    pub rule: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivResponse {
    Ok,
    Pong,
    Error(String),
}
