use crate::RenderTag;
use crate::Transform;
use crate::Instance;
use crate::BindGroupLayoutKey;
use crate::RenderObject;


use std::sync::Arc;
use std::sync::Mutex;

use crate::Camera;
use crate::CameraManager;
use crate::Gear;
use crate::GearEvent;
use crate::Graphics;
use crate::Scene;
use crate::Time;


pub struct Game {
    gears: Vec<Arc<Mutex<dyn Gear>>>,
    pub(crate) graphics: Graphics,
    pub time: Time,
    pub cameras: CameraManager,
    pub scene: Scene,
}

impl Game {
    pub(crate) async fn new(window: Arc<winit::window::Window>) -> Self {
        Self {
            gears: Vec::new(),
            graphics: Graphics::new(window).await,
            time: Time::new(),
            cameras: CameraManager::new(),
            scene: Scene::new(),
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

    // pub(crate) fn dispatch_event(game: &mut Game, event: GearEvent) {
    //     let mut gears = std::mem::take(&mut game.gears);
    //
    //     for gear in &mut gears {
    //         gear.handle_event(&event, game);
    //     }
    //
    //     game.gears = gears;
    // }
    
















    pub fn spawn_model(
        &mut self,
        file_path: &str,
        transform: Transform,
        render_tags: Vec<RenderTag>,
    ) -> anyhow::Result<usize> {
        if !self.scene.render_objects.contains_key(file_path) {
            let _ = self.load_model(file_path);
        }

        let instance_id = self.generate_unique_id();

        let instance = Instance {
            id: instance_id,
            object_name: file_path.to_string(),
            transform,
            render_tags,
        };

        self.scene.add_instance(instance);

        Ok(instance_id)
    }

    fn load_model(&mut self, file_path: &str) -> anyhow::Result<()> {
        let graphics = &self.graphics;
        
        let texture_layout = graphics.bind_group_layouts.get(&BindGroupLayoutKey::Texture)
            .ok_or_else(|| anyhow::anyhow!("Texture bind group layout not found"))?;

        let rt = tokio::runtime::Runtime::new()?;

        let model = rt.block_on(async {
            super::resources::load_model(
                file_path,
                &graphics.device,
                &graphics.queue,
                texture_layout,
            ).await
        })?;

        self.scene.add_render_object(file_path.to_string(), RenderObject { model });
        Ok(())
    }

    fn generate_unique_id(&self) -> usize {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }
}
