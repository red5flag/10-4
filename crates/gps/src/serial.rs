use crate::nmea::{parse_nmea_line, GpsFix, GpsStatus, NmeaMessage};
use anyhow::Result;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio_serial::SerialPortBuilderExt;

const GPS_BAUD: u32 = 9_600;
const READ_TIMEOUT: Duration = Duration::from_secs(5);

pub struct GpsReader {
    port_path: String,
}

impl GpsReader {
    pub fn new(port_path: &str) -> Self {
        Self {
            port_path: port_path.to_string(),
        }
    }

    pub fn detect_port() -> Option<String> {
        let candidates = [
            "/dev/ttyAMA0",
            "/dev/serial0",
            "/dev/ttyACM0",
            "/dev/ttyUSB0",
            "/dev/ttyUSB1",
        ];

        for path in &candidates {
            if std::path::Path::new(path).exists() {
                return Some(path.to_string());
            }
        }

        if let Ok(entries) = std::fs::read_dir("/dev") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str == "ttyAMA0" || name_str == "serial0" {
                    let full = format!("/dev/{}", name_str);
                    return Some(full);
                }
            }
        }

        None
    }

    pub async fn read_status(&self) -> Result<GpsStatus> {
        let mut port = tokio_serial::new(&self.port_path, GPS_BAUD)
            .timeout(READ_TIMEOUT)
            .open_native_async()?;

        let mut buffer = Vec::new();
        let mut buf = [0u8; 256];
        let mut status = GpsStatus::default();
        let mut sats_visible: u32 = 0;
        let mut sats_used: u32 = 0;
        let deadline = tokio::time::Instant::now() + READ_TIMEOUT;

        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                break;
            }

            let read_fut = port.read(&mut buf);
            match tokio::time::timeout(remaining, read_fut).await {
                Ok(Ok(0)) => break,
                Ok(Ok(n)) => {
                    buffer.extend_from_slice(&buf[..n]);
                    while let Some(nl) = buffer.iter().position(|&b| b == b'\n') {
                        let line = String::from_utf8_lossy(&buffer[..nl]).trim().to_string();
                        buffer = buffer[nl + 1..].to_vec();

                        if let Some(msg) = parse_nmea_line(&line) {
                            match msg {
                                NmeaMessage::Gga(gga) => {
                                    if gga.fix_quality != crate::nmea::FixQuality::Invalid {
                                        sats_used = gga.satellite_count;
                                        if let (Some(lat), Some(lon)) = (gga.latitude, gga.longitude) {
                                            status.fix = Some(GpsFix {
                                                latitude: lat,
                                                longitude: lon,
                                                altitude: gga.altitude,
                                                fix_quality: gga.fix_quality,
                                                satellite_count: gga.satellite_count,
                                                timestamp: Some(chrono::Utc::now()),
                                                ..Default::default()
                                            });
                                            status.last_update = Some(chrono::Utc::now());
                                        }
                                    }
                                }
                                NmeaMessage::Rmc(rmc) => {
                                    if rmc.valid {
                                        if let (Some(lat), Some(lon)) = (rmc.latitude, rmc.longitude) {
                                            if status.fix.is_none() {
                                                status.fix = Some(GpsFix {
                                                    latitude: lat,
                                                    longitude: lon,
                                                    speed_knots: rmc.speed_knots,
                                                    course: rmc.course,
                                                    timestamp: Some(chrono::Utc::now()),
                                                    ..Default::default()
                                                });
                                                status.last_update = Some(chrono::Utc::now());
                                            } else if let Some(fix) = &mut status.fix {
                                                fix.speed_knots = rmc.speed_knots;
                                                fix.course = rmc.course;
                                            }
                                        }
                                    }
                                }
                                NmeaMessage::Gsv(gsv) => {
                                    sats_visible = gsv.total_sats;
                                }
                                NmeaMessage::Gsa(gsa) => {
                                    sats_used = gsa.sat_prns.len() as u32;
                                }
                            }
                        }
                    }
                }
                Ok(Err(_)) | Err(_) => break,
            }
        }

        status.satellites_visible = sats_visible;
        status.satellites_used = sats_used;
        Ok(status)
    }
}
