use crate::InstanceRaw;
use cgmath::Matrix4;

#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: cgmath::Vector3<f32>
}

impl Transform {
    pub fn to_matrix(&self) -> Matrix4<f32> {
        let translation = Matrix4::from_translation(self.position);
        let rotation = Matrix4::from(self.rotation);
        let scale = Matrix4::from_nonuniform_scale(
            self.scale.x,
            self.scale.y,
            self.scale.z,
        );
        translation * rotation * scale
    }
}


#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum RenderTag {
    PBR,
    Unlit,
    Wireframe,
    ShadowMap,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Instance {
    id: usize,
    pub transform: Transform,
    pub render_tags: Vec<RenderTag>,
    pub name: String,
}

impl Instance {
    pub fn new(name: String, transform: Transform, render_tags: Vec<RenderTag>) -> Self {
        Self {
            id: Instance::gen_id(),
            transform,
            render_tags,
            name,
        }
    }

    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.transform.position)
                * cgmath::Matrix4::from(self.transform.rotation)
                * cgmath::Matrix4::from_nonuniform_scale(self.transform.scale.x, self.transform.scale.y, self.transform.scale.z))
            .into(),
            normal: cgmath::Matrix3::from(self.transform.rotation).into(),
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    fn gen_id() -> usize {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }
}
