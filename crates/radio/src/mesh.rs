use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeshStatus {
    pub radio_present: bool,
    pub node_id: Option<String>,
    pub frequency_mhz: Option<f32>,
    pub tx_power_dbm: Option<i32>,
    pub nodes: Vec<MeshNode>,
    pub last_update: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeshNode {
    pub node_id: String,
    pub last_seen: Option<DateTime<Utc>>,
    pub rssi: Option<i32>,
    pub hops: u32,
}

pub struct MeshManager {
    port: Option<String>,
}

impl MeshManager {
    pub fn new() -> Self {
        Self { port: None }
    }

    pub fn detect(&mut self) -> bool {
        if let Some(port) = crate::detect_radio() {
            self.port = Some(port);
            true
        } else {
            false
        }
    }

    pub fn get_status(&self) -> MeshStatus {
        let present = self.port.is_some();
        MeshStatus {
            radio_present: present,
            ..Default::default()
        }
    }
}

impl Default for MeshManager {
    fn default() -> Self {
        Self::new()
    }
}
