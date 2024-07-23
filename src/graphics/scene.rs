use super::*;
use std::ffi::CString;

pub struct Scene {
    pub models: Vec<Model>,
    pub ambient_light: Option<AmbientLight>,
    pub light_sources: Vec<LightSource>,
    pub light_space_matrices: Vec<Matrix4<f32>>,
    pub shadow_map: ShadowMap,
    pub fog: Option<Fog>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            models: Vec::new(),
            ambient_light: None,
            light_sources: Vec::new(),
            light_space_matrices: Vec::new(),
            shadow_map: ShadowMap::new(0),
            fog: None,
        }
    }

    pub fn add_model(&mut self, model: Model) {
        self.models.push(model);
    }

    pub fn set_ambient_light(&mut self, ambient_light: AmbientLight) {
        self.ambient_light = Some(ambient_light);
    }

    pub fn set_fog(&mut self, fog: Fog) {
        self.fog = Some(fog);
    }

    pub fn add_light_source(&mut self, light_source: LightSource) {
        self.light_sources.push(light_source);
        self.light_space_matrices.push(Matrix4::identity());
        self.shadow_map = ShadowMap::new(self.light_sources.len());
    }

    pub fn update_light_space_matrices(&mut self) {
        self.light_space_matrices.clear();
        for light_source in self.light_sources.iter() {
            self.light_space_matrices.push(light_source.create_light_space_matrix());
        }
    }

    pub fn draw(&self, shader: &Shader) {
        for model in &self.models {
            model.Draw(shader);
        }
    }

    pub fn apply_lights(&self, shader: &Shader) {
        unsafe {
            if let Some(ambient_light) = &self.ambient_light {
                ambient_light.apply(shader);
            }
            for (i, light_source) in self.light_sources.iter().enumerate() {
                light_source.apply(shader, i);
            }
            if let Some(fog) = &self.fog {
                fog.apply(shader);
            }
        }
    }

    pub fn render_depth_map(&mut self, depth_shader: &Shader) {
        self.update_light_space_matrices();
        unsafe {
            gl::Viewport(0, 0, 1024, 1024);
        }
        for (i, light_space_matrix) in self.light_space_matrices.iter().enumerate() {
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, self.shadow_map.fbo);
                gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, self.shadow_map.textures[i], 0);
                gl::Clear(gl::DEPTH_BUFFER_BIT);
            }
            depth_shader.useProgram();
            depth_shader.setMat4(c_str!("lightSpaceMatrix"), light_space_matrix);
            self.draw(depth_shader);

            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            }
        }
    }

    pub fn render(&self, shader: &Shader, camera: &Camera, (width, height): (u32, u32)) {
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        shader.useProgram();

        let projection: Matrix4<f32> = perspective(Deg(camera.Zoom), width as f32 / height as f32, 0.1, 100.0);
        shader.setMat4(c_str!("projection"), &projection);

        let view = camera.GetViewMatrix();
        shader.setMat4(c_str!("view"), &view);

        for (i, light_space_matrix) in self.light_space_matrices.iter().enumerate() {
            let uniform_name = CString::new(format!("lightSpaceMatrices[{}]", i)).unwrap();
            shader.setMat4(&uniform_name, light_space_matrix);

            let uniform_name = CString::new(format!("shadowMaps[{}]", i)).unwrap();
            unsafe {
                gl::ActiveTexture(gl::TEXTURE1 + i as u32);
                gl::BindTexture(gl::TEXTURE_2D, self.shadow_map.textures[i]);
            }
            shader.setInt(&uniform_name, 1 + i as i32);
        }

        self.apply_lights(shader);
        self.draw(shader);
    }
}
