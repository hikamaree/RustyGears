use crate::RenderTag;
use crate::Transform;
use crate::Instance;
use crate::BindGroupLayoutKey;
use crate::RenderObject;
use crate::CommandBuffer;
use crate::Camera;
use crate::Gear;
use crate::GearEvent;
use crate::Graphics;
use crate::Scene;
use crate::Time;

use std::sync::Arc;
use std::sync::Mutex;

use rayon::iter::ParallelIterator;
use rayon::iter::IntoParallelIterator;
use winit::event_loop::EventLoop;

pub struct GameView<'a> {
    pub graphics: &'a Graphics,
    pub time: &'a Time,
    pub scene: &'a Scene,
}

pub struct Game {
    pub(crate) setupfns: Vec<Box<dyn FnOnce(&mut Game) + Send>>,
    pub(crate) gears: Vec<Arc<Mutex<dyn Gear>>>,
    pub graphics: Option<Graphics>,
    pub time: Time,
    pub scene: Scene,
}

impl Game {

    /// Creates a new `Game` instance with default components.
    ///
    /// # Returns
    /// A new `Game` instance.

    pub fn new() -> Self {
        Game {
            setupfns: Vec::new(),
            gears: Vec::new(),
            graphics: None,
            time: Time::new(),
            scene: Scene::new(),
        }
    }

    /// Starts the main game loop using the event system.
    ///
    /// Creates a new `EventLoop` and begins running the application,
    /// passing a mutable reference to `self`.
    ///
    /// # Panics
    /// If the event loop fails to initialize or run.

    pub fn run(mut self) {
        let game_loop = EventLoop::new().expect("ERROR: Failed to crate game loop");
        game_loop.run_app(&mut self).expect("ERROR: Failed to run game loop");
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
    /// The camera is stored as a shared resource and is both managed by the scene
    /// and added to the gear list.
    ///
    /// # Returns
    /// A mutable reference to the `Game` instance to allow method chaining.

    pub fn add_camera(&mut self, camera: Camera) -> &mut Self {
        let camera = Arc::new(Mutex::new(camera)); 
        self.scene.add_camera(camera.clone());
        self.gears.push(camera);
        self
    }

    /// Queues a setup function to be called later during initialization.
    ///
    /// # Arguments
    /// * `setupfn` - A closure that takes a mutable reference to the game instance
    ///   and performs any desired setup logic (e.g. spawning models, adding systems).
    ///
    /// The setup functions are deferred and can be executed later in a controlled manner.
    ///
    /// # Returns
    /// A new `Game` instance with the setup function added to its queue.

    pub fn setup<F>(mut self, setupfn: F) -> Self 
    where F: FnOnce(&mut Game) + Send + 'static {
        self.setupfns.push(Box::new(setupfn));
        self
    }

    /// Dispatches an event to all gears in the game.
    ///
    /// # Arguments
    /// * `event` - A `GearEvent` to be handled by each gear.
    ///
    /// Clones the gear list and sends the event to each gear in parallel.
    /// Each gear can emit commands via `CommandBuffer`, which are collected and applied
    /// to the game after all event handling is done.

    pub(crate) fn dispatch_event(&mut self, event: GearEvent) {
        let gears = self.gears.clone();

        let game = GameView {
            graphics: self.graphics.as_ref().expect("ERROR: Graphics is not initialized"),
            time: &self.time,
            scene: &self.scene,
        };

        let command_buffers: Vec<CommandBuffer> = gears
            .into_par_iter()
            .map(|gear| {
                let mut cmd = CommandBuffer::new();
                let mut gear = gear.lock().unwrap();
                gear.handle_event(&event, &game, &mut cmd);
                cmd
            })
            .collect();

        for buffer in command_buffers {
            buffer.apply(self);
        }
    }

    pub fn spawn_model(&mut self, file_path: &str, transform: Transform, render_tags: Vec<RenderTag>) -> usize {
        if !self.scene.render_objects.contains_key(file_path) {
            self.load_model(file_path);
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

    pub fn load_model(&mut self, file_path: &str) {
        let graphics = self.graphics.as_ref().expect("ERROR: Graphics is not initialized");

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
