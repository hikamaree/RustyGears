use crate::system::gpu::*;
use crate::Gear;
use crate::GearEvent;
use crate::Game;
use std::io;
use std::io::Write;
use std::any::Any;

pub struct EngineStats {
    gpu: GpuInfo,
    cl: u32,
    lastupdate: f32,
}

impl EngineStats {
    pub fn new() -> Self {
        print!("\n\n\n\n\n");
        print!("\x1B[?25l");
        Self {
            gpu: GpuInfo::new(),
            cl: 5,
            lastupdate: 0.0,
        }
    }
}

impl Drop for EngineStats {
    fn drop(&mut self) {
        println!("\x1B[?25h");
    }
}

impl Gear for EngineStats {
    fn handle_event(&mut self, event: &GearEvent, game: &mut Game) {
        if let GearEvent::Update() = event {
            if game.time.total_time() - self.lastupdate <= 1.0 {
                return;
            }

            self.lastupdate = game.time.total_time();

            print!("\x1B[{}A", self.cl);

            for _ in 1..self.cl {
                print!("\x1B[2K");
            }

            println!("FPS: {}", game.time.fps());
            println!("{}", self.gpu.display());

            io::stdout().flush().unwrap();
        }
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
