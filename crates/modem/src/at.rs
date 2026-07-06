use anyhow::Result;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio_serial::{SerialPortBuilderExt, SerialPort};

const SIM7600_BAUD: u32 = 115_200;
const AT_TIMEOUT: Duration = Duration::from_secs(3);

pub struct AtSerial {
    port_path: String,
}

impl AtSerial {
    pub fn new(port_path: &str) -> Self {
        Self {
            port_path: port_path.to_string(),
        }
    }

    pub async fn send_command(&self, cmd: &str) -> Result<String> {
        let mut port = tokio_serial::new(&self.port_path, SIM7600_BAUD)
            .timeout(AT_TIMEOUT)
            .open_native_async()?;

        port.write_data_terminal_ready(true).ok();
        port.write_request_to_send(true).ok();

        tokio::time::sleep(Duration::from_millis(100)).await;

        use tokio::io::AsyncWriteExt;
        port.flush().await.ok();

        let full_cmd = format!("{}\r\n", cmd);
        port.write_all(full_cmd.as_bytes()).await?;
        port.flush().await?;

        let mut response = String::new();
        let mut buf = [0u8; 1024];
        let deadline = tokio::time::Instant::now() + AT_TIMEOUT;

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
                    if response.contains("OK\r\n") || response.contains("ERROR\r\n") {
                        break;
                    }
                }
                Ok(Err(_)) | Err(_) => break,
            }
        }

        Ok(response)
    }

    pub async fn send_sms(&self, number: &str, text: &str) -> Result<()> {
        self.send_command("AT+CMGF=1").await?;
        self.send_command(&format!("AT+CMGS=\"{}\"", number)).await?;
        let ctrl_z = format!("{}\x1A", text);
        self.send_command(&ctrl_z).await?;
        Ok(())
    }

    pub async fn read_sms_list(&self) -> Result<Vec<SmsMessage>> {
        self.send_command("AT+CMGF=1").await?;
        let response = self.send_command("AT+CMGL=\"ALL\"").await?;

        let mut messages = Vec::new();
        let mut lines = response.lines().peekable();

        while let Some(line) = lines.next() {
            if line.starts_with("+CMGL:") {
                let parts: Vec<&str> = line.split(',').collect();
                let index = parts.get(0).and_then(|s| s.split(':').nth(1)).and_then(|s| s.trim().parse::<u32>().ok());
                let sender = parts.get(2).map(|s| s.trim_matches('"').to_string());

                let body = lines.next().map(|s| s.to_string()).unwrap_or_default();

                messages.push(SmsMessage {
                    index: index.unwrap_or(0),
                    sender: sender.unwrap_or_default(),
                    body,
                    timestamp: chrono::Utc::now(),
                });
            }
        }

        Ok(messages)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SmsMessage {
    pub index: u32,
    pub sender: String,
    pub body: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub fn detect_serial_ports() -> Vec<String> {
    let mut ports = Vec::new();

    let candidates = [
        "/dev/ttyUSB0",
        "/dev/ttyUSB1",
        "/dev/ttyUSB2",
        "/dev/ttyACM0",
        "/dev/ttyACM1",
        "/dev/ttyACM2",
        "/dev/ttyS0",
    ];

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            ports.push(path.to_string());
        }
    }

    if let Ok(entries) = std::fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("ttyUSB") || name_str.starts_with("ttyACM") {
                let full = format!("/dev/{}", name_str);
                if !ports.contains(&full) {
                    ports.push(full);
                }
            }
        }
    }

    ports
}
