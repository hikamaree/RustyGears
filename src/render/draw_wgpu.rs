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

use crate::{DrawModel, Material, Mesh, MeshRenderRange, Model};

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
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
        ranges: &Vec<MeshRenderRange>,
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
