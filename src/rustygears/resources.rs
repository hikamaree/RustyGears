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

use crate::BoundingSphere;
use cgmath::InnerSpace;
use wgpu::Device;
use wgpu::Queue;
use crate::Mesh;
use crate::ModelVertex;
use crate::Model;
use crate::Material;
use crate::Texture;
use std::path::Path;
use std::sync::Arc;

use wgpu::util::DeviceExt;

pub async fn load_model(path: &Path, device: &Device, queue: &Queue, layout: &wgpu::BindGroupLayout) -> Model {
    let base_dir = path.parent().unwrap().to_path_buf();

    let (models, obj_materials) = tobj::load_obj(
        &path,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    ).expect(&format!("Failed to load OBJ model at {:?}", path));

    let obj_materials = obj_materials.unwrap_or_default();

    let mut materials = Vec::new();

    async fn load_texture(base_dir: &Path, path: &str, is_normal_map: bool, device: &Device, queue: &Queue) -> Texture {
        let tex_path = base_dir.join(path);
        let tex_path_str = tex_path.to_string_lossy();
        let data = std::fs::read(&tex_path)
            .expect(&format!("Failed to load texture: {:?}", tex_path));
        Texture::from_bytes(device, queue, &data, &tex_path_str, is_normal_map)
    }


    if obj_materials.is_empty() {
        let diffuse_texture = load_texture(&base_dir, "default.jpg", false, device, queue).await;
        let normal_texture = Texture::from_color(device, queue, [0.0, 0.0, 0.0, 0.0], Some("color"), false);

        materials.push(Material::new(
                device,
                "default",
                diffuse_texture,
                normal_texture,
                None,
                layout,
        ));
    } else {
        for m in obj_materials {
            let diffuse_texture = match &m.diffuse_texture {
                Some(path) if !path.is_empty() => {
                    load_texture(&base_dir, path, false, device, queue).await
                }
                _ => {
                    match &m.diffuse {
                        Some(color) => {
                            Texture::from_color(device, queue, [color[0], color[1], color[2], 1.0], Some("color"), false)
                        }
                        _ => {
                            load_texture(&base_dir, "default.jpg", false, device, queue).await
                        }
                    }
                }
            };

            let normal_texture = match &m.normal_texture {
                Some(path) if !path.is_empty() => {
                    load_texture(&base_dir, path, true, device, queue).await
                }
                _ => {
                    Texture::from_color(device, queue, [0.0, 0.0, 0.0, 1.0], Some("color"), false)
                }
            };

            // let dissolve_texture = match &m.dissolve_texture {
            //     Some(path) if !path.is_empty() => {
            //         Some(load_texture(&base_dir, path, true, device, queue).await)
            //     }
            //     _ => {
            //         Some(Texture::from_color(device, queue, [1.0, 1.0, 1.0, 1.0], Some("alpha"), false))
            //     }
            // };

            let dissolve_texture = if m.dissolve.unwrap_or(1.0) < 1.0 {
                Some(Texture::from_color(device, queue, [1.0, 1.0, 1.0, m.dissolve.unwrap()], Some("alpha"), false))
            } else if let Some(path) = &m.dissolve_texture {
                if !path.is_empty() {
                    Some(load_texture(&base_dir, path, false, device, queue).await)
                } else {
                    Some(Texture::from_color(device, queue, [1.0, 1.0, 1.0, 1.0], Some("alpha"), false))
                }
            } else {
                Some(Texture::from_color(device, queue, [1.0, 1.0, 1.0, 1.0], Some("alpha"), false))
            };


            materials.push(Material::new(
                    device,
                    &m.name,
                    diffuse_texture,
                    normal_texture,
                    dissolve_texture,
                    layout,
            ));
        }
    }

    let meshes = models.into_iter().map(|m| {
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

        let indices = &m.mesh.indices;
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

            vertices[c[0] as usize].tangent = (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
            vertices[c[1] as usize].tangent = (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
            vertices[c[2] as usize].tangent = (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();
            vertices[c[0] as usize].bitangent = (bitangent + cgmath::Vector3::from(vertices[c[0] as usize].bitangent)).into();
            vertices[c[1] as usize].bitangent = (bitangent + cgmath::Vector3::from(vertices[c[1] as usize].bitangent)).into();
            vertices[c[2] as usize].bitangent = (bitangent + cgmath::Vector3::from(vertices[c[2] as usize].bitangent)).into();

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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", path)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", path)),
            contents: bytemuck::cast_slice(&m.mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

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


        Mesh {
            name: path.to_string_lossy().to_string(),
            vertex_buffer: Arc::new(vertex_buffer),
            index_buffer: Arc::new(index_buffer),
            num_elements: m.mesh.indices.len() as u32,
            material: m.mesh.material_id.unwrap_or(0),
            bounding_sphere,
        }
    })
    .collect::<Vec<_>>();

    Model {
        meshes,
        materials,
    }
}
