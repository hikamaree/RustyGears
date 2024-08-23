pub mod character;
pub mod object;

use cgmath::Vector3;
pub use character::*;
pub use object::*;

use crate::Shader;
use crate::PhysicsWorld;

pub trait Entity {
    fn clone_entity(&self) -> Box<dyn Entity>;
    fn draw(&self, shader: &Shader);
    fn set_physics(&self, world: &mut PhysicsWorld);
    fn update(&mut self);
    fn get_position(&self) -> Vector3<f32>;
}

impl Clone for Box<dyn Entity> {
    fn clone(&self) -> Box<dyn Entity> {
        self.clone_entity()
    }
}
