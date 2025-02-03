use cgmath::*;
use std::{f32::consts::FRAC_PI_2, sync::{Arc, Mutex}};

use crate::{Game, GearEvent};

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;


static ID_COUNTER: Mutex<u64> = Mutex::new(0);

//#[derive(Copy, Clone)]
pub struct Camera {
    id: u64,
    pub position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
    speed: f32,
    sensitivity: f32,
    forward: Vector3<f32>,
    right: Vector3<f32>,
    pub custom_handler: Option<Box<dyn FnMut(&mut Camera, &GearEvent, &mut Game) + Send + Sync>>,
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        let mut id_counter = ID_COUNTER.lock().unwrap();
        *id_counter += 1;
        let id = *id_counter;

        let mut camera = Camera {
            id,
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
            speed: 40.0,
            sensitivity: 0.4,
            forward: vec3(0.0, 0.0, -1.0),
            right: Vector3::zero(),
            custom_handler: None,
        };

        camera.update_camera_vectors();
        
        camera
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }

    pub fn update_view_proj(&mut self, projection: &Projection) {
        self.view_position = self.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * self.calc_matrix()).into()
    }

    pub fn get_uniform(&self) -> Box<[u8]> {
        let data: Vec<f32> = vec![
            self.view_position[0],
            self.view_position[1],
            self.view_position[2],
            self.view_position[3],
            self.view_proj[0][0], self.view_proj[0][1], self.view_proj[0][2], self.view_proj[0][3],
            self.view_proj[1][0], self.view_proj[1][1], self.view_proj[1][2], self.view_proj[1][3],
            self.view_proj[2][0], self.view_proj[2][1], self.view_proj[2][2], self.view_proj[2][3],
            self.view_proj[3][0], self.view_proj[3][1], self.view_proj[3][2], self.view_proj[3][3],
        ];

        bytemuck::cast_slice(&data).into()
    }

    pub fn move_forward(&mut self, dt: f32) {
        let velocity = self.speed * dt;
        self.position += self.forward * velocity;
    }

    pub fn move_backward(&mut self, dt: f32) {
        let velocity = self.speed * dt;
        self.position += -(self.forward * (velocity));
    }

    pub fn move_left(&mut self, dt: f32) {
        let velocity = self.speed * dt;
        self.position += -(self.right * velocity);
    }

    pub fn move_right(&mut self, dt: f32) {
        let velocity = self.speed * dt;
        self.position += self.right * velocity;
    }

    pub fn rotate(&mut self, xpos: f32, ypos: f32, dt: f32) {
        self.yaw += Rad(xpos) * self.sensitivity * dt;
        self.pitch += Rad(-ypos) * self.sensitivity * dt;
        self.update_camera_vectors();
    }

    fn update_camera_vectors(&mut self) {
        if self.pitch < -Rad(SAFE_FRAC_PI_2) {
            self.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if self.pitch > Rad(SAFE_FRAC_PI_2) {
            self.pitch = Rad(SAFE_FRAC_PI_2);
        }

        let forward = Vector3 {
            x: self.yaw.cos() * self.pitch.cos(),
            y: self.pitch.sin(),
            z: self.yaw.sin() * self.pitch.cos(),
        };
        self.forward = forward.normalize();
        self.right = self.forward.cross(Vector3::unit_y()).normalize();
    }

    pub fn set_handle(&mut self, handler: impl FnMut(&mut Camera, &GearEvent, &mut Game) + 'static + Send + Sync) -> &mut Self {
        self.custom_handler = Some(Box::new(handler));
        self
    }
}

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

pub struct CameraManagerGear {
    cameras: Vec<Arc<Mutex<Camera>>>,
    active_camera_index: usize,
    active_camera_id: u64,
}

impl CameraManagerGear {
    pub fn new() -> Self {
        Self {
            cameras: Vec::new(),
            active_camera_index: 0,
            active_camera_id: 0
        }
    }

    pub fn add_camera(&mut self, camera: Arc<Mutex<Camera>>) {
        if self.cameras.is_empty() {
            self.active_camera_id = camera.lock().unwrap().get_id();
        }
        self.cameras.push(camera);
    }

    pub fn set_active_camera(&mut self, index: usize) {
        if index < self.cameras.len() {
            self.active_camera_index = index;
            self.active_camera_id = self.cameras[index].lock().unwrap().get_id();
        }
    }

    pub fn get_active_camera(&mut self) -> Option<Arc<Mutex<Camera>>> {
        self.cameras.get_mut(self.active_camera_index).cloned()
    }

    pub fn index(&self) -> usize {
        self.active_camera_index
    }

    pub fn get_active_camera_id(&self) -> u64 {
        self.active_camera_id
    }

    pub fn count(&self) -> usize {
        self.cameras.len()
    }
}
