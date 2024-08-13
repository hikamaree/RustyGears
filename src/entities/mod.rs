pub mod character;
pub mod object;

pub use character::*;
pub use object::*;
use crate::BodyRef;
use crate::ModelRef;

use crate::Shader;
use crate::PhysicsWorld;

pub trait Entity {
    fn draw(&self, shader: &Shader);
    fn set_physics(&mut self, world: &mut PhysicsWorld) -> &mut dyn Entity;
    fn update(&mut self);
    fn set_body(&mut self, body: BodyRef) -> &mut dyn Entity;
    fn add_model(&mut self, model: ModelRef) -> &mut dyn Entity;
}

impl Entity for Object {
    fn draw(&self, shader: &Shader) {
        for model in &self.models {
            model.borrow().draw(shader, self.position, self.rotation);
        }
    }

    fn set_physics(&mut self, world: &mut PhysicsWorld) -> &mut dyn Entity {
        if let Some(body) = &self.body {
            world.add_body(body.clone());
        }
        self
    }

    fn update(&mut self) {
    }

    fn set_body(&mut self, body: BodyRef) -> &mut dyn Entity {
        body.borrow_mut().movable = false;
        self.body = Some(body);
        self
    }

    fn add_model(&mut self, model: ModelRef) -> &mut dyn Entity {
        self.models.push(model);
        self
    }
}

impl Entity for Character {
    fn draw(&self, shader: &Shader) {
        for model in &self.models {
            model.borrow().draw(shader, self.position, self.rotation);
        }
    }

    fn set_physics(&mut self, world: &mut PhysicsWorld) -> &mut dyn Entity {
        if let Some(body) = &self.body {
            world.add_body(body.clone());
        }
        self
    }

    fn update(&mut self) {
        if let Some(body) = &self.body {
            self.position = body.borrow().position;
            self.rotation = body.borrow().rotation;
        }
    }

    fn set_body(&mut self, body: BodyRef) -> &mut dyn Entity {
        body.borrow_mut().movable = true;
        self.body = Some(body);
        self
    }

    fn add_model(&mut self, model: ModelRef) -> &mut dyn Entity {
        self.models.push(model);
        self
    }
}
