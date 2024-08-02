pub mod character;
pub mod object;

pub use character::*;
pub use object::*;

use crate::Shader;
use crate::PhysicsWorld;

pub enum Entity {
    Character(Character),
    Object(Object)
}

impl Entity {
    pub fn draw(&self, shader: &Shader) {
        match self {
            Entity::Character(character) => character.draw(shader),
            Entity::Object(object) => object.draw(shader),
        }
    }

    pub fn set_physics(&self, world: &mut PhysicsWorld) {
        match self {
            Entity::Character(character) => character.set_physics(world),
            Entity::Object(object) => object.set_physics(world),
        }
    }

    pub fn update(&mut self) {
        match self {
            Entity::Character(character) => character.update(),
            Entity::Object(_) => ()
        }
    }

}
