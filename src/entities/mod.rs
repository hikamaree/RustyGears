pub mod character;
pub mod object;

pub use character::*;
pub use object::*;

use cgmath::{
    Quaternion,
    Vector3,
};

use crate::BodyRef;
use crate::Model;

use crate::Shader;
use crate::PhysicsWorld;

pub trait Entity {
       fn clone_entity(&self) -> Box<dyn Entity>;

    fn draw(&self, shader: &Shader);
    fn set_physics(&mut self, world: &mut PhysicsWorld) -> Box<dyn Entity>;
    fn update(&mut self);
    fn set_body(&mut self, body: BodyRef) -> Box<dyn Entity>;
    fn set_mass(&mut self, mass: f32) -> Box<dyn Entity>;
    fn add_model(&mut self, model: Model) -> Box<dyn Entity>;
    fn set_position(&mut self, position: Vector3<f32>) -> Box<dyn Entity>;
    fn set_rotation(&mut self, rotation: Quaternion<f32>) -> Box<dyn Entity>;
    fn set_velocity(&mut self, velocity: Vector3<f32>) -> Box<dyn Entity>;
    fn apply_force(&mut self, force: Vector3<f32>) -> Box<dyn Entity>;
    fn apply_torque(&mut self, force: Vector3<f32>) -> Box<dyn Entity>;

}

impl Clone for Box<dyn Entity> {
    fn clone(&self) -> Box<dyn Entity> {
        self.clone_entity()
    }
}
