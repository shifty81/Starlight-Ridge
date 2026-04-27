use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct FrameTimer {
    started_at: Instant,
    last_frame_at: Instant,
    frame_count: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct FrameStats {
    pub frame_index: u64,
    pub delta: Duration,
    pub uptime: Duration,
}

impl FrameTimer {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            started_at: now,
            last_frame_at: now,
            frame_count: 0,
        }
    }

    pub fn tick(&mut self) -> FrameStats {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_at);
        self.last_frame_at = now;
        self.frame_count += 1;
        FrameStats {
            frame_index: self.frame_count,
            delta,
            uptime: now.duration_since(self.started_at),
        }
    }
}
