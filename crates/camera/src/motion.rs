use pi_kiosk_core::{DetectionEvent, DetectionKind, TamperKind};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const FRAME_HISTORY: usize = 2;
const BRIGHTNESS_DARK_THRESHOLD: f32 = 0.08;
const BRIGHTNESS_OBSTRUCTION_THRESHOLD: f32 = 0.95;
const COOLDOWN_DEFAULT_SECS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionConfig {
    pub enabled: bool,
    pub threshold: f32,
    pub cooldown_secs: u64,
    pub frame_width: u32,
    pub frame_height: u32,
}

impl Default for MotionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 0.05,
            cooldown_secs: COOLDOWN_DEFAULT_SECS,
            frame_width: 160,
            frame_height: 90,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TamperConfig {
    pub enabled: bool,
    pub cooldown_secs: u64,
}

impl Default for TamperConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cooldown_secs: COOLDOWN_DEFAULT_SECS,
        }
    }
}

pub struct MotionDetector {
    config: MotionConfig,
    prev_frames: VecDeque<Vec<u8>>,
    last_motion: Option<Instant>,
    motion_active: bool,
}

impl MotionDetector {
    pub fn new(config: MotionConfig) -> Self {
        Self {
            config,
            prev_frames: VecDeque::with_capacity(FRAME_HISTORY),
            last_motion: None,
            motion_active: false,
        }
    }

    pub fn update_config(&mut self, config: MotionConfig) {
        self.config = config;
    }

    pub fn is_active(&self) -> bool {
        self.motion_active
    }

    pub fn process_frame(&mut self, frame: &[u8]) -> Option<f32> {
        if !self.config.enabled {
            self.motion_active = false;
            return None;
        }

        let grayscale = to_grayscale_downscaled(frame, self.config.frame_width, self.config.frame_height);

        if self.prev_frames.len() < FRAME_HISTORY {
            self.prev_frames.push_back(grayscale);
            return None;
        }

        let prev = self.prev_frames.front().unwrap();
        let diff = frame_difference(prev, &grayscale);

        self.prev_frames.pop_front();
        self.prev_frames.push_back(grayscale);

        if diff > self.config.threshold {
            let now = Instant::now();
            let should_fire = match self.last_motion {
                Some(last) => now.duration_since(last) > Duration::from_secs(self.config.cooldown_secs),
                None => true,
            };

            if should_fire {
                self.last_motion = Some(now);
                self.motion_active = true;
                return Some(diff);
            }
        }

        if let Some(last) = self.last_motion {
            if Instant::now().duration_since(last) > Duration::from_secs(self.config.cooldown_secs) {
                self.motion_active = false;
            }
        }

        None
    }

    pub fn create_event(&self, confidence: f32, thumbnail: Option<String>) -> DetectionEvent {
        DetectionEvent {
            id: uuid::Uuid::new_v4().to_string(),
            kind: DetectionKind::Motion,
            timestamp: chrono::Utc::now(),
            confidence: Some(confidence),
            thumbnail_path: thumbnail,
            metadata: serde_json::json!({
                "threshold": self.config.threshold,
                "resolution": [self.config.frame_width, self.config.frame_height],
            }),
        }
    }
}

pub struct TamperDetector {
    config: TamperConfig,
    last_tamper: Option<Instant>,
    feed_loss_count: u32,
    last_brightness: f32,
}

impl TamperDetector {
    pub fn new(config: TamperConfig) -> Self {
        Self {
            config,
            last_tamper: None,
            feed_loss_count: 0,
            last_brightness: 0.5,
        }
    }

    pub fn update_config(&mut self, config: TamperConfig) {
        self.config = config;
    }

    pub fn check_brightness(&mut self, frame: &[u8]) -> Option<TamperKind> {
        if !self.config.enabled || frame.is_empty() {
            return None;
        }

        let brightness = average_brightness(frame);
        self.last_brightness = brightness;

        let now = Instant::now();
        let in_cooldown = self
            .last_tamper
            .map(|last| now.duration_since(last) < Duration::from_secs(self.config.cooldown_secs))
            .unwrap_or(false);

        if in_cooldown {
            return None;
        }

        if brightness < BRIGHTNESS_DARK_THRESHOLD {
            self.last_tamper = Some(now);
            return Some(TamperKind::SuddenDarkness);
        }

        if brightness > BRIGHTNESS_OBSTRUCTION_THRESHOLD {
            self.last_tamper = Some(now);
            return Some(TamperKind::Obstruction);
        }

        None
    }

    pub fn check_feed_loss(&mut self, frame_available: bool) -> Option<TamperKind> {
        if !self.config.enabled {
            return None;
        }

        if !frame_available {
            self.feed_loss_count += 1;
            if self.feed_loss_count >= 5 {
                self.feed_loss_count = 0;
                let now = Instant::now();
                let in_cooldown = self
                    .last_tamper
                    .map(|last| now.duration_since(last) < Duration::from_secs(self.config.cooldown_secs))
                    .unwrap_or(false);
                if !in_cooldown {
                    self.last_tamper = Some(now);
                    return Some(TamperKind::FeedLoss);
                }
            }
        } else {
            self.feed_loss_count = 0;
        }

        None
    }

    pub fn create_event(&self, kind: TamperKind) -> DetectionEvent {
        DetectionEvent {
            id: uuid::Uuid::new_v4().to_string(),
            kind: DetectionKind::Tamper,
            timestamp: chrono::Utc::now(),
            confidence: None,
            thumbnail_path: None,
            metadata: serde_json::json!({
                "tamper_kind": format!("{:?}", kind),
                "brightness": self.last_brightness,
            }),
        }
    }

    pub fn last_brightness(&self) -> f32 {
        self.last_brightness
    }
}

fn to_grayscale_downscaled(frame: &[u8], target_w: u32, target_h: u32) -> Vec<u8> {
    let mut result = vec![0u8; (target_w * target_h) as usize];
    let src_len = frame.len();

    if src_len == 0 {
        return result;
    }

    let src_pixels = src_len / 3;
    if src_pixels == 0 {
        return result;
    }

    let src_w = (src_pixels as f32).sqrt() as usize;
    if src_w == 0 {
        return result;
    }
    let src_h = src_pixels / src_w;
    let src_w = src_w as u32;
    let src_h = src_h as u32;

    for y in 0..target_h {
        for x in 0..target_w {
            let sx = (x * src_w) / target_w.max(1);
            let sy = (y * src_h) / target_h.max(1);
            let src_idx = ((sy * src_w + sx) as usize) * 3;
            if src_idx + 2 < src_len {
                let gray = (frame[src_idx] as u16 + frame[src_idx + 1] as u16 + frame[src_idx + 2] as u16) / 3;
                result[(y * target_w + x) as usize] = gray as u8;
            }
        }
    }

    result
}

fn frame_difference(a: &[u8], b: &[u8]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut total_diff: u64 = 0;
    for (pa, pb) in a.iter().zip(b.iter()) {
        let diff = (*pa as i16 - *pb as i16).unsigned_abs() as u64;
        total_diff += diff;
    }

    let max_diff = a.len() as u64 * 255;
    total_diff as f32 / max_diff as f32
}

fn average_brightness(frame: &[u8]) -> f32 {
    if frame.is_empty() {
        return 0.0;
    }

    let mut total: u64 = 0;
    for chunk in frame.chunks(3) {
        if chunk.len() >= 3 {
            total += (chunk[0] as u64 + chunk[1] as u64 + chunk[2] as u64) / 3;
        }
    }

    let pixel_count = frame.len() / 3;
    if pixel_count == 0 {
        return 0.0;
    }

    (total as f32 / pixel_count as f32) / 255.0
}
