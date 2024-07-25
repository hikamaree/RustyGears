use super::*;
use std::ffi::CStr;

pub struct Fog {
    pub color: Vector3<f32>,
    pub density: f32,
}

impl Fog {
    pub fn new(color: Vector3<f32>, density: f32) -> Self {
        Self { color, density }
    }

    pub unsafe fn apply(&self, shader: &Shader) {
        shader.setVector3(c_str!("fog.color"), &self.color);
        shader.setFloat(c_str!("fog.density"), self.density);
    }
}
