use crate::graphics::ModelRef;
use crate::graphics::Shader;
use crate::physics::BodyRef;
use crate::physics::PhysicsWorld;

#[derive(Clone)]
pub struct Character {
    pub models: Vec<ModelRef>,
    pub body: Option<BodyRef>
}

impl Character {
    pub fn new() -> Self {
        Character {
            models: Vec::new(),
            body: None
        }
    }

    pub fn set_body(&mut self, body: BodyRef) {
        body.borrow_mut().movable = true;
        self.body = Some(body);
    }

    pub fn add_model(&mut self, model: ModelRef) {
        self.models.push(model);
    }

    pub fn set_physics(&self, world: &mut PhysicsWorld) {
        if let Some(body) = &self.body {
            world.add_body(body.clone());
        }
    }

    pub fn update(&mut self) {
        if let Some(body) = &self.body {
            for model in &self.models {
                model.borrow_mut().set_position(body.borrow_mut().position);
            }
        }
    }

    pub fn draw(&self, shader: &Shader) {
        for model in &self.models {
            model.borrow().draw(shader);
        }
    }
}
