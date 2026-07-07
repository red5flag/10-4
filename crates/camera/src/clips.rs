use std::path::Path;
use chrono::Utc;

pub struct ClipManager {
    clip_dir: String,
    max_retention_days: u32,
    max_disk_usage_pct: u8,
}

impl ClipManager {
    pub fn new(clip_dir: &str, max_retention_days: u32, max_disk_usage_pct: u8) -> Self {
        Self {
            clip_dir: clip_dir.to_string(),
            max_retention_days,
            max_disk_usage_pct,
        }
    }

    pub fn list_clips(&self) -> Vec<ClipInfo> {
        let mut clips = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.clip_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("clip_") && (name.ends_with(".h264") || name.ends_with(".mp4")) {
                    let metadata = match entry.metadata() {
                        Ok(m) => m,
                        Err(_) => continue,
                    };
                    let modified = metadata.modified().ok().and_then(|t| {
                        t.duration_since(std::time::UNIX_EPOCH).ok()
                    }).map(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0).unwrap_or(Utc::now())).unwrap_or(Utc::now());
                    clips.push(ClipInfo {
                        filename: name,
                        size_bytes: metadata.len(),
                        timestamp: modified,
                    });
                }
            }
        }
        clips.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        clips
    }

    pub fn list_snapshots(&self) -> Vec<ClipInfo> {
        let mut snaps = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.clip_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("snap_") && name.ends_with(".jpg") {
                    let metadata = match entry.metadata() {
                        Ok(m) => m,
                        Err(_) => continue,
                    };
                    let modified = metadata.modified().ok().and_then(|t| {
                        t.duration_since(std::time::UNIX_EPOCH).ok()
                    }).map(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0).unwrap_or(Utc::now())).unwrap_or(Utc::now());
                    snaps.push(ClipInfo {
                        filename: name,
                        size_bytes: metadata.len(),
                        timestamp: modified,
                    });
                }
            }
        }
        snaps.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        snaps
    }

    pub fn enforce_retention(&self) -> usize {
        let mut deleted = 0;
        let cutoff = Utc::now() - chrono::Duration::days(self.max_retention_days as i64);

        let entries = match std::fs::read_dir(&self.clip_dir) {
            Ok(e) => e,
            Err(_) => return 0,
        };

        let mut all_clips: Vec<(std::path::PathBuf, String, std::time::SystemTime, u64)> = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            let is_clip = name.starts_with("clip_") || name.starts_with("snap_");
            if !is_clip {
                continue;
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let modified = metadata.modified().ok().and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH).ok()
            }).map(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0).unwrap_or(Utc::now())).unwrap_or(Utc::now());

            let mtime = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let size = metadata.len();

            all_clips.push((path, name, mtime, size));
        }

        // Phase 1: delete clips older than retention period
        for (path, name, _, _) in &all_clips {
            let modified = std::fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0).unwrap_or(Utc::now()))
                .unwrap_or(Utc::now());

            if modified < cutoff {
                if let Err(e) = std::fs::remove_file(path) {
                    tracing::warn!("failed to delete old clip {}: {e}", name);
                } else {
                    deleted += 1;
                }
            }
        }

        // Phase 2: enforce disk usage limit - delete oldest clips until under threshold
        let disk_usage_pct = self.get_disk_usage_pct();
        if disk_usage_pct >= self.max_disk_usage_pct as f64 {
            tracing::warn!(
                "disk usage {:.1}% exceeds limit {}%, pruning oldest clips",
                disk_usage_pct,
                self.max_disk_usage_pct
            );

            // Reload remaining clips sorted oldest-first
            let mut remaining: Vec<(std::path::PathBuf, String, std::time::SystemTime, u64)> = all_clips
                .into_iter()
                .filter(|(p, _, _, _)| p.exists())
                .collect();
            remaining.sort_by_key(|(_, _, mtime, _)| *mtime);

            for (path, name, _, _) in &remaining {
                if self.get_disk_usage_pct() < self.max_disk_usage_pct as f64 {
                    break;
                }
                if let Err(e) = std::fs::remove_file(path) {
                    tracing::warn!("failed to delete clip for disk space {}: {e}", name);
                } else {
                    deleted += 1;
                    tracing::info!("deleted clip {} to free disk space", name);
                }
            }
        }

        if deleted > 0 {
            tracing::info!("retention: deleted {} old clips/snapshots", deleted);
        }

        deleted
    }

    fn get_disk_usage_pct(&self) -> f64 {
        let output = std::process::Command::new("df")
            .args(["-P", &self.clip_dir])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let lines: Vec<&str> = stdout.lines().collect();
                if lines.len() >= 2 {
                    let parts: Vec<&str> = lines[1].split_whitespace().collect();
                    if parts.len() >= 5 {
                        return parts[4].trim_end_matches('%').parse::<f64>().unwrap_or(0.0);
                    }
                }
            }
            _ => {}
        }
        0.0
    }

    pub fn delete_clip(&self, filename: &str) -> anyhow::Result<()> {
        let path = Path::new(&self.clip_dir).join(filename);
        if !path.exists() {
            anyhow::bail!("clip not found: {}", filename);
        }
        std::fs::remove_file(&path)?;
        Ok(())
    }

    pub fn clip_path(&self, filename: &str) -> Option<String> {
        let path = Path::new(&self.clip_dir).join(filename);
        if path.exists() {
            Some(path.to_string_lossy().to_string())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClipInfo {
    pub filename: String,
    pub size_bytes: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
