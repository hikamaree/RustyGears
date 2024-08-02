use crate::ModelRef;
use crate::BodyRef;
use crate::PhysicsWorld;
use crate::Shader;

pub struct Object {
    models: Vec<ModelRef>,
    body: Option<BodyRef>
}

impl Object {
    pub fn new() -> Self {
        Object {
            models: Vec::new(),
            body: None
        }
    }

    pub fn set_body(&mut self, body: BodyRef) {
        body.borrow_mut().movable = false;
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

    pub fn draw(&self, shader: &Shader) {
        for model in &self.models {
            model.borrow().draw(shader);
        }
    }
}
