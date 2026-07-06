use std::process::Stdio;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::watch;

pub struct MjpegStream;

impl MjpegStream {
    pub fn spawn(
        config: pi_kiosk_core::CameraConfig,
    ) -> (impl futures::Stream<Item = Result<axum::body::Bytes, std::io::Error>>, watch::Receiver<bool>) {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let shutdown_rx_inner = shutdown_rx.clone();

        let stream = async_stream::stream! {
            let mut child = match Command::new("rpicam-vid")
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
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("failed to start rpicam-vid for MJPEG: {e}");
                    return;
                }
            };

            let stdout = child.stdout.take().unwrap();
            let mut reader = BufReader::new(stdout);
            let mut buf = vec![0u8; 64 * 1024];

            loop {
                if *shutdown_rx_inner.borrow() {
                    break;
                }

                match reader.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = axum::body::Bytes::copy_from_slice(&buf[..n]);
                        yield Ok(chunk);
                    }
                    Err(e) => {
                        tracing::warn!("mjpeg read error: {e}");
                        break;
                    }
                }
            }

            let _ = child.kill().await;
        };

        (stream, shutdown_rx)
    }
}
