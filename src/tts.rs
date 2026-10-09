#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TtsSettings {
    pub input_max: f32,

    pub noise_floor: f32,

    pub loudness_ceiling: f32,

    pub idle_speed: f32,

    pub max_speed: f32,

    pub min_scale: f32,

    pub max_scale: f32,

    pub min_glow: f32,

    pub max_glow: f32,

    pub max_opacity: f32,

    pub attack: f32,

    pub release: f32,
}

impl Default for TtsSettings {
    fn default() -> Self {
        TtsSettings {
            input_max: 1.0,
            noise_floor: 0.04,
            loudness_ceiling: 0.9,
            idle_speed: 0.6,
            max_speed: 3.0,
            min_scale: 0.94,
            max_scale: 1.14,
            min_glow: 0.85,
            max_glow: 1.6,
            max_opacity: 1.0,
            attack: 0.5,
            release: 0.12,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TtsReactive {
    settings: TtsSettings,
    smoothed: f32,
    frame_dt: f32,
}

impl Default for TtsReactive {
    fn default() -> Self {
        TtsReactive::new(TtsSettings::default())
    }
}

impl TtsReactive {
    pub fn new(settings: TtsSettings) -> Self {
        TtsReactive {
            settings,
            smoothed: 0.0,
            frame_dt: 1.0 / 60.0,
        }
    }

    pub fn settings(&self) -> &TtsSettings {
        &self.settings
    }

    pub fn set_settings(&mut self, settings: TtsSettings) {
        self.settings = settings;
    }

    pub fn update(&mut self, level: f32, dt: f32) {
        let s = &self.settings;
        let raw = level.clamp(0.0, s.input_max);
        let norm = if s.loudness_ceiling <= s.noise_floor {
            0.0
        } else {
            ((raw - s.noise_floor) / (s.loudness_ceiling - s.noise_floor)).clamp(0.0, 1.0)
        };
        self.frame_dt = dt.clamp(0.0, 0.25).max(1e-6);
        let rate = if norm > self.smoothed {
            s.attack
        } else {
            s.release
        };
        let alpha = 1.0 - (1.0 - rate).powf(self.frame_dt * 60.0);
        self.smoothed += (norm - self.smoothed) * alpha;
    }

    pub fn level(&self) -> f32 {
        self.smoothed
    }

    pub fn speed(&self) -> f64 {
        let s = &self.settings;
        (s.idle_speed + (s.max_speed - s.idle_speed) * self.smoothed) as f64
    }

    pub fn scale(&self) -> f32 {
        let s = &self.settings;
        s.min_scale + (s.max_scale - s.min_scale) * self.smoothed
    }

    pub fn glow(&self) -> f64 {
        let s = &self.settings;
        (s.min_glow + (s.max_glow - s.min_glow) * self.smoothed) as f64
    }

    pub fn opacity(&self) -> f32 {
        let s = &self.settings;
        1.0 + (s.max_opacity - 1.0) * self.smoothed
    }

    pub fn sized(&self, base_size: f32) -> f32 {
        base_size * self.scale()
    }

    pub fn reset(&mut self) {
        self.smoothed = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_is_min_scale_and_idle_speed() {
        let s = TtsSettings::default();
        let mut r = TtsReactive::new(s);
        r.update(0.0, 1.0 / 60.0);
        assert_eq!(r.scale(), s.min_scale);
        assert_eq!(r.speed(), s.idle_speed as f64);
    }

    #[test]
    fn loudness_raises_scale_and_speed() {
        let mut r = TtsReactive::default();
        for _ in 0..120 {
            r.update(1.0, 1.0 / 60.0);
        }
        assert!(r.level() > 0.9, "level {}", r.level());
        assert!(r.scale() > 1.1, "scale {}", r.scale());
        assert!(r.speed() > 2.7, "speed {}", r.speed());
        assert!(r.glow() > 1.5, "glow {}", r.glow());
    }

    #[test]
    fn attack_is_faster_than_release() {
        let mut r = TtsReactive::default();
        r.update(1.0, 1.0 / 60.0);
        let after_one_loud = r.level();
        r.update(0.0, 1.0 / 60.0);
        let drop = after_one_loud - r.level();
        assert!(drop < after_one_loud, "should not snap to zero");
        assert!(after_one_loud > 0.0);
    }

    #[test]
    fn level_is_frame_rate_independent() {
        let mut a = TtsReactive::default();
        for _ in 0..60 {
            a.update(1.0, 1.0 / 60.0);
        }
        let mut b = TtsReactive::default();
        for _ in 0..30 {
            b.update(1.0, 1.0 / 30.0);
        }
        assert!(
            (a.level() - b.level()).abs() < 0.05,
            "{} vs {}",
            a.level(),
            b.level()
        );
    }

    #[test]
    fn below_noise_floor_is_silence() {
        let mut r = TtsReactive::default();
        for _ in 0..60 {
            r.update(0.01, 1.0 / 60.0);
        }
        assert!(r.level() < 1e-6);
        assert_eq!(r.scale(), r.settings().min_scale);
        assert_eq!(r.glow(), r.settings().min_glow as f64);
    }
}
