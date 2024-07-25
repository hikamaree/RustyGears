#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use cgmath;
use cgmath::vec3;
use cgmath::prelude::*;

type Point3 = cgmath::Point3<f32>;
type Vector3 = cgmath::Vector3<f32>;
type Matrix4 = cgmath::Matrix4<f32>;

const YAW: f32 = -90.0;
const PITCH: f32 = 0.0;
const SPEED: f32 = 4.0;
const SENSITIVTY: f32 = 0.1;
const ZOOM: f32 = 45.0;

#[derive(Clone, Copy)]
pub struct Camera {
    pub Position: Point3,
    pub Front: Vector3,
    pub Up: Vector3,
    pub Right: Vector3,
    pub WorldUp: Vector3,
    pub Yaw: f32,
    pub Pitch: f32,
    pub MovementSpeed: f32,
    pub MouseSensitivity: f32,
    pub Zoom: f32,
    last_x: f32,
    last_y: f32,
}

impl Default for Camera {
    fn default() -> Camera {
        let mut camera = Camera {
            Position: Point3::new(0.0, 0.0, 0.0),
            Front: vec3(0.0, 0.0, -1.0),
            Up: Vector3::zero(),
            Right: Vector3::zero(),
            WorldUp: Vector3::unit_y(),
            Yaw: YAW,
            Pitch: PITCH,
            MovementSpeed: SPEED,
            MouseSensitivity: SENSITIVTY,
            Zoom: ZOOM,
            last_x: 0.0,
            last_y: 0.0,
        };
        camera.updateCameraVectors();
        camera
    }
}

impl Camera {
    pub fn GetViewMatrix(&self) -> Matrix4 {
        Matrix4::look_at_rh(self.Position, self.Position + self.Front, self.Up)
    }

    pub fn move_forward(&mut self, delta_time: f32) {
        let velocity = self.MovementSpeed * delta_time;
        self.Position += self.Front * velocity;
    }

    pub fn move_backward(&mut self, delta_time: f32) {
        let velocity = self.MovementSpeed * delta_time;
        self.Position += -(self.Front * velocity);
    }

    pub fn move_left(&mut self, delta_time: f32) {
        let velocity = self.MovementSpeed * delta_time;
        self.Position += -(self.Right * velocity);
    }

    pub fn move_right(&mut self, delta_time: f32) {
        let velocity = self.MovementSpeed * delta_time;
        self.Position += self.Right * velocity;
    }

    pub fn rotate(&mut self, xpos: f32, ypos: f32, constrainPitch: bool) {
        let xoffset = (xpos - self.last_x) * self.MouseSensitivity;
        let yoffset = (self.last_y - ypos) * self.MouseSensitivity;

        self.last_x = xpos;
        self.last_y = ypos;

        self.Yaw += xoffset;
        self.Pitch += yoffset;

        if constrainPitch {
            if self.Pitch > 89.0 {
                self.Pitch = 89.0;
            }
            if self.Pitch < -89.0 {
                self.Pitch = -89.0;
            }
        }

        self.updateCameraVectors();
    }

    pub fn zoom(&mut self, yoffset: f32) {
        if self.Zoom >= 1.0 && self.Zoom <= 45.0 {
            self.Zoom -= yoffset;
        }
        if self.Zoom <= 1.0 {
            self.Zoom = 1.0;
        }
        if self.Zoom >= 45.0 {
            self.Zoom = 45.0;
        }
    }

    fn updateCameraVectors(&mut self) {
        let front = Vector3 {
            x: self.Yaw.to_radians().cos() * self.Pitch.to_radians().cos(),
            y: self.Pitch.to_radians().sin(),
            z: self.Yaw.to_radians().sin() * self.Pitch.to_radians().cos(),
        };
        self.Front = front.normalize();
        self.Right = self.Front.cross(self.WorldUp).normalize();
        self.Up = self.Right.cross(self.Front).normalize();
    }
}

