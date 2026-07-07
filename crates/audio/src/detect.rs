use pi_kiosk_core::AudioStatus;
use std::process::Command;

pub fn detect_audio_device() -> AudioStatus {
    let device_name = find_alsa_device();
    let available = device_name.is_some();

    let volume_pct = if available {
        get_volume_pct()
    } else {
        None
    };

    let muted = if available {
        is_muted()
    } else {
        false
    };

    AudioStatus {
        device_available: available,
        device_name,
        mic_level: None,
        volume_pct,
        muted,
    }
}

fn find_alsa_device() -> Option<String> {
    let output = Command::new("aplay")
        .args(["-l"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.contains("card 0") && line.contains("USB Audio") {
            return Some("USB Audio Device".to_string());
        }
        if line.contains("card 0") {
            if let Some(idx) = line.find("card 0") {
                return Some(line[idx..].to_string());
            }
        }
    }

    if std::path::Path::new("/proc/asound/cards").exists() {
        if let Ok(content) = std::fs::read_to_string("/proc/asound/cards") {
            for line in content.lines() {
                if line.contains("USB") || line.contains("wm8960") || line.contains("HAT") {
                    return Some(line.trim().to_string());
                }
            }
            if content.lines().count() > 0 {
                return Some("bcm2835 Headphones".to_string());
            }
        }
    }

    None
}

fn get_volume_pct() -> Option<u8> {
    let output = Command::new("amixer")
        .args(["-M", "get", "PCM"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.contains('[') && line.contains('%') {
            let start = line.find('[')?;
            let end = line[start..].find('%')?;
            let pct_str = &line[start + 1..start + end];
            return pct_str.trim().parse::<u8>().ok();
        }
    }

    let output = Command::new("amixer")
        .args(["-M", "get", "Speaker"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.contains('[') && line.contains('%') {
            let start = line.find('[')?;
            let end = line[start..].find('%')?;
            let pct_str = &line[start + 1..start + end];
            return pct_str.trim().parse::<u8>().ok();
        }
    }

    None
}

fn is_muted() -> bool {
    let output = Command::new("amixer")
        .args(["-M", "get", "PCM"])
        .output();

    if let Ok(o) = output {
        let text = String::from_utf8_lossy(&o.stdout);
        return text.contains("[off]");
    }

    false
}
