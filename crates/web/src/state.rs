use pi_kiosk_core::{
    AppConfig, AudioStatus, CameraStatus, ConnectedClient, DashboardSnapshot, DetectionEvent,
    GpsStatus, HardwareInventory, MeshStatus, ModemStatus, SystemStats, WanSource, WanStatus,
};
use pi_kiosk_db::Database;
use std::sync::Arc;
use tokio::sync::{watch, RwLock};

pub struct AppState {
    pub config: RwLock<AppConfig>,
    pub db: Database,
    pub live: LiveState,
}

pub struct LiveState {
    pub system_stats: watch::Receiver<SystemStats>,
    pub wan_status: watch::Receiver<WanStatus>,
    pub camera_status: watch::Receiver<CameraStatus>,
    pub modem_status: watch::Receiver<ModemStatus>,
    pub gps_status: watch::Receiver<GpsStatus>,
    pub audio_status: watch::Receiver<AudioStatus>,
    pub mesh_status: watch::Receiver<MeshStatus>,
    pub hardware: watch::Receiver<HardwareInventory>,
    pub connected_clients: watch::Receiver<Vec<ConnectedClient>>,
    pub recent_events: watch::Receiver<Vec<DetectionEvent>>,
    pub recording: watch::Receiver<bool>,
    pub motion_active: watch::Receiver<bool>,
    pub person_detected: watch::Receiver<bool>,
}

impl AppState {
    pub fn new(config: AppConfig, db: Database, live: LiveState) -> Self {
        Self {
            config: RwLock::new(config),
            db,
            live,
        }
    }

    pub fn snapshot(&self) -> DashboardSnapshot {
        DashboardSnapshot {
            wan: self.live.wan_status.borrow().clone(),
            camera: self.live.camera_status.borrow().clone(),
            system: self.live.system_stats.borrow().clone(),
            modem: self.live.modem_status.borrow().clone(),
            connected_clients: self.live.connected_clients.borrow().clone(),
            recent_events: self.live.recent_events.borrow().clone(),
            recording: *self.live.recording.borrow(),
            motion_active: *self.live.motion_active.borrow(),
            person_detected: *self.live.person_detected.borrow(),
        }
    }

    pub async fn reload_settings(&self) -> anyhow::Result<()> {
        let loaded: Option<AppConfig> = self
            .db
            .with_writer(|conn| {
                pi_kiosk_db::settings::get::<AppConfig>(conn, "app_config")
            })
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        if let Some(cfg) = loaded {
            let mut config = self.config.write().await;
            *config = cfg;
            tracing::info!("settings reloaded from DB");
        }
        Ok(())
    }

    pub async fn save_settings(&self) -> anyhow::Result<()> {
        let config = self.config.read().await.clone();
        self.db
            .with_writer(|conn| {
                pi_kiosk_db::settings::set(conn, "app_config", &config)
            })
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        tracing::info!("settings saved to DB");
        Ok(())
    }
}

pub fn create_live_state() -> LiveState {
    let (system_stats_tx, system_stats_rx) = watch::channel(SystemStats {
        cpu_usage: 0.0,
        cpu_temp_c: 0.0,
        mem_used_mb: 0,
        mem_total_mb: 0,
        disk_used_gb: 0.0,
        disk_total_gb: 0.0,
        net_rx_bytes: 0,
        net_tx_bytes: 0,
    });
    let (wan_tx, wan_rx) = watch::channel(WanStatus {
        active_source: WanSource::Ethernet,
        online: false,
        ethernet_up: false,
        wifi_up: false,
        cellular_up: false,
        ip_address: None,
        gateway: None,
    });
    let (camera_tx, camera_rx) = watch::channel(CameraStatus {
        online: false,
        recording: false,
        night_vision: false,
        resolution: (1280, 720),
        fps: 15,
    });

    // Create camera controller with the camera status sender
    let (camera_controller, _recording_from_cam) =
        pi_kiosk_camera::CameraController::new(
            pi_kiosk_core::AppConfig::default().camera,
            camera_tx.clone(),
        );
    let (modem_tx, modem_rx) = watch::channel(ModemStatus {
        present: false,
        sim_ready: false,
        operator: None,
        signal_strength: None,
        access_technology: None,
        registered: false,
        connected: false,
    });
    let (gps_tx, gps_rx) = watch::channel(GpsStatus::default());
    let (audio_tx, audio_rx) = watch::channel(AudioStatus::default());
    let (mesh_tx, mesh_rx) = watch::channel(MeshStatus::default());
    let (hw_tx, hw_rx) = watch::channel(HardwareInventory::default());
    let (clients_tx, clients_rx) = watch::channel(Vec::new());
    let (events_tx, events_rx) = watch::channel(Vec::new());
    let (recording_tx, recording_rx) = watch::channel(false);
    let (motion_tx, motion_rx) = watch::channel(false);
    let (person_tx, person_rx) = watch::channel(false);

    // Store senders in a shared struct for workers
    let senders = Arc::new(LiveSenders {
        system_stats: system_stats_tx,
        wan: wan_tx,
        camera: camera_tx,
        modem: modem_tx,
        gps: gps_tx,
        audio: audio_tx,
        mesh: mesh_tx,
        hardware: hw_tx,
        connected_clients: clients_tx,
        recent_events: events_tx,
        recording: recording_tx,
        motion_active: motion_tx,
        person_detected: person_tx,
        camera_controller,
    });

    // Stash senders in a global for the SSE handler
    *LIVE_SENDERS.lock().unwrap() = Some(senders);

    LiveState {
        system_stats: system_stats_rx,
        wan_status: wan_rx,
        camera_status: camera_rx,
        modem_status: modem_rx,
        gps_status: gps_rx,
        audio_status: audio_rx,
        mesh_status: mesh_rx,
        hardware: hw_rx,
        connected_clients: clients_rx,
        recent_events: events_rx,
        recording: recording_rx,
        motion_active: motion_rx,
        person_detected: person_rx,
    }
}

pub struct LiveSenders {
    pub system_stats: watch::Sender<SystemStats>,
    pub wan: watch::Sender<WanStatus>,
    pub camera: watch::Sender<CameraStatus>,
    pub modem: watch::Sender<ModemStatus>,
    pub gps: watch::Sender<GpsStatus>,
    pub audio: watch::Sender<AudioStatus>,
    pub mesh: watch::Sender<MeshStatus>,
    pub hardware: watch::Sender<HardwareInventory>,
    pub connected_clients: watch::Sender<Vec<ConnectedClient>>,
    pub recent_events: watch::Sender<Vec<DetectionEvent>>,
    pub recording: watch::Sender<bool>,
    pub motion_active: watch::Sender<bool>,
    pub person_detected: watch::Sender<bool>,
    pub camera_controller: pi_kiosk_camera::CameraController,
}

static LIVE_SENDERS: std::sync::Mutex<Option<Arc<LiveSenders>>> = std::sync::Mutex::new(None);

pub fn get_live_senders() -> Option<Arc<LiveSenders>> {
    LIVE_SENDERS.lock().unwrap().clone()
}

pub async fn spawn_workers(state: Arc<AppState>) {
    // Spawn system stats worker
    tokio::spawn(system_stats_worker());

    // Spawn session cleanup worker
    tokio::spawn(session_cleanup_worker(state.clone()));

    // Spawn clip retention worker
    tokio::spawn(clip_retention_worker(state.clone()));

    // Spawn detection worker (motion + tamper)
    tokio::spawn(detection_worker(state.clone()));

    // Spawn network status worker
    tokio::spawn(network_worker());

    // Spawn modem status worker
    tokio::spawn(modem_worker());

    // Spawn GPS status worker
    tokio::spawn(gps_worker());

    // Spawn audio status worker
    tokio::spawn(audio_worker());

    // Spawn radio/mesh status worker
    tokio::spawn(radio_worker());

    // Spawn hardware detection worker
    tokio::spawn(hardware_worker());

    // Spawn failover worker
    tokio::spawn(failover_worker(state.clone()));

    // Spawn alert engine worker
    tokio::spawn(alert_engine_worker(state.clone()));

    tracing::info!("background workers started");
}

async fn session_cleanup_worker(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
    loop {
        interval.tick().await;
        if let Err(e) = state
            .db
            .with_writer(|conn| pi_kiosk_db::auth::cleanup_expired_sessions(conn))
            .await
        {
            tracing::warn!("session cleanup failed: {e}");
        }
    }
}

async fn clip_retention_worker(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
    loop {
        interval.tick().await;
        let config = state.config.read().await.clone();
        let mgr = pi_kiosk_camera::ClipManager::new(
            &config.storage.clip_dir,
            config.storage.max_retention_days,
            config.storage.max_disk_usage_pct,
        );
        let deleted = mgr.enforce_retention();
        if deleted > 0 {
            tracing::info!("clip retention: removed {} old files", deleted);
        }
    }
}

async fn detection_worker(state: Arc<AppState>) {
    let senders = match get_live_senders() {
        Some(s) => s,
        None => return,
    };

    let mut pipeline = pi_kiosk_detection::DetectionPipeline::new(
        pi_kiosk_camera::MotionConfig::default(),
        pi_kiosk_camera::TamperConfig::default(),
        0.5,
        30,
    );

    tracing::info!("detection worker started");

    loop {
        let config = state.config.read().await.clone();
        let interval_ms = config.detection.capture_interval_ms.max(100);

        pipeline.update_configs(
            pi_kiosk_camera::MotionConfig {
                enabled: config.detection.motion_enabled,
                threshold: config.detection.motion_threshold,
                cooldown_secs: config.detection.cooldown_seconds as u64,
                frame_width: 160,
                frame_height: 90,
            },
            pi_kiosk_camera::TamperConfig {
                enabled: config.detection.tamper_enabled,
                cooldown_secs: config.detection.cooldown_seconds as u64,
            },
            config.detection.person_enabled,
            config.detection.person_confidence,
            config.detection.cooldown_seconds as u64,
        );

        if config.detection.person_enabled && !pipeline.is_person_loaded() {
            if let Some(ref model_path) = config.detection.model_path {
                pipeline.set_model_path(Some(model_path.clone()));
                if let Err(e) = pipeline.try_load_model() {
                    tracing::warn!("failed to load person detection model: {e}");
                }
            }
        }

        let frame_result = tokio::process::Command::new("rpicam-still")
            .args([
                "--width", "160",
                "--height", "90",
                "--nopreview",
                "--encoding", "rgb",
                "-o", "-",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output()
            .await;

        let frame_available = frame_result.as_ref().map(|o| o.status.success()).unwrap_or(false);
        let frame_data = frame_result.ok().and_then(|o| if o.status.success() { Some(o.stdout) } else { None });

        let results = pipeline.process_frame(
            frame_data.as_deref().unwrap_or(&[]),
            160,
            90,
            frame_available,
        );

        if results.has_events() {
            let mut events = senders.recent_events.borrow().clone();
            for event in results.all_events() {
                if event.kind == pi_kiosk_core::DetectionKind::Tamper {
                    tracing::warn!("tamper detected: {:?}", results.tamper_kind);
                } else if event.kind == pi_kiosk_core::DetectionKind::Person {
                    tracing::info!("person detected: confidence={:?}", event.confidence);
                } else {
                    tracing::info!("motion detected: confidence={:?}", event.confidence);
                }
                log_detection_event(&state, event).await;
                events.insert(0, event.clone());
            }
            events.truncate(50);
            let _ = senders.recent_events.send(events);
        }

        let _ = senders.motion_active.send(results.motion_active);
        let _ = senders.person_detected.send(results.person_active);

        tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms as u64)).await;
    }
}

async fn log_detection_event(state: &AppState, event: &pi_kiosk_core::DetectionEvent) {
    if let Err(e) = state
        .db
        .with_writer(|conn| pi_kiosk_db::events::insert_event(conn, event))
        .await
    {
        tracing::warn!("failed to log detection event: {e}");
    }
}

async fn network_worker() {
    let senders = match get_live_senders() {
        Some(s) => s,
        None => return,
    };

    let monitor = pi_kiosk_network::NetworkMonitor::new();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

    tracing::info!("network status worker started");

    loop {
        interval.tick().await;
        let snapshot = monitor.snapshot();

        let _ = senders.wan.send(snapshot.wan.clone());
        let _ = senders.connected_clients.send(snapshot.connected_clients.clone());
    }
}

async fn modem_worker() {
    let senders = match get_live_senders() {
        Some(s) => s,
        None => return,
    };

    let mut manager = pi_kiosk_modem::ModemManager::new();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
    let mut detected = false;

    tracing::info!("modem status worker started");

    loop {
        interval.tick().await;

        if !detected {
            detected = manager.detect();
            if detected {
                tracing::info!("modem detected");
            }
        }

        if detected {
            let status = manager.get_status();
            let _ = senders.modem.send(status);
        }
    }
}

async fn gps_worker() {
    let senders = match get_live_senders() {
        Some(s) => s,
        None => return,
    };

    let mut reader: Option<pi_kiosk_gps::GpsReader> = None;
    let mut detect_failures: u32 = 0;
    let mut delay = tokio::time::Duration::from_secs(10);

    tracing::info!("GPS status worker started");

    loop {
        tokio::time::sleep(delay).await;

        if reader.is_none() {
            if let Some(port) = pi_kiosk_gps::GpsReader::detect_port() {
                tracing::info!("GPS port detected: {}", port);
                reader = Some(pi_kiosk_gps::GpsReader::new(&port));
                delay = tokio::time::Duration::from_secs(10);
            } else {
                detect_failures += 1;
                if detect_failures == 1 {
                    tracing::info!("no GPS device found, will retry periodically");
                }
                if detect_failures > 6 {
                    delay = tokio::time::Duration::from_secs(60);
                }
                continue;
            }
        }

        if let Some(r) = &reader {
            match r.read_status().await {
                Ok(gps_status) => {
                    let status = pi_kiosk_core::GpsStatus {
                        latitude: gps_status.fix.as_ref().map(|f| f.latitude),
                        longitude: gps_status.fix.as_ref().map(|f| f.longitude),
                        altitude: gps_status.fix.as_ref().and_then(|f| f.altitude),
                        speed_knots: gps_status.fix.as_ref().and_then(|f| f.speed_knots),
                        satellites_visible: gps_status.satellites_visible,
                        satellites_used: gps_status.satellites_used,
                        has_fix: gps_status.fix.is_some(),
                        last_update: gps_status.last_update,
                    };
                    let _ = senders.gps.send(status);
                }
                Err(e) => {
                    tracing::warn!("GPS read failed: {e}, resetting reader");
                    reader = None;
                    detect_failures = 0;
                    delay = tokio::time::Duration::from_secs(10);
                }
            }
        }
    }
}

async fn audio_worker() {
    let senders = match get_live_senders() {
        Some(s) => s,
        None => return,
    };

    let monitor = pi_kiosk_audio::AudioMonitor::new();
    let mut detect_failures: u32 = 0;
    let mut delay = tokio::time::Duration::from_secs(15);

    tracing::info!("audio status worker started");

    loop {
        tokio::time::sleep(delay).await;

        let status = monitor.snapshot();

        if !status.device_available {
            detect_failures += 1;
            if detect_failures == 1 {
                tracing::info!("no audio device found, will retry periodically");
            }
            if detect_failures > 4 {
                delay = tokio::time::Duration::from_secs(60);
            }
        } else {
            detect_failures = 0;
            delay = tokio::time::Duration::from_secs(15);
        }

        let _ = senders.audio.send(status);
    }
}

async fn radio_worker() {
    let senders = match get_live_senders() {
        Some(s) => s,
        None => return,
    };

    let mut manager = pi_kiosk_radio::MeshManager::new();
    let mut detected = false;
    let mut delay = tokio::time::Duration::from_secs(15);
    let mut detect_failures: u32 = 0;

    tracing::info!("radio/mesh status worker started");

    loop {
        tokio::time::sleep(delay).await;

        if !detected {
            detected = manager.detect();
            if detected {
                tracing::info!("radio device detected");
                delay = tokio::time::Duration::from_secs(15);
            } else {
                detect_failures += 1;
                if detect_failures == 1 {
                    tracing::info!("no radio device found, will retry periodically");
                }
                if detect_failures > 4 {
                    delay = tokio::time::Duration::from_secs(60);
                }
                continue;
            }
        }

        if detected {
            let mesh_status = manager.get_status();
            let status = pi_kiosk_core::MeshStatus {
                radio_present: mesh_status.radio_present,
                node_id: mesh_status.node_id,
                frequency_mhz: mesh_status.frequency_mhz,
                tx_power_dbm: mesh_status.tx_power_dbm,
                node_count: mesh_status.nodes.len() as u32,
                last_update: mesh_status.last_update,
            };
            let _ = senders.mesh.send(status);
        }
    }
}

async fn hardware_worker() {
    let senders = match get_live_senders() {
        Some(s) => s,
        None => return,
    };

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

    tracing::info!("hardware detection worker started");

    loop {
        interval.tick().await;
        let inventory = pi_kiosk_core::hardware::detect_hardware();
        let _ = senders.hardware.send(inventory);
    }
}

async fn system_stats_worker() {
    let mut prev_cpu_total: u64 = 0;
    let mut prev_cpu_idle: u64 = 0;
    let mut prev_net_rx: u64 = 0;
    let mut prev_net_tx: u64 = 0;

    // Get the sender from the global
    let senders = match get_live_senders() {
        Some(s) => s,
        None => return,
    };

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3));
    loop {
        interval.tick().await;

        let cpu_usage = read_cpu_usage(&mut prev_cpu_total, &mut prev_cpu_idle);
        let cpu_temp = read_cpu_temp();
        let (mem_used, mem_total) = read_meminfo();
        let (disk_used, disk_total) = read_disk_usage();
        let (net_rx, net_tx) = read_net_bytes();
        let net_rx_delta = net_rx.saturating_sub(prev_net_rx);
        let net_tx_delta = net_tx.saturating_sub(prev_net_tx);
        prev_net_rx = net_rx;
        prev_net_tx = net_tx;

        let stats = SystemStats {
            cpu_usage,
            cpu_temp_c: cpu_temp,
            mem_used_mb: mem_used,
            mem_total_mb: mem_total,
            disk_used_gb: disk_used,
            disk_total_gb: disk_total,
            net_rx_bytes: net_rx_delta,
            net_tx_bytes: net_tx_delta,
        };

        let _ = senders.system_stats.send(stats);
    }
}

fn read_cpu_usage(prev_total: &mut u64, prev_idle: &mut u64) -> f32 {
    let content = std::fs::read_to_string("/proc/stat").unwrap_or_default();
    let first_line = content.lines().next().unwrap_or("");
    let fields: Vec<u64> = first_line
        .split_whitespace()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    if fields.len() < 4 {
        return 0.0;
    }
    let idle = fields[3];
    let total: u64 = fields.iter().sum();
    let total_delta = total.saturating_sub(*prev_total);
    let idle_delta = idle.saturating_sub(*prev_idle);
    *prev_total = total;
    *prev_idle = idle;
    if total_delta == 0 {
        return 0.0;
    }
    let usage = 1.0 - (idle_delta as f32 / total_delta as f32);
    (usage * 100.0).round()
}

fn read_cpu_temp() -> f32 {
    std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
        .ok()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .map(|v| v / 1000.0)
        .unwrap_or(0.0)
}

fn read_meminfo() -> (u64, u64) {
    let content = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut mem_total = 0u64;
    let mut mem_available = 0u64;
    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            mem_total = line
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        } else if line.starts_with("MemAvailable:") {
            mem_available = line
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        }
    }
    let mem_used = mem_total.saturating_sub(mem_available);
    (mem_used / 1024, mem_total / 1024)
}

fn read_disk_usage() -> (f32, f32) {
    // Check the filesystem containing the clip dir
    let path = std::env::var("PI_KIOSK_DB")
        .unwrap_or_else(|_| "/var/lib/pi-kiosk/kiosk.db".to_string());
    let check_path = std::path::Path::new(&path)
        .parent()
        .unwrap_or(std::path::Path::new("/"))
        .to_str()
        .unwrap_or("/");
    let output = std::process::Command::new("df")
        .args(["-B1", "--output=used,size", check_path])
        .output();
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let line = stdout.lines().nth(1).unwrap_or("");
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let used: u64 = parts[0].parse().unwrap_or(0);
                let total: u64 = parts[1].parse().unwrap_or(0);
                (used as f32 / 1_073_741_824.0, total as f32 / 1_073_741_824.0)
            } else {
                (0.0, 0.0)
            }
        }
        Err(_) => (0.0, 0.0),
    }
}

fn read_net_bytes() -> (u64, u64) {
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;
    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str == "lo" {
                continue;
            }
            let base = format!("/sys/class/net/{}/statistics", name_str);
            if let Ok(rx) = std::fs::read_to_string(format!("{}/rx_bytes", base)) {
                total_rx += rx.trim().parse::<u64>().unwrap_or(0);
            }
            if let Ok(tx) = std::fs::read_to_string(format!("{}/tx_bytes", base)) {
                total_tx += tx.trim().parse::<u64>().unwrap_or(0);
            }
        }
    }
    (total_rx, total_tx)
}

async fn failover_worker(state: Arc<AppState>) {
    tracing::info!("failover worker started");

    loop {
        let config = state.config.read().await.clone();
        let failover = &config.failover;

        if !failover.enabled {
            tokio::time::sleep(tokio::time::Duration::from_secs(
                failover.health_check_interval_s as u64,
            ))
            .await;
            continue;
        }

        let senders = match get_live_senders() {
            Some(s) => s,
            None => {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                continue;
            }
        };

        let wan = senders.wan.borrow().clone();
        let modem = senders.modem.borrow().clone();

        let current_source = wan.active_source;
        let current_healthy = check_health(
            &failover.health_check_target,
            failover.health_check_type,
        )
        .await;

        if current_healthy {
            tokio::time::sleep(tokio::time::Duration::from_secs(
                failover.health_check_interval_s as u64,
            ))
            .await;
            continue;
        }

        tracing::warn!(
            "health check failed for {:?}, attempting failover",
            current_source
        );

        let mut switched = false;
        for &candidate in &failover.priority {
            if candidate == current_source {
                continue;
            }

            let candidate_up = match candidate {
                pi_kiosk_core::WanSource::Ethernet => wan.ethernet_up,
                pi_kiosk_core::WanSource::Wifi => wan.wifi_up,
                pi_kiosk_core::WanSource::Cellular => modem.present && modem.connected,
            };

            if !candidate_up {
                continue;
            }

            let candidate_healthy = check_health(
                &failover.health_check_target,
                failover.health_check_type,
            )
            .await;

            if candidate_healthy {
                let reason = format!(
                    "health check failed on {:?}, switched to {:?}",
                    current_source, candidate
                );

                let (iface, gateway) = match candidate {
                    pi_kiosk_core::WanSource::Ethernet => ("eth0".to_string(), wan.gateway.clone().unwrap_or_default()),
                    pi_kiosk_core::WanSource::Wifi => ("wlan0".to_string(), wan.gateway.clone().unwrap_or_default()),
                    pi_kiosk_core::WanSource::Cellular => (config.cellular.interface.clone(), String::new()),
                };

                if candidate == pi_kiosk_core::WanSource::Cellular {
                    let _ = crate::priv_client::send_request_ok(
                        &pi_kiosk_privileged::proto::PrivRequest::CellularConnect {
                            apn: config.cellular.apn.clone(),
                            interface: iface.clone(),
                        },
                    ).await;
                }

                let _ = crate::priv_client::send_request_ok(
                    &pi_kiosk_privileged::proto::PrivRequest::InterfaceUp {
                        interface: iface.clone(),
                    },
                ).await;

                if !gateway.is_empty() {
                    let _ = crate::priv_client::send_request_ok(
                        &pi_kiosk_privileged::proto::PrivRequest::SetDefaultRoute {
                            interface: iface,
                            gateway,
                        },
                    ).await;
                }

                if let Err(e) = state
                    .db
                    .with_writer(|conn| {
                        pi_kiosk_db::failover::insert_failover_event(
                            conn,
                            Some(current_source),
                            candidate,
                            &reason,
                        )
                    })
                    .await
                {
                    tracing::error!("failed to log failover event: {e}");
                }

                let new_wan = pi_kiosk_core::WanStatus {
                    active_source: candidate,
                    online: true,
                    ..wan
                };
                let _ = senders.wan.send(new_wan);

                tracing::info!("failover: switched to {:?}", candidate);
                switched = true;
                break;
            }
        }

        if !switched && !wan.online {
            tracing::error!("all WAN sources failed, no failover available");
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(
            failover.health_check_interval_s as u64,
        ))
        .await;
    }
}

async fn check_health(target: &str, check_type: pi_kiosk_core::HealthCheckType) -> bool {
    match check_type {
        pi_kiosk_core::HealthCheckType::Ping => check_ping(target).await,
        pi_kiosk_core::HealthCheckType::Dns => check_dns(target).await,
        pi_kiosk_core::HealthCheckType::Http => check_http(target).await,
    }
}

async fn check_ping(target: &str) -> bool {
    let result = tokio::process::Command::new("ping")
        .args(["-c", "1", "-W", "3", target])
        .output()
        .await;

    match result {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

async fn check_dns(target: &str) -> bool {
    let result = tokio::process::Command::new("nslookup")
        .args([target, "127.0.0.1"])
        .output()
        .await;

    match result {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

async fn check_http(target: &str) -> bool {
    let url = if target.starts_with("http") {
        target.to_string()
    } else {
        format!("http://{}/", target)
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build();

    match client {
        Ok(c) => c.get(&url).send().await.is_ok(),
        Err(_) => false,
    }
}

async fn alert_engine_worker(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(15));
    let mut last_event_count: usize = 0;

    loop {
        interval.tick().await;

        let rules = match state
            .db
            .with_writer(|conn| pi_kiosk_db::notifications::list_alert_rules(conn))
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("alert engine: failed to load rules: {}", e);
                continue;
            }
        };

        let enabled_rules: Vec<_> = rules.into_iter().filter(|r| r.enabled).collect();
        if enabled_rules.is_empty() {
            continue;
        }

        let events = state.live.recent_events.borrow().clone();
        let current_count = events.len();

        if current_count <= last_event_count {
            last_event_count = current_count;
            continue;
        }

        let new_events = if last_event_count == 0 {
            &events[..]
        } else {
            &events[last_event_count.min(events.len())..]
        };

        last_event_count = current_count;

        for event in new_events {
            let event_type = format!("detection:{:?}", event.kind).to_lowercase();

            for rule in &enabled_rules {
                if rule.event_type != event_type && rule.event_type != "*" {
                    continue;
                }

                let last_time = state
                    .db
                    .with_writer(|conn| {
                        pi_kiosk_db::notifications::last_alert_time(conn, &rule.id)
                    })
                    .await
                    .ok()
                    .flatten();

                if let Some(last) = last_time {
                    let elapsed = chrono::Utc::now().signed_duration_since(last);
                    if elapsed.num_seconds() < rule.cooldown_s as i64 {
                        continue;
                    }
                }

                let message = format!(
                    "Detection event: {:?} (confidence: {})",
                    event.kind,
                    event.confidence
                        .map(|c| format!("{:.0}%", c * 100.0))
                        .unwrap_or_else(|| "N/A".into()),
                );

                if let Err(e) =
                    crate::server_fns::notifications::dispatch_alert(&state, rule, &message).await
                {
                    tracing::warn!("alert dispatch failed: {}", e);
                }
            }
        }
    }
}
