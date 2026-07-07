use std::path::Path;

pub fn detect_radio() -> Option<String> {
    let candidates = [
        "/dev/ttyAMA0",
        "/dev/serial0",
        "/dev/ttyUSB0",
        "/dev/ttyUSB1",
        "/dev/ttyACM0",
    ];

    for path in &candidates {
        if Path::new(path).exists() {
            return Some(path.to_string());
        }
    }

    if let Ok(entries) = std::fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("ttyUSB") || name_str.starts_with("ttyACM") {
                let full = format!("/dev/{}", name_str);
                if !candidates.contains(&full.as_str()) {
                    return Some(full);
                }
            }
        }
    }

    None
}
