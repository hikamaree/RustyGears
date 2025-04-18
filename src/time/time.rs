use std::time::{Instant, Duration};

/// The `Time` struct is used for tracking time-related information in an application,
/// such as the time between frames (delta time), total elapsed time,
/// and frames per second (FPS).

pub struct Time {
    last_update: Instant,
    total_time: Duration,
    fps_time: Duration,
    delta_time: f32,
    fps: f32,
    frame_count: u64,
}

impl Time {

    /// Creates a new instance of the `Time` struct.
    ///
    /// Initializes all time-related values to zero and sets `last_update` to the current time.

    pub(crate) fn new() -> Self {
        Time {
            last_update: Instant::now(),
            total_time: Duration::new(0, 0),
            fps_time: Duration::new(0, 0),
            delta_time: 0.0,
            fps: 0.0,
            frame_count: 0,
        }
    }

    /// Updates the time tracking values.
    ///
    /// This method should be called once per frame. It calculates the `delta_time`,
    /// accumulates the total elapsed time, and updates the FPS counter.
    /// If more than one second has passed, it computes the current FPS and resets the frame counter.

    pub(crate) fn update(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);
        
        self.delta_time = elapsed.as_secs_f32();

        self.total_time += elapsed;
        self.fps_time += elapsed;

        self.frame_count += 1;

        if self.fps_time.as_secs_f32() >= 1.0 {
            self.fps = self.frame_count as f32;
            self.frame_count = 0;
            self.fps_time = Duration::new(0, 0);
        }

        self.last_update = now;
    }

    /// Returns the smoothed delta time in seconds.
    /// 
    /// This value represents the time elapsed between the last two frames,
    /// averaged over the last 10 frames to reduce fluctuations.

    pub fn delta_time(&self) -> f32 {
        self.delta_time
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
