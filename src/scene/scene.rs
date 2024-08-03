use crate::*;
use std::ffi::CString;
use std::ffi::CStr;
use core::cell::RefCell;
use std::rc::Rc;

pub struct Scene {
    entities: Vec<Entity>,
    models: Vec<ModelRef>,
    ambient_light: Option<AmbientLight>,
    light_sources: Vec<LightSource>,
    light_space_matrices: Vec<Matrix4<f32>>,
    shadow_map: ShadowMap,
    fog: Option<Fog>,
    shader: Option<Shader>,
    depth_shader: Option<Shader>,
    camera: CameraRef,
    physics_world: PhysicsWorld,
}

pub type SceneRef = Rc<RefCell<Scene>>;

impl Scene {
    fn new() -> Self {
        Scene {
            entities: Vec::new(),
            models: Vec::new(),
            ambient_light: None,
            light_sources: Vec::new(),
            light_space_matrices: Vec::new(),
            shadow_map: ShadowMap::new(0),
            fog: None,
            shader: None,
            depth_shader: None,
            camera: Camera::create(),
            physics_world: PhysicsWorld::new(Vector3::new(0.0, -10.0, 0.0)),
        }
    }

    pub fn create() -> SceneRef {
        Rc::new(RefCell::new(Scene::new()))
    }

    pub fn set_shader(&mut self, shader: Shader) {
        self.shader = Some(shader);
    }

    pub fn set_depth_shader(&mut self, shader: Shader) {
        self.depth_shader = Some(shader);
    }

    pub(super) fn set_camera(&mut self, camera: CameraRef) {
        self.camera = camera;
    }

    pub(super) fn add_entity(&mut self, entity: Entity) {
        entity.set_physics(&mut self.physics_world);
        self.entities.push(entity);
    }

    pub(super) fn add_model(&mut self, model: ModelRef) {
        self.models.push(model);
    }

    pub(super) fn set_ambient_light(&mut self, ambient_light: AmbientLight) {
        self.ambient_light = Some(ambient_light);
    }

    pub(super) fn set_fog(&mut self, fog: Fog) {
        self.fog = Some(fog);
    }

    pub fn add<T: SceneItem>(&mut self, item: T) {
        item.add_to_scene(self);
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
            model.borrow_mut().draw(shader);
        }

        for entity in &self.entities {
            entity.draw(shader);
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

    pub fn update_scene(&mut self, delta_time: f32) {
        self.physics_world.update(delta_time);

        for entity in &mut self.entities {
            entity.update();
        }
    }

    pub fn render_depth_map(&mut self) {
        self.update_light_space_matrices();
        if let Some(depth_shader) = &self.depth_shader {
            for (i, light_space_matrix) in self.light_space_matrices.iter().enumerate() {
                unsafe {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, self.shadow_map.fbo);
                    gl::Viewport(0, 0, 1024, 1024);
                    gl::Clear(gl::DEPTH_BUFFER_BIT);
                    gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, self.shadow_map.textures[i], 0);
                }
                depth_shader.use_program();
                depth_shader.set_mat4(c_str!("lightSpaceMatrix"), light_space_matrix);

                self.draw(&depth_shader);
            }
        }
    }

    pub fn render(&self, (width, height): (u32, u32)) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Viewport(0, 0, width as i32, height as i32);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        if let Some(shader) = &self.shader {
            shader.use_program();

            for (i, light_space_matrix) in self.light_space_matrices.iter().enumerate() {
                let uniform_name = CString::new(format!("lightSpaceMatrices[{}]", i)).unwrap();
                shader.set_mat4(&uniform_name, light_space_matrix);

                let uniform_name = CString::new(format!("shadowMaps[{}]", i)).unwrap();
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE1 + i as u32);
                    gl::BindTexture(gl::TEXTURE_2D, self.shadow_map.textures[i]);
                }
                shader.set_int(&uniform_name, 1 + i as i32);
            }

            let camera = self.camera.borrow();

            let projection: Matrix4<f32> = perspective(Deg(camera.zoom), width as f32 / height as f32, 0.1, 100.0);
            shader.set_mat4(c_str!("projection"), &projection);

            let view = camera.get_view_matrix();
            shader.set_mat4(c_str!("view"), &view);

            shader.set_vector3(c_str!("cameraPosition"), &camera.position.to_vec());
            shader.set_int(c_str!("lightSourceNum"), self.light_sources.len() as i32);

            self.apply_lights(shader);
            self.draw(shader);
        }
    }
}
