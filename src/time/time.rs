use std::time::{Instant, Duration};

pub struct Time {
    last_update: Instant,
    total_time: Duration,
    delta_time: f32,
    fps: f32,
    frame_count: u64,
}

impl Time {
    pub fn new() -> Self {
        Time {
            last_update: Instant::now(),
            total_time: Duration::new(0, 0),
            delta_time: 0.0,
            fps: 0.0,
            frame_count: 0,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);
        
        self.delta_time = elapsed.as_secs_f32();

        self.total_time += elapsed;

        self.frame_count += 1;

        if self.total_time.as_secs_f32() >= 1.0 {
            self.fps = self.frame_count as f32;
            self.frame_count = 0;
            self.total_time = Duration::new(0, 0);
        }

        self.last_update = now;
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub fn total_time(&self) -> f32 {
        self.total_time.as_secs_f32()
    }
}
