use cgmath;
use cgmath::vec3;
use cgmath::prelude::*;

type Point3 = cgmath::Point3<f32>;
type Vector3 = cgmath::Vector3<f32>;
type Matrix4 = cgmath::Matrix4<f32>;

use core::cell::RefCell;
use std::rc::Rc;

const YAW: f32 = 0.0;
const PITCH: f32 = 0.0;
const SPEED: f32 = 4.0;
const SENSITIVTY: f32 = 0.1;
const ZOOM: f32 = 45.0;

#[derive(Clone, Copy)]
pub struct Camera {
    pub position: Point3,
    pub front: Vector3,
    pub up: Vector3,
    pub right: Vector3,
    pub world_up: Vector3,
    pub yaw: f32,
    pub pitch: f32,
    pub movement_speed: f32,
    pub mouse_sensitivity: f32,
    pub zoom: f32,
    pub last_x: f32,
    pub last_y: f32,
}

pub type CameraRef = Rc<RefCell<Camera>>;

impl Default for Camera {
    fn default() -> Camera {
        let mut camera = Camera {
            position: Point3::new(0.0, 12.0, 0.0),
            front: vec3(0.0, 0.0, -1.0),
            up: Vector3::zero(),
            right: Vector3::zero(),
            world_up: Vector3::unit_y(),
            yaw: YAW,
            pitch: PITCH,
            movement_speed: SPEED,
            mouse_sensitivity: SENSITIVTY,
            zoom: ZOOM,
            last_x: 0.0,
            last_y: 0.0,
        };
        camera.update_camera_vectors();
        camera
    }
}

impl Camera {
    pub fn create() -> CameraRef {
        Rc::new(RefCell::new(Camera::default()))
    }

    pub fn get_view_matrix(&self) -> Matrix4 {
        Matrix4::look_at_rh(self.position, self.position + self.front, self.up)
    }

    pub fn move_forward(&mut self, delta_time: f32) {
        let velocity = self.movement_speed * delta_time;
        self.position += self.front * velocity;
    }

    pub fn move_backward(&mut self, delta_time: f32) {
        let velocity = self.movement_speed * delta_time;
        self.position += -(self.front * velocity);
    }

    pub fn move_left(&mut self, delta_time: f32) {
        let velocity = self.movement_speed * delta_time;
        self.position += -(self.right * velocity);
    }

    pub fn move_right(&mut self, delta_time: f32) {
        let velocity = self.movement_speed * delta_time;
        self.position += self.right * velocity;
    }

    pub fn rotate(&mut self, xpos: f32, ypos: f32, constrain_pitch: bool) {
        let xoffset = (xpos - self.last_x) * self.mouse_sensitivity;
        let yoffset = (self.last_y - ypos) * self.mouse_sensitivity;

        self.last_x = xpos;
        self.last_y = ypos;

        self.yaw += xoffset;
        self.pitch += yoffset;

        self.yaw %= 360.0;

        if constrain_pitch {
            if self.pitch > 89.0 {
                self.pitch = 89.0;
            }
            if self.pitch < -89.0 {
                self.pitch = -89.0;
            }
        }

        self.update_camera_vectors();
    }

    pub fn zoom(&mut self, yoffset: f32) {
        if self.zoom >= 1.0 && self.zoom <= 45.0 {
            self.zoom -= yoffset;
        }
        if self.zoom <= 1.0 {
            self.zoom = 1.0;
        }
        if self.zoom >= 45.0 {
            self.zoom = 45.0;
        }
    }

    fn update_camera_vectors(&mut self) {
        let front = Vector3 {
            x: self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            y: self.pitch.to_radians().sin(),
            z: self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        };
        self.front = front.normalize();
        self.right = self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }

    pub fn print(&self) {
        println!("Position {:?}", self.position);
        println!("Yaw {}", self.yaw);
        println!("Pitch {}", self.pitch);
    }
}

