use std::path::Path;
use std::ffi::CStr;
use core::cell::RefCell;
use std::rc::Rc;

use cgmath::{
    vec2,
    vec3,
    vec4,
    Vector3,
    Matrix4,
    Quaternion,
};

use tobj;

use super::{
    Mesh,
    Texture,
    Vertex,
    Shader,
    load_texture,
};

#[derive(Clone)]
pub struct ModelController {
    pub meshes: Vec<Mesh>,
    pub textures_loaded: Vec<Texture>,
    directory: String,
}

type ModelRef = Rc<RefCell<ModelController>>;

impl ModelController {
    fn new(path: &str) -> ModelController {
        let mut model = ModelController {
            meshes: Vec::default(),
            textures_loaded: Vec::default(),
            directory: String::default(),
        };
        model.load_model(path);
        model
    }

    pub fn draw(&self, shader: &Shader, position: Vector3<f32>, rotation: Quaternion<f32>) {
        let translation_matrix = Matrix4::from_translation(position);
        let rotation_matrix = Matrix4::from(rotation);
        let model_matrix = translation_matrix * rotation_matrix;
        shader.set_mat4(c_str!("model"), &model_matrix);

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
                        vertex.color = vec4(material.diffuse[0], material.diffuse[1], material.diffuse[2], 1.0);
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

#[derive(Clone)]
pub struct Model {
    pub(crate)model: ModelRef,
}

impl Model {
    pub fn open(path: &str) -> Self {
        Self {
            model: Rc::new(RefCell::new(ModelController::new(path)))
        }
    }

    pub fn draw(&self, shader: &Shader, position: Vector3<f32>, rotation: Quaternion<f32>) {
        self.model.borrow().draw(shader, position, rotation);
    }

    pub(crate) fn get_meshes(&self) -> Vec<Mesh> {
        self.model.borrow().meshes.clone()
    }
}
