// SPDX-License-Identifier: GPL-3.0-or-later
//
// This file is part of Rusty Gears.
//
// Rusty Gears is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Rusty Gears is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::GameView;
use crate::Material;
use std::sync::Arc;

use cgmath::vec2;
use cgmath::vec3;
use cgmath::InnerSpace;

use wgpu::util::DeviceExt;

use crate::ModelVertex;
use crate::Mesh;
use crate::Model;
use crate::RenderTag;
use crate::BoundingSphere;


#[derive(Debug, Clone)]
pub struct Landscape {
    pub heightmap: Vec<f32>,
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
}

impl Landscape {
    pub fn from_heightmap(path: &str, cell_size: f32, max_height: f32) -> Result<Landscape, Box<dyn std::error::Error>> {
        let img = image::open(path)?.to_luma8(); 

        let (width, height) = img.dimensions();

        let heightmap: Vec<f32> = img
            .pixels()
            .map(|p| (p[0] as f32 / 255.0) * max_height)
            .collect();

        Ok(Landscape {
            heightmap,
            width,
            height,
            cell_size,
        })
    }

    pub fn generate_model(&mut self, game: &GameView) -> Result<Model, String> {
        let width = self.width as usize;
        let height = self.height as usize;

        let mut vertices = Vec::with_capacity(width * height);

        for z in 0..height {
            for x in 0..width {
                let y = self.heightmap[z * width + x];
                let position = [
                    (x as f32 - (width as f32 - 1.0) / 2.0) * self.cell_size,
                    y,
                    (z as f32 - (height as f32 - 1.0) / 2.0) * self.cell_size,
                ];
                let tex_coords = [
                    x as f32 / (width - 1) as f32,
                    z as f32 / (height - 1) as f32,
                ];
                vertices.push(ModelVertex {
                    position,
                    tex_coords,
                    normal: [0.0; 3],
                    tangent: [0.0; 3],
                    bitangent: [0.0; 3],
                });
            }
        }

        let mut indices = Vec::new();
        for z in 0..(height - 1) {
            for x in 0..(width - 1) {
                let i0 = (z * width + x) as u32;
                let i1 = i0 + 1;
                let i2 = i0 + width as u32;
                let i3 = i2 + 1;

                indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
            }
        }

        let mut normals = vec![vec3(0.0, 0.0, 0.0); vertices.len()];
        let mut tangents = vec![vec3(0.0, 0.0, 0.0); vertices.len()];
        let mut bitangents = vec![vec3(0.0, 0.0, 0.0); vertices.len()];

        for tri in indices.chunks(3) {
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;

            let v0 = &vertices[i0];
            let v1 = &vertices[i1];
            let v2 = &vertices[i2];

            let p0 = vec3(v0.position[0], v0.position[1], v0.position[2]);
            let p1 = vec3(v1.position[0], v1.position[1], v1.position[2]);
            let p2 = vec3(v2.position[0], v2.position[1], v2.position[2]);

            let uv0 = vec2(v0.tex_coords[0], v0.tex_coords[1]);
            let uv1 = vec2(v1.tex_coords[0], v1.tex_coords[1]);
            let uv2 = vec2(v2.tex_coords[0], v2.tex_coords[1]);

            let delta_pos1 = p1 - p0;
            let delta_pos2 = p2 - p0;

            let delta_uv1 = uv1 - uv0;
            let delta_uv2 = uv2 - uv0;

            let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
            let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
            let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;

            let normal = delta_pos1.cross(delta_pos2).normalize();

            normals[i0] += normal;
            normals[i1] += normal;
            normals[i2] += normal;

            tangents[i0] += tangent;
            tangents[i1] += tangent;
            tangents[i2] += tangent;

            bitangents[i0] += bitangent;
            bitangents[i1] += bitangent;
            bitangents[i2] += bitangent;
        }

        for (i, v) in vertices.iter_mut().enumerate() {
            v.normal = normals[i].normalize().into();
            v.tangent = tangents[i].normalize().into();
            v.bitangent = bitangents[i].normalize().into();
        }

        let center = vec3(0.0, 0.0, 0.0);

        let mut max_radius = 0.0;
        for v in &vertices {
            let pos = vec3(v.position[0], v.position[1], v.position[2]);
            let dist = (pos - center).magnitude();
            if dist > max_radius {
                max_radius = dist;
            }
        }

        let bounding_sphere = BoundingSphere {
            center,
            radius: max_radius,
        };

        let Some(graphics) = game.get::<crate::Graphics>() else {
            return Err("Failed to get Graphics component".to_string());
        };

        let vertex_buffer = graphics.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("landscape_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = graphics.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("landscape_index_buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mesh = Mesh {
            name: "landscape_mesh".to_string(),
            vertex_buffer: Arc::new(vertex_buffer),
            index_buffer: Arc::new(index_buffer),
            num_elements: indices.len() as u32,
            material: 0,
            bounding_sphere,
            render_tag: RenderTag::Opaque,
        };

        let Some(texture_layout) = graphics.bind_group_layouts.get(&crate::BindGroupLayoutKey::Texture) else {
            return Err("Missing texture bind group layout".into());
        };

        let material = Material::default(&graphics.device, &graphics.queue, texture_layout);

        Ok(Model {
            meshes: vec![mesh.into()],
            materials: vec![material],
        })
    }
}
