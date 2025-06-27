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

use crate::Material;
use crate::MeshRenderRange;
use crate::RenderTag;
use std::sync::Arc;

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

/// A 3D model composed of one or more meshes and materials.
///
/// Models group multiple `Mesh` objects, each with its own geometry and material reference.
/// All meshes share the same transform during rendering, and materials are resolved by index.
#[derive(Clone)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
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
