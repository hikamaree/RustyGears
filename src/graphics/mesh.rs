#![allow(deref_nullptr)]

use std::ffi::CString;
use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr;

use cgmath::{ Vector4, Vector3, Vector2, Point3 };
use cgmath::prelude::*;
use gl;

use super::shader::Shader;
use crate::collision_box::*;

#[repr(C)]
#[derive(Clone)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coords: Vector2<f32>,
    pub tangent: Vector3<f32>,
    pub bitangent: Vector3<f32>,
    pub color: Vector4<f32>
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: Vector3::zero(),
            normal: Vector3::zero(),
            tex_coords: Vector2::zero(),
            tangent: Vector3::zero(),
            bitangent: Vector3::zero(),
            color: Vector4::zero(),
        }
    }
}

#[derive(Clone)]
pub struct Textures {
    pub id: u32,
    pub type_: String,
    pub path: String,
}

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<Textures>,
    pub vao: u32,

    vbo: u32,
    ebo: u32,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, textures: Vec<Textures>) -> Mesh {
        let mut mesh = Mesh {
            vertices, indices, textures,
            vao: 0, vbo: 0, ebo: 0
        };

        unsafe { mesh.setup_mesh() }
        mesh
    }

    pub unsafe fn draw(&self, shader: &Shader) {
        let mut diffuse_nr  = 0;
        let mut specular_nr = 0;
        let mut normal_nr   = 0;
        let mut height_nr   = 0;
        for (i, texture) in self.textures.iter().enumerate() {
            gl::ActiveTexture(gl::TEXTURE0 + i as u32);
            let name = &texture.type_;
            let number = match name.as_str() {
                "texture_diffuse" => {
                    diffuse_nr += 1;
                    diffuse_nr
                },
                "texture_specular" => {
                    specular_nr += 1;
                    specular_nr
                }
                "texture_normal" => {
                    normal_nr += 1;
                    normal_nr
                }
                "texture_height" => {
                    height_nr += 1;
                    height_nr
                }
                _ => panic!("unknown texture type")
            };
            let sampler = CString::new(format!("{}{}", name, number)).unwrap();
            gl::Uniform1i(gl::GetUniformLocation(shader.id, sampler.as_ptr()), i as i32);
            gl::BindTexture(gl::TEXTURE_2D, texture.id);
        }

        gl::BindVertexArray(self.vao);
        gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, ptr::null());
        gl::BindVertexArray(0);

        gl::ActiveTexture(gl::TEXTURE0);
    }

    unsafe fn setup_mesh(&mut self) {
        gl::GenVertexArrays(1, &mut self.vao);
        gl::GenBuffers(1, &mut self.vbo);
        gl::GenBuffers(1, &mut self.ebo);

        gl::BindVertexArray(self.vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        let size = (self.vertices.len() * size_of::<Vertex>()) as isize;
        let data = &self.vertices[0] as *const Vertex as *const c_void;
        gl::BufferData(gl::ARRAY_BUFFER, size, data, gl::STATIC_DRAW);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
        let size = (self.indices.len() * size_of::<u32>()) as isize;
        let data = &self.indices[0] as *const u32 as *const c_void;
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size, data, gl::STATIC_DRAW);

        let size = size_of::<Vertex>() as i32;
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, position) as *const c_void);
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, normal) as *const c_void);
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, tex_coords) as *const c_void);
        gl::EnableVertexAttribArray(3);
        gl::VertexAttribPointer(3, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, tangent) as *const c_void);
        gl::EnableVertexAttribArray(4);
        gl::VertexAttribPointer(4, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, bitangent) as *const c_void);
        gl::EnableVertexAttribArray(5);
        gl::VertexAttribPointer(5, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, color) as *const c_void);

        gl::BindVertexArray(0);
    }

    pub fn calculate_sphere(&self) -> Sphere {
        if self.vertices.is_empty() {
            panic!("Cannot calculate sphere for an empty mesh.");
        }

        let center = self.vertices.iter().fold(Vector3::zero(), |acc, v| acc + v.position) / (self.vertices.len() as f32);

        let radius = self.vertices.iter()
            .map(|v| (v.position - center).magnitude())
            .fold(0.0, f32::max);

        Sphere {
            center: Point3::from_vec(center),
            radius,
        }
    }

    pub fn calculate_bounding_box(&self) -> BoundingBox {
        if self.vertices.is_empty() {
            panic!("Cannot calculate bounding box for an empty mesh.");
        }

        let mut min = Vector3::from_value(f32::MAX);
        let mut max = Vector3::from_value(f32::MIN);

        for vertex in &self.vertices {
            min.x = min.x.min(vertex.position.x);
            min.y = min.y.min(vertex.position.y);
            min.z = min.z.min(vertex.position.z);

            max.x = max.x.max(vertex.position.x);
            max.y = max.y.max(vertex.position.y);
            max.z = max.z.max(vertex.position.z);
        }

        BoundingBox { 
            min: Point3::from_vec(min),
            max: Point3::from_vec(max)
        }
    }
}
