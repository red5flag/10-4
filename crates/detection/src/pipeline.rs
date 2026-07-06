use pi_kiosk_camera::{MotionDetector, MotionConfig, TamperConfig, TamperDetector};
use pi_kiosk_core::{DetectionEvent, TamperKind};

use crate::person::PersonDetector;

pub struct DetectionPipeline {
    pub motion: MotionDetector,
    pub tamper: TamperDetector,
    pub person: PersonDetector,
    person_enabled: bool,
    model_path: Option<String>,
}

impl DetectionPipeline {
    pub fn new(
        motion_config: MotionConfig,
        tamper_config: TamperConfig,
        person_confidence: f32,
        person_cooldown_secs: u64,
    ) -> Self {
        Self {
            motion: MotionDetector::new(motion_config),
            tamper: TamperDetector::new(tamper_config),
            person: PersonDetector::new(person_confidence, person_cooldown_secs),
            person_enabled: false,
            model_path: None,
        }
    }

    pub fn set_person_enabled(&mut self, enabled: bool) {
        self.person_enabled = enabled;
        self.person.set_enabled(enabled);
    }

    pub fn set_model_path(&mut self, path: Option<String>) {
        self.model_path = path;
    }

    pub fn try_load_model(&self) -> anyhow::Result<()> {
        if let Some(ref path) = self.model_path {
            self.person.load_model(path)
        } else {
            anyhow::bail!("no model path configured")
        }
    }

    pub fn is_person_loaded(&self) -> bool {
        self.person.is_loaded()
    }

    pub fn process_frame(
        &mut self,
        frame: &[u8],
        width: u32,
        height: u32,
        frame_available: bool,
    ) -> DetectionResults {
        let mut results = DetectionResults::default();

        if let Some(tamper_kind) = self.tamper.check_feed_loss(frame_available) {
            results.tamper = Some(self.tamper.create_event(tamper_kind));
            results.tamper_kind = Some(tamper_kind);
        }

        if frame_available && !frame.is_empty() {
            if let Some(tamper_kind) = self.tamper.check_brightness(frame) {
                results.tamper = Some(self.tamper.create_event(tamper_kind));
                results.tamper_kind = Some(tamper_kind);
            }

            if let Some(confidence) = self.motion.process_frame(frame) {
                results.motion = Some(self.motion.create_event(confidence, None));

                if self.person_enabled && self.person.is_loaded() {
                    if let Some(person_det) = self.person.detect(frame, width, height) {
                        results.person = Some(self.person.create_event(&person_det));
                    }
                }
            }
        }

        results.motion_active = self.motion.is_active();
        results.person_active = self.person.is_active();

        results
    }

    pub fn update_configs(
        &mut self,
        motion_config: MotionConfig,
        tamper_config: TamperConfig,
        person_enabled: bool,
        person_confidence: f32,
        person_cooldown_secs: u64,
    ) {
        self.motion.update_config(motion_config);
        self.tamper.update_config(tamper_config);
        self.person.set_enabled(person_enabled);
        self.person.set_confidence_threshold(person_confidence);
        self.person.set_cooldown(person_cooldown_secs);
        self.person_enabled = person_enabled;
    }
}

#[derive(Debug, Default)]
pub struct DetectionResults {
    pub motion: Option<DetectionEvent>,
    pub person: Option<DetectionEvent>,
    pub tamper: Option<DetectionEvent>,
    pub tamper_kind: Option<TamperKind>,
    pub motion_active: bool,
    pub person_active: bool,
}

impl DetectionResults {
    pub fn has_events(&self) -> bool {
        self.motion.is_some() || self.person.is_some() || self.tamper.is_some()
    }

    pub fn all_events(&self) -> Vec<&DetectionEvent> {
        let mut events = Vec::new();
        if let Some(ref e) = self.tamper { events.push(e); }
        if let Some(ref e) = self.motion { events.push(e); }
        if let Some(ref e) = self.person { events.push(e); }
        events
    }
}
