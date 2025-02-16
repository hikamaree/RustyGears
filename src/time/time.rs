use std::time::{Instant, Duration};

pub struct Time {
    last_update: Instant,
    total_time: Duration,
    delta_time: f32,
    smoothed_delta_time: f32,
    fps: f32,
    frame_count: u64,
    delta_time_history: [f32; 10],
    history_index: usize,
}

impl Time {
    pub(crate) fn new() -> Self {
        Time {
            last_update: Instant::now(),
            total_time: Duration::new(0, 0),
            delta_time: 0.0,
            smoothed_delta_time: 0.0,
            fps: 0.0,
            frame_count: 0,
            delta_time_history: [0.0; 10],
            history_index: 0,
        }
    }

    pub(crate) fn update(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);
        
        self.delta_time = elapsed.as_secs_f32();

        self.delta_time_history[self.history_index] = self.delta_time;
        self.history_index = (self.history_index + 1) % self.delta_time_history.len();

        self.smoothed_delta_time = self.delta_time_history.iter().sum::<f32>() / self.delta_time_history.len() as f32;

        self.total_time += elapsed;

        self.frame_count += 1;

        if self.total_time.as_secs_f32() >= 1.0 {
            self.fps = self.frame_count as f32;
            self.frame_count = 0;
            self.total_time = Duration::new(0, 0);
        }

        self.last_update = now;
    }


    /// Returns the smoothed delta time in seconds.
    /// 
    /// This value represents the time elapsed between the last two frames,
    /// averaged over the last 10 frames to reduce fluctuations.

    pub fn delta_time(&self) -> f32 {
        self.smoothed_delta_time
    }

    /// Returns the frames per second (FPS).
    /// 
    /// FPS is updated once per second and represents
    /// how many frames were rendered in the last second.

    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// Returns the total elapsed time in seconds.
    /// 
    /// This value represents the total time since the `Time` instance was created,
    /// continuously increasing as the game runs.

    pub fn total_time(&self) -> f32 {
        self.total_time.as_secs_f32()
    }
}
