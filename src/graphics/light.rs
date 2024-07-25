use cgmath::*;
use super::shader::*;
use crate::*;
use std::ffi::CString;
use std::ffi::CStr;

pub struct AmbientLight {
    pub color: Vector3<f32>,
    pub intensity: f32,
}

impl AmbientLight {
    pub fn new(color: Vector3<f32>, intensity: f32) -> Self {
        Self { color, intensity }
    }

    pub fn apply(&self, shader: &Shader) {
        shader.setVector3(c_str!("ambientLight.color"), &self.color);
        shader.setFloat(c_str!("ambientLight.intensity"), self.intensity);
    }
}

pub struct LightSource {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
    pub intensity: f32,
}

impl LightSource {
    pub fn new(position: Vector3<f32>, color: Vector3<f32>, intensity: f32) -> Self {
        Self { position, color, intensity }
    }

    pub unsafe fn apply(&self, shader: &Shader, index: usize) {
        let position_uniform = CString::new(format!("lightSources[{}].position", index)).unwrap();
        let color_uniform = CString::new(format!("lightSources[{}].color", index)).unwrap();
        let intensity_uniform = CString::new(format!("lightSources[{}].intensity", index)).unwrap();

        shader.setVector3(&position_uniform, &self.position);
        shader.setVector3(&color_uniform, &self.color);
        shader.setFloat(&intensity_uniform, self.intensity);
    }

    pub fn create_light_space_matrix(&self) -> Matrix4<f32> {
        let near_plane = 1.0;
        let far_plane = 7.5;
        let light_projection = ortho(-10.0, 10.0, -10.0, 10.0, near_plane, far_plane);
        let light_view = Matrix4::look_at_rh(Point3::from_vec(self.position), Point3::new(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));
        light_projection * light_view
    }
}

pub struct ShadowMap {
    pub fbo: u32,
    pub textures: Vec<u32>,
    pub width: i32,
    pub height: i32
}

impl ShadowMap {
    pub fn new(num_light_sources: usize) -> ShadowMap {
        let mut shadow_map = ShadowMap {
            fbo: 0,
            textures: Vec::with_capacity(num_light_sources),
            width: 1024,
            height: 1024,
        };

        unsafe {
            if num_light_sources > 0 {
                gl::GenFramebuffers(1, &mut shadow_map.fbo);

                for _ in 0..num_light_sources {
                    let mut texture: u32 = 0;
                    gl::GenTextures(1, &mut texture);
                    gl::BindTexture(gl::TEXTURE_2D, texture);
                    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::DEPTH_COMPONENT as i32, shadow_map.width,
                        shadow_map.height, 0, gl::DEPTH_COMPONENT, gl::FLOAT, std::ptr::null(),);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as i32);
                    let border_color: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
                    gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, border_color.as_ptr());

                    shadow_map.textures.push(texture);
                }

                gl::BindFramebuffer(gl::FRAMEBUFFER, shadow_map.fbo);
                gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, shadow_map.textures[0], 0);
                gl::DrawBuffer(gl::NONE);
                gl::ReadBuffer(gl::NONE);
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            }
        }
        shadow_map
    }
}
