use crate::ModelRef;
use crate::BodyRef;

use cgmath::{
    Vector3,
    Quaternion,
    Zero,
    One
};

#[derive(Clone)]
pub struct Object {
    pub(super) models: Vec<ModelRef>,
    pub(super) body: Option<BodyRef>,
    pub(super) position: Vector3<f32>,
    pub(super) rotation: Quaternion<f32>,
}

impl Object {
    pub fn new() -> Self {
        Object {
            models: Vec::new(),
            body: None,
            position: Vector3::zero(),
            rotation: Quaternion::one(),
        }
    }
}
