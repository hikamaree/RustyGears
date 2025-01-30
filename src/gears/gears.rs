use std::any::Any;
use winit::{event::{KeyEvent, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};
use crate::{Camera, Game};

pub trait Gear: Send + Sync {
    fn handle_event(&mut self, event: &GearEvent, game: &mut Game);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

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
