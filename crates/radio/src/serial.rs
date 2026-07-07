use anyhow::Result;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::{SerialPort, SerialPortBuilderExt};

const RADIO_BAUD: u32 = 9_600;
const READ_TIMEOUT: Duration = Duration::from_secs(5);

pub struct RadioSerial {
    port_path: String,
}

impl RadioSerial {
    pub fn new(port_path: &str) -> Self {
        Self {
            port_path: port_path.to_string(),
        }
    }

    pub async fn send_command(&self, cmd: &str) -> Result<String> {
        let mut port = tokio_serial::new(&self.port_path, RADIO_BAUD)
            .timeout(READ_TIMEOUT)
            .open_native_async()?;

        port.write_data_terminal_ready(true).ok();
        port.write_request_to_send(true).ok();
        tokio::time::sleep(Duration::from_millis(50)).await;
        port.flush().await.ok();

        let full_cmd = format!("{}\r\n", cmd);
        port.write_all(full_cmd.as_bytes()).await?;
        port.flush().await?;

        let mut response = String::new();
        let mut buf = [0u8; 1024];
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
                    response.push_str(&String::from_utf8_lossy(&buf[..n]));
                    if response.contains("\r\n") {
                        break;
                    }
                }
                Ok(Err(_)) | Err(_) => break,
            }
        }

        Ok(response)
    }

    pub async fn send_packet(&self, data: &[u8]) -> Result<()> {
        let mut port = tokio_serial::new(&self.port_path, RADIO_BAUD)
            .timeout(READ_TIMEOUT)
            .open_native_async()?;

        port.write_all(data).await?;
        port.flush().await?;
        Ok(())
    }
}
