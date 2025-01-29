use cgmath::*;
use std::{cell::RefCell, f32::consts::FRAC_PI_2, rc::Rc};

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

//#[derive(Copy, Clone)]
pub struct Camera {
    pub position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
    speed: f32,
    sensitivity: f32,
    forward: Vector3<f32>,
    right: Vector3<f32>
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
            speed: 40.0,
            sensitivity: 0.4,
            forward: vec3(0.0, 0.0, -1.0),
            right: Vector3::zero(),
        }
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
    cameras: Vec<Rc<RefCell<Camera>>>,
    active_camera_index: usize,
}

impl CameraManagerGear {
    pub fn new() -> Self {
        Self {
            cameras: Vec::new(),
            active_camera_index: 0,
        }
    }

    pub fn add_camera(&mut self, camera: Rc<RefCell<Camera>>) {
        self.cameras.push(camera);
    }

    pub fn set_active_camera(&mut self, index: usize) {
        if index < self.cameras.len() {
            self.active_camera_index = index;
        }
    }

    pub fn get_active_camera(&mut self) -> Option<Rc<RefCell<Camera>>> {
        self.cameras.get_mut(self.active_camera_index).cloned()
    }

    pub fn index(&self) -> usize {
        self.active_camera_index
    }

    pub fn count(&self) -> usize {
        self.cameras.len()
    }
}
