use pi_kiosk_core::AudioStatus;
use std::process::Command;

pub struct AudioMonitor;

impl AudioMonitor {
    pub fn new() -> Self {
        Self
    }

    pub fn snapshot(&self) -> AudioStatus {
        let mut status = crate::detect_audio_device();
        if status.device_available {
            status.mic_level = self.read_mic_level();
        }
        status
    }

    fn read_mic_level(&self) -> Option<f32> {
        let output = Command::new("arecord")
            .args(["-l"])
            .output()
            .ok()?;

        let text = String::from_utf8_lossy(&output.stdout);
        if !text.contains("card") {
            return None;
        }

        let record_result = std::thread::scope(|s| {
            s.spawn(|| {
                Command::new("arecord")
                    .args(["-d", "1", "-f", "S16_LE", "-r", "8000", "-c", "1", "/dev/null"])
                    .output()
            })
            .join()
            .ok()
        });

        if record_result.is_none() {
            return None;
        }

        Some(0.0)
    }
}

impl Default for AudioMonitor {
    fn default() -> Self {
        Self::new()
    }
}
