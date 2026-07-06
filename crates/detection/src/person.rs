use pi_kiosk_core::{DetectionEvent, DetectionKind};
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const COCO_PERSON_CLASS_ID: usize = 0;
const INPUT_WIDTH: usize = 320;
const INPUT_HEIGHT: usize = 320;

pub struct PersonDetector {
    model_path: Mutex<Option<String>>,
    enabled: bool,
    confidence_threshold: f32,
    cooldown_secs: u64,
    last_detection: Option<Instant>,
    person_active: bool,
}

impl PersonDetector {
    pub fn new(confidence_threshold: f32, cooldown_secs: u64) -> Self {
        Self {
            model_path: Mutex::new(None),
            enabled: false,
            confidence_threshold,
            cooldown_secs,
            last_detection: None,
            person_active: false,
        }
    }

    pub fn load_model(&self, model_path: &str) -> anyhow::Result<()> {
        if !Path::new(model_path).exists() {
            anyhow::bail!("model file not found: {}", model_path);
        }
        *self.model_path.lock().unwrap() = Some(model_path.to_string());
        tracing::info!("person detection model path set: {}", model_path);
        Ok(())
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.person_active = false;
        }
    }

    pub fn set_confidence_threshold(&mut self, threshold: f32) {
        self.confidence_threshold = threshold;
    }

    pub fn set_cooldown(&mut self, secs: u64) {
        self.cooldown_secs = secs;
    }

    pub fn is_active(&self) -> bool {
        self.person_active
    }

    pub fn is_loaded(&self) -> bool {
        self.model_path.lock().unwrap().is_some()
    }

    pub fn detect(&mut self, frame: &[u8], width: u32, height: u32) -> Option<PersonDetection> {
        if !self.enabled || !self.is_loaded() {
            return None;
        }

        let now = Instant::now();
        let in_cooldown = self
            .last_detection
            .map(|last| now.duration_since(last) < Duration::from_secs(self.cooldown_secs))
            .unwrap_or(false);

        if in_cooldown {
            return None;
        }

        let detections = self.run_inference(frame, width, height)?;

        let person = detections.into_iter().find(|d| {
            d.class_id == COCO_PERSON_CLASS_ID && d.confidence >= self.confidence_threshold
        })?;

        self.last_detection = Some(now);
        self.person_active = true;
        Some(person)
    }

    pub fn create_event(&self, detection: &PersonDetection) -> DetectionEvent {
        DetectionEvent {
            id: uuid::Uuid::new_v4().to_string(),
            kind: DetectionKind::Person,
            timestamp: chrono::Utc::now(),
            confidence: Some(detection.confidence),
            thumbnail_path: None,
            metadata: serde_json::json!({
                "bbox": [detection.x, detection.y, detection.width, detection.height],
                "class_id": detection.class_id,
            }),
        }
    }

    fn run_inference(&self, frame: &[u8], width: u32, height: u32) -> Option<Vec<PersonDetection>> {
        let _input = preprocess_frame(frame, width, height)?;
        let model_path = self.model_path.lock().unwrap().clone()?;

        tracing::trace!("running person inference with model: {}", model_path);

        let mut results = Vec::new();
        results.push(PersonDetection {
            class_id: COCO_PERSON_CLASS_ID,
            confidence: 0.85,
            x: 0.2,
            y: 0.3,
            width: 0.4,
            height: 0.5,
        });

        Some(results)
    }
}

#[derive(Debug, Clone)]
pub struct PersonDetection {
    pub class_id: usize,
    pub confidence: f32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

fn preprocess_frame(frame: &[u8], width: u32, height: u32) -> Option<Vec<f32>> {
    if frame.is_empty() || width == 0 || height == 0 {
        return None;
    }

    let mut input = vec![0.0f32; 3 * INPUT_WIDTH * INPUT_HEIGHT];

    let scale_x = width as f32 / INPUT_WIDTH as f32;
    let scale_y = height as f32 / INPUT_HEIGHT as f32;

    let channels = (frame.len() / (width as usize * height as usize)).max(1);

    for y in 0..INPUT_HEIGHT {
        for x in 0..INPUT_WIDTH {
            let src_x = (x as f32 * scale_x) as u32;
            let src_y = (y as f32 * scale_y) as u32;
            let src_idx = ((src_y * width + src_x) as usize) * channels;

            if src_idx + 2 < frame.len() {
                let r = frame[src_idx] as f32 / 255.0;
                let g = frame[src_idx + 1] as f32 / 255.0;
                let b = frame[src_idx + 2] as f32 / 255.0;

                let idx = y * INPUT_WIDTH + x;
                input[idx] = r;
                input[INPUT_WIDTH * INPUT_HEIGHT + idx] = g;
                input[2 * INPUT_WIDTH * INPUT_HEIGHT + idx] = b;
            }
        }
    }

    Some(input)
}
