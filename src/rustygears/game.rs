use crate::RenderTag;
use crate::Transform;
use crate::Instance;
use crate::BindGroupLayoutKey;
use crate::RenderObject;
use crate::CommandBuffer;
use crate::Camera;
use crate::CameraManager;
use crate::Gear;
use crate::GearEvent;
use crate::Graphics;
use crate::Scene;
use crate::Time;

use std::sync::Arc;
use std::sync::Mutex;

use rayon::iter::ParallelIterator;
use rayon::iter::IntoParallelIterator;

pub struct Game {
    gears: Vec<Arc<Mutex<dyn Gear>>>,
    pub graphics: Graphics,
    pub time: Time,
    pub cameras: CameraManager,
    pub scene: Scene,
}

impl Game {
    pub(crate) async fn new(window: Arc<winit::window::Window>) -> Self {
        let game = Game {
            gears: Vec::new(),
            graphics: Graphics::new(window).await,
            time: Time::new(),
            cameras: CameraManager::new(),
            scene: Scene::new(),
        };

        game
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

    pub(crate) fn dispatch_event(&mut self, event: GearEvent) {
        let gears = self.gears.clone();

        let command_buffers: Vec<CommandBuffer> = gears
            .into_par_iter()
            .map(|gear| {
                let mut cmd = CommandBuffer::new();
                let mut gear = gear.lock().unwrap();
                gear.handle_event(&event, &self, &mut cmd);
                cmd
            })
            .collect();

        for buffer in command_buffers {
            buffer.apply(self);
        }
    }

    pub fn spawn_model(&mut self, file_path: &str, transform: Transform, render_tags: Vec<RenderTag>) -> usize {
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

        instance_id
    }

    fn load_model(&mut self, file_path: &str) {
        let graphics = &self.graphics;

        let texture_layout = graphics.bind_group_layouts.get(&BindGroupLayoutKey::Texture)
            .expect("Texture bind group layout not found");

        let rt = tokio::runtime::Runtime::new().unwrap();

        let model = rt.block_on(async {
            super::resources::load_model(
                file_path,
                &graphics.device,
                &graphics.queue,
                texture_layout,
            ).await
        });

        self.scene.add_render_object(file_path.to_string(), RenderObject { model });
    }

    fn generate_unique_id(&self) -> usize {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }
}
