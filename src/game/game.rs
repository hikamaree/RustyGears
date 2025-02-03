use std::sync::Arc;
use std::sync::Mutex;

use crate::Camera;
use crate::CameraManagerGear;
use crate::Gear;
use crate::GearEvent;
use crate::Time;
use crate::game::gameloop::GameLoop;

pub struct Game {
    gears: Vec<Arc<Mutex<dyn Gear>>>,
    timer: Time,
    camera_manager: CameraManagerGear,
}

impl Game {
    pub fn new() -> Self {
        Self {
            gears: Vec::new(),
            timer: Time::new(),
            camera_manager: CameraManagerGear::new(),
        }
    }

    pub fn add_gear<T: Gear + 'static>(&mut self, gear: T) -> &mut Self {
        self.gears.push(Arc::new(Mutex::new(gear)));
        self
    }

    pub fn add_camera(&mut self, camera: Camera) -> &mut Self {
        let camera = Arc::new(Mutex::new(camera)); 
        self.camera_manager.add_camera(camera.clone());
        self.gears.push(camera);
        self
    }

    pub(crate) fn dispatch_event(&mut self, event: GearEvent) {
        let mut gears = std::mem::take(&mut self.gears);
        for gear in &mut gears {
            gear.lock().unwrap().handle_event(&event, self);
        }
        self.gears = gears;
    }

    pub fn run(&mut self) -> &mut Self {
        let mut gameloop = GameLoop::new();
        gameloop.run(self);
        self
    }

    pub fn delta_time(&self) -> f32 {
        self.timer.delta_time()
    }

    pub fn fps(&self) -> f32 {
        self.timer.fps()
    }

    pub(crate) fn update_time(&mut self) {
        self.timer.update();
    }

    pub fn get_camera(&mut self) -> Arc<Mutex<Camera>> {
        self.camera_manager.get_active_camera().expect("no camera found")
    }

    pub fn get_camera_id(&self) -> u64 {
        self.camera_manager.get_active_camera_id()
    }

    pub fn set_active_camera(&mut self, index: usize) {
        self.camera_manager.set_active_camera(index);
    }

    pub fn get_camera_cnt(&self) -> usize {
        self.camera_manager.count()
    }

    pub fn get_camera_index(&self) -> usize {
        self.camera_manager.index()
    }
}
