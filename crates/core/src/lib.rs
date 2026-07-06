pub mod config;
pub mod error;
pub mod types;

pub use config::{
    AppConfig, CameraConfig, DetectionConfig, Encryption, FailoverConfig, NetworkConfig,
    StorageConfig, WifiBand, WifiMode,
};
pub use error::{Error, Result};
pub use types::*;
