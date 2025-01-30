use crate::Camera;
use crate::CameraManagerGear;
use crate::Time;
use crate::game::gameloop::GameLoop;
use std::any::Any;

use winit::event::ElementState;
use winit::event::KeyEvent;
use winit::event::WindowEvent;

use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;

pub trait Gear: Send + Sync {
    fn handle_event(&mut self, event: &GearEvent, game: &mut Game);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

use std::cell::RefCell;
use std::rc::Rc;

pub enum GearEvent<'a> {
    Update(f32),
    RenderRequested(),
    Input(&'a winit::event::WindowEvent),
}

pub struct RenderingGear;

impl Gear for RenderingGear {
    fn handle_event(&mut self, event: &GearEvent, _game: &mut Game) {
        if let GearEvent::RenderRequested() = event {
            println!("Rendering frame...");
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct PhysicsGear;

impl Gear for PhysicsGear {
    fn handle_event(&mut self, event: &GearEvent, _game: &mut Game) {
        if let GearEvent::Update(delta_time) = event {
            println!("Updating physics with delta time: {}", delta_time);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct InputGear;

impl Gear for InputGear {
    fn handle_event(&mut self, event: &GearEvent, _game: &mut Game) {
        if let GearEvent::Input(window_event) = event {
            println!("Processing input: {:?}", window_event);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Gear for Camera {
    fn handle_event(&mut self, event: &GearEvent, game: &mut Game) {
        if let GearEvent::Input(window_event) = event {
            let dt = game.delta_time();
            match window_event {
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(key),
                            state,
                            ..
                        },
                        ..
                } => {
                    match key {
                        KeyCode::KeyW | KeyCode::ArrowUp => {
                            self.move_forward(dt);
                        }
                        KeyCode::KeyS | KeyCode::ArrowDown => {
                            self.move_backward(dt);
                        }
                        KeyCode::KeyA | KeyCode::ArrowLeft => {
                            self.move_left(dt);
                        }
                        KeyCode::KeyD | KeyCode::ArrowRight => {
                            self.move_right(dt);
                        }
                        KeyCode::KeyC => {
                            if *state == ElementState::Pressed {
                                let index = game.get_camera_index();
                                println!("{}, {}", index, game.get_camera_cnt());
                                game.set_active_camera((index + 1) % game.get_camera_cnt());
                            }
                        }
                        _ => {},
                    }
                }
                _ => {},
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct Game {
    gears: Vec<Rc<RefCell<dyn Gear>>>,
    timer: Time,
    camera_manager: CameraManagerGear,
}

impl<'a> Game {
    pub fn new() -> Self {
        Self {
            gears: Vec::new(),
            timer: Time::new(),
            camera_manager: CameraManagerGear::new(),
        }
    }

    pub fn default() -> Self {
        let mut game = Game {
            gears: Vec::new(),
            timer: Time::new(),
            camera_manager: CameraManagerGear::new(),
        };

        //game.add_gear(RenderingGear);
        //game.add_gear(PhysicsGear);

        let camera = Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        game.add_camera(camera);

        let camera1 = Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        game.add_camera(camera1);

        game
    }

    pub fn add_gear<T: Gear + 'static>(&mut self, gear: T) -> &mut Self {
        self.gears.push(Rc::new(RefCell::new(gear)));
        self
    }

    pub fn add_camera(&mut self, camera: Camera) -> &mut Self {
        let camera = Rc::new(RefCell::new(camera)); 
        self.camera_manager.add_camera(camera.clone());
        self.gears.push(camera);
        self
    }

    pub(crate) fn dispatch_event(&mut self, event: GearEvent) {
        let mut gears = std::mem::take(&mut self.gears); // Privremeno uzmi vlasništvo nad gears
        for gear in &mut gears {
            gear.borrow_mut().handle_event(&event, self);
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

    pub fn get_camera(&mut self) -> Rc<RefCell<Camera>> {
        self.camera_manager.get_active_camera().expect("no camera found")
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
