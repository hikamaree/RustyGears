use crate::RenderTag;
use crate::Transform;
use crate::Command;
use crate::{Camera, Game};


pub struct SetDefaultCamera {
    pub id: u64,
}

impl Command for SetDefaultCamera {
    fn apply(self: Box<Self>, game: &mut Game) {
        game.scene.set_active_camera(self.id);
    }
}

pub struct AddCamera {
    pub position: (f32, f32, f32),
    pub yaw: f32,
    pub pitch: f32,
}

impl Command for AddCamera {
    fn apply(self: Box<Self>, game: &mut Game) {
        game.scene.add_camera(Camera::new(self.position, self.yaw, self.pitch));
    }
}

pub struct SpawnModel {
    pub file_path: String,
    pub transform: Transform,
    pub render_tags: Vec<RenderTag>
}

impl Command for SpawnModel {
    fn apply(self: Box<Self>, game: &mut Game) {
        game.spawn_model(&self.file_path, self.transform, self.render_tags);
    }
}

pub struct SetInstanceTransform {
    pub id: usize,
    pub transform: Transform,
}

impl Command for SetInstanceTransform {
    fn apply(self: Box<Self>, game: &mut Game) {
        if let Some(instance) = game.scene.instances.get_mut(&self.id) {
            instance.transform = self.transform;
        }
    }
}
