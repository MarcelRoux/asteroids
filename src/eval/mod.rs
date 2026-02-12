pub struct PerformanceGuard {
    last_fps: f32,
    degrade_threshold: f32,
}

impl PerformanceGuard {
    pub fn new() -> Self {
        Self {
            last_fps: 60.0,
            degrade_threshold: 55.0,
        }
    }

    pub fn observe(&mut self, fps: f32) {
        self.last_fps = fps;
    }

    pub fn should_degrade(&self) -> bool {
        self.last_fps < self.degrade_threshold
    }
}
