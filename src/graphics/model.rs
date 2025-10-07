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

use wgpu::util::DeviceExt;
use wgpu::Device;
use wgpu::Queue;

use std::path::Path;
use std::sync::Arc;

use crate::GameView;
use crate::Graphics;
use crate::MeshRenderRange;
use crate::RenderTag;
use crate::ModelVertex;
use crate::Material;
use crate::Texture;

/// A bounding volume in the form of a sphere.
///
/// Used for fast frustum culling and visibility checks in 3D space.
/// The sphere is defined by its center position and radius.
#[derive(Debug, Clone, Copy)]
pub struct BoundingSphere {
    pub center: cgmath::Vector3<f32>,
    pub radius: f32,
}

/// A single mesh containing vertex/index buffers and material info.
///
/// Each mesh represents a discrete renderable surface with its own geometry
/// and associated material. It holds GPU buffer handles, material index,
/// and precomputed bounding volume used for culling and LOD operations.
#[derive(Debug, Clone)]
pub struct Mesh {
    pub name: String,
    pub vertex_buffer: Arc<wgpu::Buffer>,
    pub index_buffer: Arc<wgpu::Buffer>,
    pub num_elements: u32,
    pub material: usize,
    pub bounding_sphere: BoundingSphere,
    pub render_tag: RenderTag,
}

/// A 3D model composed of one or more meshes and materials.
///
/// Models group multiple `Mesh` objects, each with its own geometry and material reference.
/// All meshes share the same transform during rendering, and materials are resolved by index.
#[derive(Debug, Default, Clone)]
pub struct Model {
    pub meshes: Vec<Arc<Mesh>>,
    pub materials: Vec<Arc<Material>>,
}

impl Model {
    /// Loads a 3D model from an OBJ file and prepares it for GPU rendering.
    ///
    /// This function parses the OBJ file at the given `path`, loads all associated materials and textures,
    /// calculates tangents and bitangents needed for normal mapping, and creates vertex and index buffers
    /// for each mesh in the model. It also assigns a `RenderTag` to each mesh based on its transparency.
    ///
    /// ### Parameters
    /// - `path`: Path to the `.obj` file to load. Associated `.mtl` and texture files are resolved relative to this path.
    /// - `device`: Reference to the `wgpu::Device` used for buffer and texture creation.
    /// - `queue`: Reference to the `wgpu::Queue` used to upload texture and buffer data.
    /// - `layout`: A reference to a `BindGroupLayout` used for constructing the material bind groups.
    ///
    /// ### Returns
    /// - A fully prepared [`Model`] containing GPU-ready meshes and materials.
    ///
    /// ### Panics
    /// - If the OBJ file cannot be loaded.
    /// - If any required texture or default texture file (`default.jpg`) is missing.
    /// - If any mesh has malformed vertex data (e.g., out-of-bounds indices).
    ///
    /// ### Notes
    /// - All vertex tangents and bitangents are automatically computed per-triangle for normal mapping.
    /// - If no materials are found in the OBJ file, a default material with a `default.jpg` texture will be created.
    /// - If a material has partial transparency (`dissolve < 1.0`) or an alpha texture, it is marked as either
    ///   `SortedTransparent` or `WeightedTransparent` depending on internal heuristics (`use_weighted_blended()`).
    ///
    /// ### Example
    /// ```rust
    /// use std::path::Path;
    /// let model = load_model(Path::new("assets/model.obj"), &device, &queue, &material_layout).await;
    /// ```

    pub fn from_obj(path: &str, game: &GameView) -> Result<Model, String> {
        let Ok(graphics) = game.get::<Graphics>() else {
            return Err("Graphics is not initialized".to_string());
        };

        let layout = match graphics.bind_group_layouts.get(&crate::BindGroupLayoutKey::Texture) {
            Some(layout) => layout.clone(),
            None => return Err("Texture bind group layout is not initialized".to_string()),
        };

        let device = graphics.device.clone();
        let queue = graphics.queue.clone();
        drop(graphics);

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

        fn load_texture(base_dir: &Path, path: &str, is_normal_map: bool, device: &Device, queue: &Queue) -> Option<Texture> {
            let tex_path = base_dir.join(path);
            let tex_path_str = tex_path.to_string_lossy();
            let data = std::fs::read(&tex_path).ok()?;
            Texture::from_bytes(device, queue, &data, &tex_path_str, is_normal_map).ok()
        }

        if obj_materials.is_empty() {
            materials.push(Material::new(
                    &device,
                    &queue,
                    "default",
                    None,
                    None,
                    None,
                    1.0,
                    &layout,
            ).into());
        } else {
            for m in obj_materials {
                let diffuse_texture = match &m.diffuse_texture {
                    Some(path) if !path.is_empty() => {
                        load_texture(&base_dir, path, false, &device, &queue)
                    }
                    _ => {
                        match &m.diffuse {
                            Some(color) => {
                                Some(Texture::from_color(&device, &queue, [color[0], color[1], color[2], 1.0], Some("color"), false))
                            }
                            _ => {
                                None
                            }
                        }
                    }
                };

                let normal_texture = match &m.normal_texture {
                    Some(path) if !path.is_empty() => {
                        load_texture(&base_dir, path, true, &device, &queue)
                    }
                    _ => {
                        None
                    }
                };

                let dissolve = if let Some(value) = m.dissolve {
                    value
                } else {
                    1.0
                };

                let dissolve_texture = if dissolve < 1.0 {
                    Some(Texture::from_color(&device, &queue, [1.0, 1.0, 1.0, dissolve], Some("alpha"), false))
                } else if let Some(path) = &m.dissolve_texture {
                    if !path.is_empty() {
                        load_texture(&base_dir, path, false, &device, &queue)
                    } else {
                        None
                    }
                } else {
                    None
                };

                materials.push(Material::new(
                        &device,
                        &queue,
                        &m.name,
                        diffuse_texture,
                        normal_texture,
                        dissolve_texture,
                        dissolve,
                        &layout,
                ).into());
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

            let material_index = match m.mesh.material_id {
                Some(id) => id,
                None => 0,
            };

            let material: &Arc<Material> = &materials[material_index];

            let render_tag = if material.dissolve < 1.0 || material.has_transparency_texture() {
                if material.use_weighted_blended() {
                    RenderTag::WeightedTransparent
                } else {
                    RenderTag::SortedTransparent
                }
            } else {
                RenderTag::Opaque
            };

            Arc::new( Mesh {
                name: m.name,
                vertex_buffer: Arc::new(vertex_buffer),
                index_buffer: Arc::new(index_buffer),
                num_elements: m.mesh.indices.len() as u32,
                material: material_index,
                bounding_sphere,
                render_tag,
            })
        })
        .collect::<Vec<_>>();

        Ok(Model {
            meshes,
            materials,
        })
    }
}

/// A trait for drawing models and meshes with instancing support.
///
/// This trait abstracts drawing logic for individual meshes and full models
/// with support for camera and lighting bind groups, as well as visibility ranges.
/// Implemented for `wgpu::RenderPass`, this allows efficient instanced rendering.
pub trait DrawModel<'a> {
    /// Draws a single mesh with one material, using instanced drawing.
    ///
    /// Accepts a precomputed list of visible instance ranges for the mesh,
    /// as well as bind groups for camera and lighting data.
    ///
    /// Returns the total number of triangles rendered.
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        ranges: &MeshRenderRange
    ) -> u32;

    /// Draws all meshes within a model using instanced rendering.
    ///
    /// Meshes are rendered using their associated materials. Each mesh in the model
    /// is matched with a `MeshRenderRange` entry containing visibility information.
    ///
    /// Returns the total number of triangles rendered across all meshes.
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
