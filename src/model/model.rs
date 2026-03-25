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

use cgmath::InnerSpace;
use std::path::Path;
use std::sync::Arc;
use wgpu::util::DeviceExt;

use super::{GpuResources, TextureLoader};
use crate::Material;
use crate::MaterialData;
use crate::MeshRenderRange;
use crate::ModelVertex;
use crate::RenderTag;
use crate::TextureSource;

#[derive(Debug, Clone, Copy)]
pub struct BoundingSphere {
    pub center: cgmath::Vector3<f32>,
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub struct MeshData {
    pub name: String,
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
    pub bounding_sphere: BoundingSphere,
    pub render_tag: RenderTag,
    pub material: usize,
}

impl MeshData {
    pub fn bounding_sphere(&self) -> BoundingSphere {
        self.bounding_sphere
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub data: Arc<MeshData>,
    pub vertex_buffer: Arc<wgpu::Buffer>,
    pub index_buffer: Arc<wgpu::Buffer>,
    pub num_elements: u32,
    pub material: usize,
}

#[derive(Debug, Default, Clone)]
pub struct Model {
    pub meshes: Vec<Arc<Mesh>>,
    pub materials: Vec<Arc<Material>>,
}

#[derive(Debug, Default, Clone)]
pub struct ModelData {
    pub meshes: Vec<MeshData>,
    pub materials: Vec<MaterialData>,
}

impl ModelData {
    pub fn from_obj(path: &str) -> Result<Self, String> {
        let path = Path::new("resources").join(path);
        let base_dir = match path.parent() {
            Some(parent) => parent.to_path_buf(),
            None => return Err(format!("Invalid model path: {:?}", path)),
        };

        let obj_result = tobj::load_obj(
            &path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        );

        let (models, maybe_materials) = match obj_result {
            Ok(result) => result,
            Err(e) => return Err(format!("Failed to load OBJ model at {:?}: {}", path, e)),
        };

        let obj_materials = match maybe_materials {
            Ok(materials) => materials,
            Err(e) => {
                crate::log!(crate::LogKind::Error, "{}", e);
                Vec::new()
            }
        };

        let mut materials = Vec::new();

        if obj_materials.is_empty() {
            materials.push(MaterialData::new(String::from("default")));
        } else {
            for m in obj_materials {
                let diffuse = match &m.diffuse_texture {
                    Some(path) if !path.is_empty() => {
                        Some(TextureSource::File(base_dir.join(path)))
                    }
                    _ => m
                        .diffuse
                        .map(|c| TextureSource::Color([c[0], c[1], c[2], 1.0])),
                };

                let normal = m
                    .normal_texture
                    .as_ref()
                    .filter(|p| !p.is_empty())
                    .map(|p| TextureSource::File(base_dir.join(p)));

                let dissolve = m.dissolve.unwrap_or(1.0);

                let dissolve_source = if dissolve < 1.0 {
                    Some((TextureSource::Color([1.0, 1.0, 1.0, dissolve]), dissolve))
                } else if let Some(path) = &m.dissolve_texture {
                    if !path.is_empty() {
                        Some((TextureSource::File(base_dir.join(path)), dissolve))
                    } else {
                        None
                    }
                } else {
                    None
                };

                let material = MaterialData::new(String::from(&m.name))
                    .with_diffuse(diffuse)
                    .with_normal(normal);

                let material = if let Some((source, d)) = dissolve_source {
                    material.with_dissolve(Some(source), d)
                } else {
                    material.with_dissolve(None, dissolve)
                };

                materials.push(material);
            }
        }

        let meshes = models
            .into_iter()
            .map(|m| {
                let has_texcoords = !m.mesh.texcoords.is_empty();
                let has_normals = !m.mesh.normals.is_empty();

                let num_vertices = m.mesh.positions.len() / 3;

                let mut vertices = (0..num_vertices)
                    .map(|i| {
                        let tex_coords = if has_texcoords && i * 2 + 1 < m.mesh.texcoords.len() {
                            [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]]
                        } else {
                            [0.0, 0.0]
                        };

                        let normal = if has_normals && i * 3 + 2 < m.mesh.normals.len() {
                            [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ]
                        } else {
                            [0.0, 0.0, 0.0]
                        };

                        let position = if i * 3 + 2 < m.mesh.positions.len() {
                            [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ]
                        } else {
                            panic!("Model vertex position index out of bounds at index {}", i);
                        };

                        ModelVertex {
                            position,
                            tex_coords,
                            normal,
                            tangent: [0.0; 3],
                            bitangent: [0.0; 3],
                        }
                    })
                    .collect::<Vec<_>>();

                let indices = m.mesh.indices.clone();
                let mut triangles_included = vec![0; vertices.len()];

                for c in indices.chunks(3) {
                    let v0 = vertices[c[0] as usize];
                    let v1 = vertices[c[1] as usize];
                    let v2 = vertices[c[2] as usize];

                    let pos0: cgmath::Vector3<_> = v0.position.into();
                    let pos1: cgmath::Vector3<_> = v1.position.into();
                    let pos2: cgmath::Vector3<_> = v2.position.into();

                    let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
                    let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
                    let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

                    let delta_pos1 = pos1 - pos0;
                    let delta_pos2 = pos2 - pos0;

                    let delta_uv1 = uv1 - uv0;
                    let delta_uv2 = uv2 - uv0;

                    let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                    let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                    let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

                    vertices[c[0] as usize].tangent =
                        (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
                    vertices[c[1] as usize].tangent =
                        (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
                    vertices[c[2] as usize].tangent =
                        (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();
                    vertices[c[0] as usize].bitangent = (bitangent
                        + cgmath::Vector3::from(vertices[c[0] as usize].bitangent))
                    .into();
                    vertices[c[1] as usize].bitangent = (bitangent
                        + cgmath::Vector3::from(vertices[c[1] as usize].bitangent))
                    .into();
                    vertices[c[2] as usize].bitangent = (bitangent
                        + cgmath::Vector3::from(vertices[c[2] as usize].bitangent))
                    .into();

                    triangles_included[c[0] as usize] += 1;
                    triangles_included[c[1] as usize] += 1;
                    triangles_included[c[2] as usize] += 1;
                }

                for (i, n) in triangles_included.into_iter().enumerate() {
                    let denom = 1.0 / n as f32;
                    let v = &mut vertices[i];
                    v.tangent = (cgmath::Vector3::from(v.tangent) * denom).into();
                    v.bitangent = (cgmath::Vector3::from(v.bitangent) * denom).into();
                }

                let mut min = cgmath::Vector3::new(f32::MAX, f32::MAX, f32::MAX);
                let mut max = cgmath::Vector3::new(f32::MIN, f32::MIN, f32::MIN);

                for v in &vertices {
                    let p: cgmath::Vector3<_> = v.position.into();
                    min = min.zip(p, |a, b| a.min(b));
                    max = max.zip(p, |a, b| a.max(b));
                }

                let center = (min + max) * 0.5;
                let mut radius = 0.0;
                for v in &vertices {
                    let p: cgmath::Vector3<_> = v.position.into();
                    let dist = (p - center).magnitude();
                    if dist > radius {
                        radius = dist;
                    }
                }
                let bounding_sphere = BoundingSphere { center, radius };

                let material_index = m.mesh.material_id.unwrap_or(0);

                let material = &materials[material_index];
                let dissolve = material.dissolve_only;

                let render_tag = if dissolve < 1.0 || material.dissolve.is_some() {
                    if material.use_weighted_blended() {
                        RenderTag::WeightedTransparent
                    } else {
                        RenderTag::SortedTransparent
                    }
                } else {
                    RenderTag::Opaque
                };

                MeshData {
                    name: m.name,
                    vertices,
                    indices,
                    bounding_sphere,
                    render_tag,
                    material: material_index,
                }
            })
            .collect::<Vec<_>>();

        Ok(ModelData { meshes, materials })
    }

    pub fn into_gpu<R: GpuResources + TextureLoader>(
        self,
        resources: &R,
        layout: &wgpu::BindGroupLayout,
    ) -> Result<Model, String> {
        let device = resources.device();
        let queue = resources.queue();

        let materials = self
            .materials
            .into_iter()
            .map(|m| {
                let diffuse = m.diffuse.as_ref().and_then(|src| match src {
                    TextureSource::File(path) => resources.load_texture(path, false),
                    TextureSource::Color(color) => {
                        Some(resources.create_color_texture(*color, Some("diffuse"), false))
                    }
                });

                let normal = m.normal.as_ref().and_then(|src| match src {
                    TextureSource::File(path) => resources.load_texture(path, true),
                    TextureSource::Color(_) => None,
                });

                let dissolve = m.dissolve.clone();
                let (dissolve_texture, _dissolve) = dissolve
                    .map(|(src, d)| {
                        let tex = match src {
                            TextureSource::File(path) => resources.load_texture(&path, false),
                            TextureSource::Color(color) => {
                                Some(resources.create_color_texture(color, Some("alpha"), false))
                            }
                        };
                        (tex, d)
                    })
                    .unwrap_or((None, 1.0));

                let material =
                    Material::new(m, device, queue, diffuse, normal, dissolve_texture, layout);
                Arc::new(material)
            })
            .collect::<Vec<_>>();

        let meshes = self
            .meshes
            .into_iter()
            .map(|m| {
                let material_index = m.material;
                let num_elements = m.indices.len() as u32;
                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{} Vertex Buffer", m.name)),
                    contents: bytemuck::cast_slice(&m.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{} Index Buffer", m.name)),
                    contents: bytemuck::cast_slice(&m.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

                Arc::new(Mesh {
                    data: Arc::new(m),
                    vertex_buffer: Arc::new(vertex_buffer),
                    index_buffer: Arc::new(index_buffer),
                    num_elements,
                    material: material_index,
                })
            })
            .collect::<Vec<_>>();

        Ok(Model { meshes, materials })
    }
}

pub trait DrawModel<'a> {
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        ranges: &MeshRenderRange,
    ) -> u32;

    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        ranges: &Vec<MeshRenderRange>,
    ) -> u32;
}
