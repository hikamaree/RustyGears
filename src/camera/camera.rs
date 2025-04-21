use cgmath::Matrix;
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicU64;
use crate::Projection;
use cgmath::vec4;
use cgmath::Zero;
use cgmath::vec3;
use cgmath::Vector3;
use cgmath::Point3;
use cgmath::Matrix4;
use cgmath::Angle;
use cgmath::Rad;
use cgmath::SquareMatrix;
use cgmath::InnerSpace;

const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// The `Camera` struct represents a camera in a 3D space.

#[derive(Clone)]
pub struct Camera {
    pub id: u64,
    pub position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub roll: Rad<f32>,
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
    pub speed: f32,
    pub sensitivity: f32,
    pub forward: Vector3<f32>,
    pub right: Vector3<f32>,
    frustum: [cgmath::Vector4<f32>; 6],
}

impl Camera {

    /// Creates a new camera with an initial position, yaw, and pitch values.
    ///
    /// # Arguments
    /// * `position` - A tuple containing three values representing the camera's position in 3D space.
    /// * `yaw` - The yaw angle for rotating the camera around the Y axis.
    /// * `pitch` - The pitch angle for rotating the camera around the X axis.
    ///
    /// # Returns
    /// Returns an instance of the camera.
    pub fn new(position: (f32, f32, f32), yaw: f32, pitch: f32) -> Self {
        let mut camera = Camera {
            id: ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            position: position.into(),
            yaw: Rad(yaw),
            pitch: Rad(pitch),
            roll: Rad(0.0),
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
            speed: 40.0,
            sensitivity: 0.4,
            forward: vec3(0.0, 0.0, -1.0),
            right: Vector3::zero(),
            frustum: [vec4(0.0, 0.0, 0.0, 0.0); 6],
        };

        camera.update_camera_vectors();
        
        camera
    }

    /// Returns the camera's ID.
    pub fn get_id(&self) -> u64 {
        self.id
    }

    /// Calculates the camera's view matrix based on the current yaw and pitch values.
    ///
    /// # Returns
    /// A view matrix that determines how objects will be rendered in relation to the camera.
    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }

    /// Updates the camera's view frustum planes from the current view-projection matrix.
    ///
    /// This method extracts six clipping planes (left, right, bottom, top, near, far) from
    /// the combined view-projection matrix and normalizes them. These planes are used for
    /// frustum culling, allowing efficient visibility checks against objects in the scene.
    pub fn update_frustum(&mut self) {
        let m = cgmath::Matrix4::from(self.view_proj);

        let planes = [
            m.row(3) + m.row(0), // Left
            m.row(3) - m.row(0), // Right
            m.row(3) + m.row(1), // Bottom
            m.row(3) - m.row(1), // Top
            m.row(3) + m.row(2), // Near
            m.row(3) - m.row(2), // Far
        ];

        self.frustum = planes.map(|v| {
            let normal = v.truncate();
            let length = normal.magnitude();
            v / length
        });
    }

    /// Checks if a bounding sphere is inside or intersects the camera's view frustum.
    ///
    /// # Parameters
    /// - `center`: The center of the sphere in world space.
    /// - `radius`: The radius of the sphere.
    ///
    /// # Returns
    /// - `true` if the sphere is at least partially inside the view frustum.
    /// - `false` if the sphere is completely outside and can be culled.
    ///
    /// The `radius` is scaled by 20.0 as a conservative threshold to account for object size variance.
    pub fn can_see(&self, center: Vector3<f32>, radius: f32) -> bool {
        for plane in &self.frustum {
            let distance = plane.truncate().dot(center) + plane.w;
            if distance < -radius {
                return false;
            }
        }
        true
    }

    /// Updates the camera's view and projection matrix.
    ///
    /// # Arguments
    /// * `projection` - The projection used to update the projection matrix.
    pub fn update_view_proj(&mut self, projection: &Projection) {
        self.view_position = self.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * self.calc_matrix()).into();
        self.update_frustum();
    }

    /// Returns the camera's uniform containing view and projection data.
    ///
    /// # Returns
    /// A uniform that contains the camera's position and matrices as a byte slice.
    pub fn get_uniform(&self) -> Box<[u8]> {
        bytemuck::cast_slice(&[
            self.view_position[0],
            self.view_position[1],
            self.view_position[2],
            self.view_position[3],
            self.view_proj[0][0], self.view_proj[0][1], self.view_proj[0][2], self.view_proj[0][3],
            self.view_proj[1][0], self.view_proj[1][1], self.view_proj[1][2], self.view_proj[1][3],
            self.view_proj[2][0], self.view_proj[2][1], self.view_proj[2][2], self.view_proj[2][3],
            self.view_proj[3][0], self.view_proj[3][1], self.view_proj[3][2], self.view_proj[3][3],
        ]).into()
    }

    pub fn set_position(&mut self, position: Point3<f32>) {
        self.position = position;
    }

    pub fn set_rotation(&mut self, yaw: Rad<f32>, pitch: Rad<f32>, roll: Rad<f32>) {
        self.yaw = yaw;
        self.pitch = pitch;
        self.roll = roll;
        self.update_camera_vectors();
    }

    /// Moves the camera forward based on the current orientation.
    ///
    /// # Arguments
    /// * `dt` - The time delta used to move the camera.
    pub fn move_forward(&mut self, dt: f32) {
        self.position += self.forward * self.speed * dt;
    }

    /// Moves the camera backward based on the current orientation.
    ///
    /// # Arguments
    /// * `dt` - The time delta used to move the camera.
    pub fn move_backward(&mut self, dt: f32) {
        self.position += -self.forward * self.speed * dt;
    }

    /// Moves the camera left based on the current orientation.
    ///
    /// # Arguments
    /// * `dt` - The time delta used to move the camera.
    pub fn move_left(&mut self, dt: f32) {
        self.position += -self.right * self.speed * dt;
    }

    /// Moves the camera right based on the current orientation.
    ///
    /// # Arguments
    /// * `dt` - The time delta used to move the camera.
    pub fn move_right(&mut self, dt: f32) {
        self.position += self.right * self.speed * dt;
    }

    /// Rotates the camera around the Y axis (yaw) by specified angle in radians
    /// 
    /// # Arguments
    /// * `angle` - Angle in radians to rotate
    pub fn rotate_yaw(&mut self, angle: f32) {
        self.yaw += Rad(angle);
        self.yaw = Rad(self.yaw.0.rem_euclid(2.0 * std::f32::consts::PI));
    }

    /// Rotates the camera around the X axis (pitch) by specified angle in radians
    /// 
    /// # Arguments
    /// * `angle` - Angle in radians to rotate
    /// 
    /// # Note
    /// Automatically clamps pitch between -89° and +89° to prevent gimbal lock
    pub fn rotate_pitch(&mut self, angle: f32) {
        self.pitch += Rad(angle);
    }

    /// Rotates the camera around the Z axis (roll) by specified angle in radians
    /// 
    /// # Arguments
    /// * `angle` - Angle in radians to rotate
    /// 
    /// # Note
    /// This is less common in FPS-style cameras but useful for flight simulators
    pub fn rotate_roll(&mut self, angle: f32) {
        self.roll += Rad(angle);
        self.roll = Rad(self.roll.0.rem_euclid(2.0 * std::f32::consts::PI));
    }

    /// Updates the camera's orientation based on the current yaw and pitch values.
    pub fn update_camera_vectors(&mut self) {
        if self.pitch < -Rad(SAFE_FRAC_PI_2) {
            self.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if self.pitch > Rad(SAFE_FRAC_PI_2) {
            self.pitch = Rad(SAFE_FRAC_PI_2);
        }

        self.forward = Vector3 {
            x: self.yaw.cos() * self.pitch.cos(),
            y: self.pitch.sin(),
            z: self.yaw.sin() * self.pitch.cos(),
        }.normalize();

        self.right = self.forward.cross(Vector3::unit_y()).normalize();
    }
}
