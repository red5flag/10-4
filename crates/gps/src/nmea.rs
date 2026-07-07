use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GpsStatus {
    pub fix: Option<GpsFix>,
    pub satellites_visible: u32,
    pub satellites_used: u32,
    pub last_update: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GpsFix {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub speed_knots: Option<f64>,
    pub course: Option<f64>,
    pub fix_quality: FixQuality,
    pub satellite_count: u32,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FixQuality {
    #[default]
    Invalid,
    Gps,
    Dgps,
    Pps,
    Rtk,
    FloatRtk,
    Estimated,
    Manual,
    Simulation,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SatelliteInfo {
    pub prn: u32,
    pub elevation: Option<f32>,
    pub azimuth: Option<f32>,
    pub snr: Option<f32>,
    pub used: bool,
}

pub fn parse_nmea_line(line: &str) -> Option<NmeaMessage> {
    let line = line.trim();
    if !line.starts_with('$') {
        return None;
    }

    let checksum_idx = line.rfind('*')?;
    let _checksum = &line[checksum_idx + 1..];
    let body = &line[1..checksum_idx];

    let fields: Vec<&str> = body.split(',').collect();
    if fields.is_empty() {
        return None;
    }

    match fields[0] {
        "GPGGA" | "GNGGA" => parse_gga(&fields).map(NmeaMessage::Gga),
        "GPRMC" | "GNRMC" => parse_rmc(&fields).map(NmeaMessage::Rmc),
        "GPGSV" | "GNGSV" => parse_gsv(&fields).map(NmeaMessage::Gsv),
        "GPGSA" | "GNGSA" => parse_gsa(&fields).map(NmeaMessage::Gsa),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum NmeaMessage {
    Gga(GgaData),
    Rmc(RmcData),
    Gsv(GsvData),
    Gsa(GsaData),
}

#[derive(Debug, Clone, Default)]
pub struct GgaData {
    pub time: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub fix_quality: FixQuality,
    pub satellite_count: u32,
    pub altitude: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct RmcData {
    pub time: String,
    pub date: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub speed_knots: Option<f64>,
    pub course: Option<f64>,
    pub valid: bool,
}

#[derive(Debug, Clone, Default)]
pub struct GsvData {
    pub total_messages: u32,
    pub message_number: u32,
    pub total_sats: u32,
    pub sats: Vec<SatelliteInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct GsaData {
    pub mode: String,
    pub fix_type: String,
    pub sat_prns: Vec<u32>,
}

fn parse_lat_lon(value: &str, dir: &str) -> Option<f64> {
    if value.is_empty() {
        return None;
    }

    let dot = value.find('.')?;
    let deg_len = if dot <= 4 { 2 } else { 3 };
    let deg: f64 = value[..deg_len].parse().ok()?;
    let min: f64 = value[deg_len..].parse().ok()?;

    let mut result = deg + min / 60.0;
    if dir == "S" || dir == "W" {
        result = -result;
    }
    Some(result)
}

fn parse_gga(f: &[&str]) -> Option<GgaData> {
    let lat = if f.len() > 2 && f.len() > 3 {
        parse_lat_lon(f[2], f[3])
    } else {
        None
    };
    let lon = if f.len() > 4 && f.len() > 5 {
        parse_lat_lon(f[4], f[5])
    } else {
        None
    };
    let quality = match f.get(6).and_then(|s| s.parse::<u32>().ok()) {
        Some(0) => FixQuality::Invalid,
        Some(1) => FixQuality::Gps,
        Some(2) => FixQuality::Dgps,
        Some(3) => FixQuality::Pps,
        Some(4) => FixQuality::Rtk,
        Some(5) => FixQuality::FloatRtk,
        Some(6) => FixQuality::Estimated,
        Some(7) => FixQuality::Manual,
        Some(8) => FixQuality::Simulation,
        _ => FixQuality::Invalid,
    };
    let sats = f.get(7).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    let alt = f.get(9).and_then(|s| s.parse::<f64>().ok());

    Some(GgaData {
        time: f.get(1).map(|s| s.to_string()).unwrap_or_default(),
        latitude: lat,
        longitude: lon,
        fix_quality: quality,
        satellite_count: sats,
        altitude: alt,
    })
}

fn parse_rmc(f: &[&str]) -> Option<RmcData> {
    let valid = f.get(2).map(|s| *s == "A").unwrap_or(false);
    let lat = if f.len() > 3 && f.len() > 4 {
        parse_lat_lon(f[3], f[4])
    } else {
        None
    };
    let lon = if f.len() > 5 && f.len() > 6 {
        parse_lat_lon(f[5], f[6])
    } else {
        None
    };
    let speed = f.get(7).and_then(|s| s.parse::<f64>().ok());
    let course = f.get(8).and_then(|s| s.parse::<f64>().ok());

    Some(RmcData {
        time: f.get(1).map(|s| s.to_string()).unwrap_or_default(),
        date: f.get(9).map(|s| s.to_string()).unwrap_or_default(),
        latitude: lat,
        longitude: lon,
        speed_knots: speed,
        course,
        valid,
    })
}

fn parse_gsv(f: &[&str]) -> Option<GsvData> {
    let total_msgs = f.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    let msg_num = f.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    let total_sats = f.get(3).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);

    let mut sats = Vec::new();
    let mut idx = 4;
    while idx + 3 < f.len() {
        let prn = f[idx].parse::<u32>().ok();
        let elev = f[idx + 1].parse::<f32>().ok();
        let azim = f[idx + 2].parse::<f32>().ok();
        let snr = f[idx + 3].parse::<f32>().ok();
        if let Some(prn) = prn {
            sats.push(SatelliteInfo {
                prn,
                elevation: elev,
                azimuth: azim,
                snr,
                used: false,
            });
        }
        idx += 4;
    }

    Some(GsvData {
        total_messages: total_msgs,
        message_number: msg_num,
        total_sats: total_sats,
        sats,
    })
}

fn parse_gsa(f: &[&str]) -> Option<GsaData> {
    let mode = f.get(1).map(|s| s.to_string()).unwrap_or_default();
    let fix_type = f.get(2).map(|s| s.to_string()).unwrap_or_default();

    let mut prns = Vec::new();
    for i in 3..15 {
        if let Some(prn) = f.get(i).and_then(|s| s.parse::<u32>().ok()) {
            prns.push(prn);
        }
    }

    Some(GsaData {
        mode,
        fix_type,
        sat_prns: prns,
    })
}
