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

use crate::dummy_texture;
use crate::Model3d;
use crate::RenderTag;
use crate::InstanceRaw;
use std::sync::Arc;
use crate::Texture;
use std::ops::Range;


#[derive(Debug)]
pub struct MeshRenderRange {
    pub mesh_index: usize,
    pub visible_instance_ranges: Vec<Range<u32>>,
}

#[derive(Debug)]
pub struct ModelRenderData {
    pub instance_data: Arc<[InstanceRaw]>,
    pub mesh_ranges: Vec<MeshRenderRange>,
    pub model3d: Model3d,
    pub lod_index: usize,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    pub _padding: u32,
    pub color: [f32; 3],
    pub _padding2: u32,
}


pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub diffuse_texture: Option<Texture>,
    pub normal_texture: Option<Texture>,
    pub dissolve_texture: Option<Texture>,
    pub bind_group: wgpu::BindGroup,
    pub dissolve: f32,
}

impl Material {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: &str,
        diffuse_texture: Option<Texture>,
        normal_texture: Option<Texture>,
        dissolve_texture: Option<Texture>,
        dissolve: f32,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let dummy = dummy_texture(device, queue);

        let mut entries = Vec::new();

        if let Some(ref diffuse_texture) = diffuse_texture {
            entries.push(wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
            });
        } else {
            entries.push(wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&dummy.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&dummy.sampler),
            });
        }

        if let Some(ref normal_texture) = diffuse_texture {
            entries.push(wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&normal_texture.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
            });
        } else {
            entries.push(wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&dummy.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&dummy.sampler),
            });
        }

        if let Some(ref dissolve_texture) = dissolve_texture {
            entries.push(wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&dissolve_texture.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(&dissolve_texture.sampler),
            });
        } else {
            entries.push(wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&dummy.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(&dummy.sampler),
            });
        }

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &entries,
            label: Some(name),
        });

        Self {
            name: String::from(name),
            diffuse_texture,
            normal_texture,
            dissolve_texture,
            dissolve,
            bind_group,
        }
    }

    pub fn has_transparency_texture(&self) -> bool {
        self.dissolve_texture.is_some()
    }

    pub fn use_weighted_blended(&self) -> bool {
        // TODO
        false
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingSphere {
    pub center: cgmath::Vector3<f32>,
    pub radius: f32,
}

#[derive(Clone)]
pub struct Mesh {
    pub name: String,
    pub vertex_buffer: Arc<wgpu::Buffer>,
    pub index_buffer: Arc<wgpu::Buffer>,
    pub num_elements: u32,
    pub material: usize,
    pub bounding_sphere: BoundingSphere,
    pub render_tag: RenderTag,
}

#[derive(Clone)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

pub trait DrawModel<'a> {
    #[allow(unused)]
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        ranges: &MeshRenderRange
    ) -> u32;

    #[allow(unused)]
    fn draw_model(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        ranges: MeshRenderRange
    );

    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        ranges: &Vec<MeshRenderRange>
    ) -> u32;
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where 'b: 'a {
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_mesh_instanced(
            mesh,
            material,
            camera_bind_group,
            light_bind_group,
            &MeshRenderRange {
                mesh_index: 0,
                visible_instance_ranges: vec![0..1]
            });
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        ranges: &MeshRenderRange,
    ) -> u32 {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.set_bind_group(2, light_bind_group, &[]);

        for instance_range in &ranges.visible_instance_ranges {
            self.draw_indexed(0..mesh.num_elements, 0, instance_range.clone());
        }

        let triangle_count_per_instance = mesh.num_elements / 3;
        let instances_visible: u32 = ranges
            .visible_instance_ranges
            .iter()
            .map(|r| r.end - r.start)
            .sum();

        triangle_count_per_instance * instances_visible
    }

    fn draw_model(
        &mut self,
        model: &'b Model,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
        ranges: MeshRenderRange,
    ) {
        self.draw_model_instanced(model, camera_bind_group, light_bind_group, &vec![ranges]);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
        ranges: &Vec<MeshRenderRange>
    ) -> u32 {
        let mut total_triangles = 0;

        for mesh_range in ranges {
            let mesh = &model.meshes[mesh_range.mesh_index];
            let material = &model.materials[mesh.material];

            total_triangles += self.draw_mesh_instanced(
                mesh,
                material,
                camera_bind_group,
                light_bind_group,
                mesh_range,
            );
        }

        total_triangles
    }
}
