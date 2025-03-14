use std::sync::Arc;
use std::sync::Mutex;

use crate::window::State;
use crate::Camera;
use crate::CameraManager;
use crate::Gear;
use crate::GearEvent;
use crate::Time;

pub struct Game {
    gears: Vec<Arc<Mutex<dyn Gear>>>,
    pub(crate) state: Arc<Mutex<State>>,
    pub time: Time,
    pub cameras: CameraManager,
}

impl Game {
    pub(crate) fn new(state: State) -> Self {
        Self {
            gears: Vec::new(),
            state: Arc::new(Mutex::new(state)),
            time: Time::new(),
            cameras: CameraManager::new(),
        }
    } 

    /// Adds a new gear to the game.
    /// # Arguments
    /// * `gear` - An instance of a type that implements the `Gear` trait.
    ///
    /// # Returns
    /// A mutable reference to the `Game` instance to allow method chaining.

    pub fn add_gear<T: Gear + 'static>(&mut self, gear: T) -> &mut Self {
        self.gears.push(Arc::new(Mutex::new(gear)));
        self
    }

    /// Adds a new camera to the game.
    ///
    /// # Arguments
    /// * `camera` - An instance of `Camera`.
    ///
    /// The camera is stored as a shared resource and is both managed by the camera manager
    /// and added to the gear list.
    ///
    /// # Returns
    /// A mutable reference to the `Game` instance to allow method chaining.

    pub fn add_camera(&mut self, camera: Camera) -> &mut Self {
        let camera = Arc::new(Mutex::new(camera)); 
        self.cameras.add_camera(camera.clone());
        self.gears.push(camera);
        self
    }

    pub(crate) fn dispatch_event(self_arc: Arc<Mutex<Self>>, event: GearEvent) {
        let gears = {
            let mut game = self_arc.lock().unwrap();
            std::mem::take(&mut game.gears)
        };

        let handles: Vec<_> = gears
            .iter()
            .map(|gear| {
                let gear = gear.clone();
                let event = event.clone();
                let game_arc = self_arc.clone();

                std::thread::spawn(move || {
                    let mut game = game_arc.lock().unwrap();
                    gear.lock().unwrap().handle_event(&event, &mut game);
                })
            })
        .collect();

        for handle in handles {
            let _ = handle.join();
        }

        let mut game = self_arc.lock().unwrap();
        game.gears = gears;
    }
}
