use crate::HardwareInventory;
use std::path::Path;

pub fn detect_hardware() -> HardwareInventory {
    let mut inv = HardwareInventory::default();

    inv.modem_present = detect_modem();
    inv.gps_present = detect_gps();
    inv.audio_present = detect_audio();
    inv.poe_hat_present = detect_poe_hat();
    inv.camera_present = detect_camera();
    inv.radio_present = detect_radio();
    inv.serial_ports = list_serial_ports();
    inv.i2c_devices = list_i2c_devices();

    inv
}

fn detect_modem() -> bool {
    let candidates = ["/dev/ttyUSB0", "/dev/ttyUSB1", "/dev/ttyUSB2"];
    for p in &candidates {
        if Path::new(p).exists() {
            return true;
        }
    }

    if let Ok(entries) = std::fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("ttyUSB") || name_str.starts_with("wwan") {
                return true;
            }
        }
    }

    false
}

fn detect_gps() -> bool {
    let candidates = ["/dev/ttyAMA0", "/dev/serial0", "/dev/ttyACM0"];
    for p in &candidates {
        if Path::new(p).exists() {
            return true;
        }
    }
    false
}

fn detect_audio() -> bool {
    Path::new("/proc/asound/cards").exists()
        && std::fs::read_to_string("/proc/asound/cards")
            .map(|c| !c.trim().is_empty())
            .unwrap_or(false)
}

fn detect_poe_hat() -> bool {
    if let Ok(entries) = std::fs::read_dir("/sys/class/i2c-adapter") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(sub) = std::fs::read_dir(&path) {
                for s in sub.flatten() {
                    let name = s.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.contains("0048") || name_str.contains("poehat") {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn detect_camera() -> bool {
    Path::new("/dev/video0").exists()
}

fn detect_radio() -> bool {
    if let Ok(entries) = std::fs::read_dir("/sys/class/spidev") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("spidev") {
                return true;
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str == "serial0" || name_str == "ttyAMA0" {
                return true;
            }
        }
    }

    false
}

fn list_serial_ports() -> Vec<String> {
    let mut ports = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("ttyUSB")
                || name_str.starts_with("ttyACM")
                || name_str.starts_with("ttyAMA")
                || name_str == "serial0"
            {
                ports.push(format!("/dev/{}", name_str));
            }
        }
    }
    ports.sort();
    ports
}

fn list_i2c_devices() -> Vec<String> {
    let mut devices = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/sys/class/i2c-adapter") {
        for entry in entries.flatten() {
            let bus_name = entry.file_name();
            let bus_path = entry.path();
            if let Ok(sub) = std::fs::read_dir(&bus_path) {
                for s in sub.flatten() {
                    let name = s.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with('-') {
                        let addr = name_str.split('-').nth(1).unwrap_or("");
                        devices.push(format!("{} @ 0x{}", bus_name.to_string_lossy(), addr));
                    }
                }
            }
        }
    }
    devices.sort();
    devices
}
