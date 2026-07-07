pub mod nmea;
pub mod serial;

pub use nmea::{GpsFix, GpsStatus, SatelliteInfo};
pub use serial::GpsReader;
