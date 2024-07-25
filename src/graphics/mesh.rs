#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(deref_nullptr)]

use std::ffi::CString;
use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr;

use cgmath::{ Vector4, Vector3, Vector2 };
use cgmath::prelude::*;
use gl;

use super::shader::Shader;

#[repr(C)]
pub struct Vertex {
    pub Position: Vector3<f32>,
    pub Normal: Vector3<f32>,
    pub TexCoords: Vector2<f32>,
    pub Tangent: Vector3<f32>,
    pub Bitangent: Vector3<f32>,
    pub Color: Vector4<f32>
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            Position: Vector3::zero(),
            Normal: Vector3::zero(),
            TexCoords: Vector2::zero(),
            Tangent: Vector3::zero(),
            Bitangent: Vector3::zero(),
            Color: Vector4::zero(),
        }
    }
}

#[derive(Clone)]
pub struct Texture {
    pub id: u32,
    pub type_: String,
    pub path: String,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<Texture>,
    pub VAO: u32,

    VBO: u32,
    EBO: u32,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, textures: Vec<Texture>) -> Mesh {
        let mut mesh = Mesh {
            vertices, indices, textures,
            VAO: 0, VBO: 0, EBO: 0
        };

        unsafe { mesh.setupMesh() }
        mesh
    }

    pub unsafe fn Draw(&self, shader: &Shader) {
        let mut diffuseNr  = 0;
        let mut specularNr = 0;
        let mut normalNr   = 0;
        let mut heightNr   = 0;
        for (i, texture) in self.textures.iter().enumerate() {
            gl::ActiveTexture(gl::TEXTURE0 + i as u32);
            let name = &texture.type_;
            let number = match name.as_str() {
                "texture_diffuse" => {
                    diffuseNr += 1;
                    diffuseNr
                },
                "texture_specular" => {
                    specularNr += 1;
                    specularNr
                }
                "texture_normal" => {
                    normalNr += 1;
                    normalNr
                }
                "texture_height" => {
                    heightNr += 1;
                    heightNr
                }
                _ => panic!("unknown texture type")
            };
            let sampler = CString::new(format!("{}{}", name, number)).unwrap();
            gl::Uniform1i(gl::GetUniformLocation(shader.ID, sampler.as_ptr()), i as i32);
            gl::BindTexture(gl::TEXTURE_2D, texture.id);
        }

        gl::BindVertexArray(self.VAO);
        gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, ptr::null());
        gl::BindVertexArray(0);

        gl::ActiveTexture(gl::TEXTURE0);
    }

    unsafe fn setupMesh(&mut self) {
        gl::GenVertexArrays(1, &mut self.VAO);
        gl::GenBuffers(1, &mut self.VBO);
        gl::GenBuffers(1, &mut self.EBO);

        gl::BindVertexArray(self.VAO);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.VBO);
        let size = (self.vertices.len() * size_of::<Vertex>()) as isize;
        let data = &self.vertices[0] as *const Vertex as *const c_void;
        gl::BufferData(gl::ARRAY_BUFFER, size, data, gl::STATIC_DRAW);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.EBO);
        let size = (self.indices.len() * size_of::<u32>()) as isize;
        let data = &self.indices[0] as *const u32 as *const c_void;
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size, data, gl::STATIC_DRAW);

        let size = size_of::<Vertex>() as i32;
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, Position) as *const c_void);
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, Normal) as *const c_void);
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, TexCoords) as *const c_void);
        gl::EnableVertexAttribArray(3);
        gl::VertexAttribPointer(3, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, Tangent) as *const c_void);
        gl::EnableVertexAttribArray(4);
        gl::VertexAttribPointer(4, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, Bitangent) as *const c_void);
        gl::EnableVertexAttribArray(5);
        gl::VertexAttribPointer(5, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, Color) as *const c_void);

        gl::BindVertexArray(0);
    }
}
