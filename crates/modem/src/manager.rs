use anyhow::Result;
use pi_kiosk_core::ModemStatus;
use std::process::Command;

use crate::at::{detect_serial_ports, AtSerial, SmsMessage};

pub struct ModemManager {
    serial_port: Option<String>,
}

impl ModemManager {
    pub fn new() -> Self {
        Self { serial_port: None }
    }

    pub fn detect(&mut self) -> bool {
        if self.probe_modemmanager() {
            return true;
        }

        let ports = detect_serial_ports();
        for port in &ports {
            if self.probe_serial(port) {
                self.serial_port = Some(port.clone());
                return true;
            }
        }

        false
    }

    fn probe_modemmanager(&self) -> bool {
        Command::new("mmcli")
            .args(["-L"])
            .output()
            .map(|o| o.status.success() && !String::from_utf8_lossy(&o.stdout).trim().is_empty())
            .unwrap_or(false)
    }

    fn probe_serial(&self, port: &str) -> bool {
        let at = AtSerial::new(port);
        std::thread::scope(|s| {
            let handle = s.spawn(|| {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(_) => return false,
                };
                rt.block_on(async {
                    match at.send_command("AT").await {
                        Ok(resp) => resp.contains("OK"),
                        Err(_) => false,
                    }
                })
            });
            handle.join().unwrap_or(false)
        })
    }

    pub fn get_status(&self) -> ModemStatus {
        if let Some(port) = &self.serial_port {
            return self.get_status_via_serial(port);
        }

        self.get_status_via_modemmanager()
    }

    fn get_status_via_modemmanager(&self) -> ModemStatus {
        let list_output = Command::new("mmcli").args(["-L"]).output();

        let modem_index = match list_output {
            Ok(o) if o.status.success() => {
                let text = String::from_utf8_lossy(&o.stdout);
                text.lines()
                    .find_map(|line| {
                        if line.contains("Modem/") {
                            line.split('/')
                            .last()
                            .and_then(|s| s.trim().parse::<u32>().ok())
                        } else {
                            None
                        }
                    })
            }
            _ => None,
        };

        let Some(idx) = modem_index else {
            return ModemStatus {
                present: false,
                sim_ready: false,
                operator: None,
                signal_strength: None,
                access_technology: None,
                registered: false,
                connected: false,
            };
        };

        let info = Command::new("mmcli")
            .args(["-m", &idx.to_string()])
            .output();

        let Ok(info_out) = info else {
            return ModemStatus {
                present: true,
                sim_ready: false,
                operator: None,
                signal_strength: None,
                access_technology: None,
                registered: false,
                connected: false,
            };
        };

        let text = String::from_utf8_lossy(&info_out.stdout);

        let sim_ready = text.contains("SIM: connected") || text.contains("state: 'connected'");
        let operator = extract_field(&text, "operator name:");
        let signal = extract_field(&text, "signal quality:").and_then(|s| {
            s.trim().strip_suffix('%').and_then(|n| n.trim().parse::<i32>().ok()).map(|pct| {
                (pct as i32 - 113) + (pct as i32 * 113 / 100)
            })
        });
        let tech = extract_field(&text, "access tech:");
        let connected = text.contains("state: 'connected'");
        let registered = text.contains("state: 'registered'") || text.contains("state: 'connected'");

        ModemStatus {
            present: true,
            sim_ready,
            operator,
            signal_strength: signal,
            access_technology: tech,
            registered,
            connected,
        }
    }

    fn get_status_via_serial(&self, port: &str) -> ModemStatus {
        let at = AtSerial::new(port);

        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(_) => return ModemStatus::default(),
        };

        rt.block_on(async {
            let _ = at.send_command("ATE0").await;

            let sim_ready = match at.send_command("AT+CPIN?").await {
                Ok(r) => r.contains("READY"),
                Err(_) => false,
            };

            let operator = at.send_command("AT+COPS?").await
                .ok()
                .and_then(|r| {
                    r.lines()
                        .find_map(|l| {
                            if l.contains("+COPS:") {
                                let parts: Vec<&str> = l.split(',').collect();
                                parts.get(2).map(|s| s.trim_matches('"').to_string())
                            } else {
                                None
                            }
                        })
                });

            let signal = at.send_command("AT+CSQ").await
                .ok()
                .and_then(|r| {
                    r.lines().find_map(|l| {
                        if l.contains("+CSQ:") {
                            let parts: Vec<&str> = l.split(':').nth(1)?.split(',').collect();
                            let rssi = parts.first()?.trim().parse::<i32>().ok()?;
                            if rssi == 99 {
                                None
                            } else {
                                Some(-113 + rssi * 2)
                            }
                        } else {
                            None
                        }
                    })
                });

            let tech = at.send_command("AT+COPS?").await
                .ok()
                .and_then(|r| {
                    r.lines().find_map(|l| {
                        if l.contains("+COPS:") {
                            let parts: Vec<&str> = l.split(',').collect();
                            parts.get(3).and_then(|s| match s.trim() {
                                "0" => Some("GSM".to_string()),
                                "2" => Some("UTRAN".to_string()),
                                "7" => Some("LTE".to_string()),
                                _ => None,
                            })
                        } else {
                            None
                        }
                    })
                });

            let registered = at.send_command("AT+CREG?").await
                .map(|r| {
                    r.lines().any(|l| {
                        l.contains("+CREG:") && {
                            let parts: Vec<&str> = l.split(',').collect();
                            parts.get(1).map(|s| s.trim() == "1" || s.trim() == "5").unwrap_or(false)
                        }
                    })
                })
                .unwrap_or(false);

            let connected = at.send_command("AT+CGACT?").await
                .map(|r| r.lines().any(|l| l.contains(",1")))
                .unwrap_or(false);

            ModemStatus {
                present: true,
                sim_ready,
                operator,
                signal_strength: signal,
                access_technology: tech,
                registered,
                connected,
            }
        })
    }

    pub fn send_sms(&self, number: &str, text: &str) -> Result<()> {
        let port = self.serial_port.as_ref()
            .ok_or_else(|| anyhow::anyhow!("no serial port available"))?;

        let at = AtSerial::new(port);
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            at.send_sms(number, text).await
        })
    }

    pub fn read_sms(&self) -> Result<Vec<SmsMessage>> {
        let port = self.serial_port.as_ref()
            .ok_or_else(|| anyhow::anyhow!("no serial port available"))?;

        let at = AtSerial::new(port);
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            at.read_sms_list().await
        })
    }

    pub fn get_at_diagnostics(&self) -> Result<String> {
        let port = self.serial_port.as_ref()
            .ok_or_else(|| anyhow::anyhow!("no serial port available"))?;

        let at = AtSerial::new(port);
        let rt = tokio::runtime::Runtime::new()?;
        let result = rt.block_on(async {
            let mut output = String::new();

            let commands = [
                "ATI",
                "AT+CGMM",
                "AT+CGMR",
                "AT+CIMI",
                "AT+CPIN?",
                "AT+COPS?",
                "AT+CSQ",
                "AT+CREG?",
                "AT+CGREG?",
                "AT+CEREG?",
                "AT+CGACT?",
                "AT+CGDCONT?",
            ];

            for cmd in &commands {
                if let Ok(resp) = at.send_command(cmd).await {
                    output.push_str(&format!("{} ->\n{}\n\n", cmd, resp));
                }
            }

            Ok::<String, anyhow::Error>(output)
        })?;

        Ok(result)
    }
}

impl Default for ModemManager {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_field(text: &str, prefix: &str) -> Option<String> {
    text.lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if let Some(pos) = trimmed.find(prefix) {
                let value = trimmed[pos + prefix.len()..].trim();
                let clean = value.split_whitespace().next()?;
                if clean.is_empty() {
                    None
                } else {
                    Some(clean.to_string())
                }
            } else {
                None
            }
        })
}
