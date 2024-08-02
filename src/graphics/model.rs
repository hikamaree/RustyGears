use std::path::Path;
use std::ffi::CStr;
use core::cell::RefCell;
use std::rc::Rc;

use cgmath::{vec2, vec3, vec4, Vector3, Matrix4};
use tobj;

use super::mesh::{ Mesh, Texture, Vertex };
use super::shader::Shader;
use super::utils::*;
use super::scene::*;
use crate::physics::*;

#[derive(Clone)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub textures_loaded: Vec<Texture>,
    directory: String,
    pub position: Vector3<f32>
}

pub type ModelRef = Rc<RefCell<Model>>;

impl Model {
    fn default() -> Self {
        Self {
            meshes: Vec::default(),
            textures_loaded: Vec::default(),
            directory: String::default(),
            position: vec3(0.0, 0.0, 0.0)
        }
    }

    pub fn add_physics(&mut self, scene: &mut Scene, mass: f32) {
        let body = RigidBody::from_model_with_bounding_boxes(self, mass);
        scene.physics_world.add_body(body);
    }

    fn new(path: &str, position: Vector3<f32>) -> Model {
        let mut model = Model::default();
        model.load_model(path);
        model.position = position;
        model
    }

    pub fn create(path: &str, position: Vector3<f32>) -> ModelRef {
        Rc::new(RefCell::new(Model::new(path, position)))
    }

    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
    }

    pub fn move_position(&mut self, offset: Vector3<f32>) {
        self.position += offset;
    }

    pub fn draw(&self, shader: &Shader) {
        let mmodel = Matrix4::from_translation(self.position);
        shader.set_mat4(c_str!("model"), &mmodel);

        unsafe {
            for mesh in &self.meshes {
                mesh.draw(shader);
            }
        }
    }

    fn load_model(&mut self, path: &str) {
        let path = Path::new(path);

        self.directory = path.parent().unwrap_or_else(|| Path::new("")).to_str().unwrap().into();
        let obj = tobj::load_obj(path);

        let (models, materials) = obj.unwrap();
        for model in models {
            let mesh = &model.mesh;
            let num_vertices = mesh.positions.len() / 3;

            let mut vertices: Vec<Vertex> = Vec::with_capacity(num_vertices);
            let indices: Vec<u32> = mesh.indices.clone();

            let (p, n, t) = (&mesh.positions, &mesh.normals, &mesh.texcoords);
            for i in 0..num_vertices {
                let position = vec3(p[i*3], p[i*3+1], p[i*3+2]);

                let normal = if n.len() > i*3+2 {
                    vec3(n[i*3], n[i*3+1], n[i*3+2])
                } else {
                    vec3(0.0, 0.0, 0.0)
                };

                let texcoords = if t.len() > i*2+1 {
                    vec2(t[i*2], t[i*2+1])
                } else {
                    vec2(0.0, 0.0)
                };

                vertices.push(Vertex {
                    position,
                    normal,
                    tex_coords: texcoords,
                    color: vec4(1.0, 1.0, 1.0, 1.0),
                    ..Vertex::default()
                });
            }

            let mut textures = Vec::new();
            if let Some(material_id) = mesh.material_id {
                let material = &materials[material_id];

                if !material.diffuse_texture.is_empty() {
                    let texture = self.load_material_texture(&material.diffuse_texture, "texture_diffuse");
                    textures.push(texture);
                } else {
                    for vertex in vertices.iter_mut() {
                        if material.diffuse.len() > 3 {
                            vertex.color = vec4(material.diffuse[0], material.diffuse[1], material.diffuse[2], material.diffuse[3]);
                        } else {
                            vertex.color = vec4(material.diffuse[0], material.diffuse[1], material.diffuse[2], 1.0);
                        }
                    }
                }
                if !material.specular_texture.is_empty() {
                    let texture = self.load_material_texture(&material.specular_texture, "texture_specular");
                    textures.push(texture);
                }
                if !material.normal_texture.is_empty() {
                    let texture = self.load_material_texture(&material.normal_texture, "texture_normal");
                    textures.push(texture);
                }
            }

            self.meshes.push(Mesh::new(vertices, indices, textures));
        }

    }

    fn load_material_texture(&mut self, path: &str, type_name: &str) -> Texture {
        {
            let texture = self.textures_loaded.iter().find(|t| t.path == path);
            if let Some(texture) = texture {
                return texture.clone();
            }
        }

        let texture = Texture {
            id: load_texture(format!("{}/{}", self.directory, path).as_str()),
            type_: type_name.into(),
            path: path.into()
        };
        self.textures_loaded.push(texture.clone());
        texture
    }
}
