use pi_kiosk_core::{CameraConfig, CameraStatus};
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::{watch, Mutex};

pub struct CameraController {
    config: Arc<Mutex<CameraConfig>>,
    preview_proc: Mutex<Option<Child>>,
    record_proc: Mutex<Option<Child>>,
    status_tx: watch::Sender<CameraStatus>,
    recording_tx: watch::Sender<bool>,
}

impl CameraController {
    pub fn new(config: CameraConfig, status_tx: watch::Sender<CameraStatus>) -> (Self, watch::Receiver<bool>) {
        let (recording_tx, recording_rx) = watch::channel(false);
        let ctrl = Self {
            config: Arc::new(Mutex::new(config)),
            preview_proc: Mutex::new(None),
            record_proc: Mutex::new(None),
            status_tx,
            recording_tx,
        };
        (ctrl, recording_rx)
    }

    pub async fn update_status(&self) {
        let config = self.config.lock().await;
        let recording = *self.recording_tx.borrow();
        let has_preview = self.preview_proc.lock().await.is_some();
        let status = CameraStatus {
            online: has_preview || recording,
            recording,
            night_vision: config.ir_enabled,
            resolution: (config.width, config.height),
            fps: config.fps,
        };
        let _ = self.status_tx.send(status);
    }

    pub async fn start_preview(&self) -> anyhow::Result<()> {
        let config = self.config.lock().await;
        let mut proc = self.preview_proc.lock().await;
        if proc.is_some() {
            return Ok(());
        }

        let child = Command::new("rpicam-vid")
            .args([
                "--inline",
                "--codec", "mjpeg",
                "--width", &config.width.to_string(),
                "--height", &config.height.to_string(),
                "--framerate", &config.fps.to_string(),
                "--timeout", "0",
                "--nopreview",
                "-o", "-",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        *proc = Some(child);
        tracing::info!("camera preview started ({}x{}@{}fps)", config.width, config.height, config.fps);
        drop(proc);
        drop(config);
        self.update_status().await;
        Ok(())
    }

    pub async fn stop_preview(&self) -> anyhow::Result<()> {
        let mut proc = self.preview_proc.lock().await;
        if let Some(mut child) = proc.take() {
            let _ = child.kill().await;
            tracing::info!("camera preview stopped");
        }
        drop(proc);
        self.update_status().await;
        Ok(())
    }

    pub async fn start_recording(&self, clip_dir: &str) -> anyhow::Result<String> {
        let config = self.config.lock().await;
        let mut proc = self.record_proc.lock().await;
        if proc.is_some() {
            anyhow::bail!("already recording");
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("{}/clip_{}.h264", clip_dir, timestamp);

        std::fs::create_dir_all(clip_dir)?;

        let child = Command::new("rpicam-vid")
            .args([
                "--codec", "h264",
                "--width", &config.width.to_string(),
                "--height", &config.height.to_string(),
                "--framerate", &config.fps.to_string(),
                "--timeout", "0",
                "--nopreview",
                "--inline",
                "-o", &filename,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        *proc = Some(child);
        let _ = self.recording_tx.send(true);
        tracing::info!("recording started: {}", filename);
        drop(proc);
        drop(config);
        self.update_status().await;
        Ok(filename)
    }

    pub async fn stop_recording(&self) -> anyhow::Result<()> {
        let mut proc = self.record_proc.lock().await;
        if let Some(mut child) = proc.take() {
            let _ = child.kill().await;
            tracing::info!("recording stopped");
        }
        let _ = self.recording_tx.send(false);
        drop(proc);
        self.update_status().await;
        Ok(())
    }

    pub async fn snapshot(&self, clip_dir: &str) -> anyhow::Result<String> {
        let config = self.config.lock().await;
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("{}/snap_{}.jpg", clip_dir, timestamp);

        std::fs::create_dir_all(clip_dir)?;

        let status = Command::new("rpicam-still")
            .args([
                "--width", &config.width.to_string(),
                "--height", &config.height.to_string(),
                "--nopreview",
                "-o", &filename,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await?;

        if !status.success() {
            anyhow::bail!("rpicam-still failed with status {:?}", status);
        }

        tracing::info!("snapshot saved: {}", filename);
        Ok(filename)
    }

    pub async fn set_night_vision(&self, enabled: bool) -> anyhow::Result<()> {
        let mut config = self.config.lock().await;
        config.ir_enabled = enabled;
        tracing::info!("night vision: {}", if enabled { "on" } else { "off" });
        drop(config);
        self.update_status().await;
        Ok(())
    }

    pub async fn is_recording(&self) -> bool {
        *self.recording_tx.borrow()
    }

    pub async fn shutdown(&self) {
        let _ = self.stop_preview().await;
        let _ = self.stop_recording().await;
    }
}
