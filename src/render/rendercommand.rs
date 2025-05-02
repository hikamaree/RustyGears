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

use egui_wgpu::ScreenDescriptor;
use crate::DrawModel;
use crate::BufferStrategy;
use crate::Buffer;
use crate::InstanceRaw;
use crate::Game;
use crate::GameView;
use crate::Command;
use crate::Camera;
use crate::ModelRenderData;

/// A render command containing all the data necessary to draw a frame.
///
/// This command is typically created by the built-in `Render` gear, which prepares
/// models for rendering based on visibility checks, camera frustum culling,
/// and instance data.
///
/// # Note
/// This command should **only** be constructed and sent by the default `Render` gear.
/// If you're implementing a custom rendering pipeline or replacing the default renderer,
/// you may emit your own `RenderCommand`, but in most cases users **should not** send
/// this command manually.
///
/// # Fields
/// - `prepared_models`: A list of models and associated instance data that are ready to be rendered.
/// - `camera`: The camera used to render the scene, including view and projection matrices.
pub struct RenderCommand {
    /// Models and their instance data to be drawn this frame.
    pub prepared_models: Vec<ModelRenderData>,

    /// The active camera used for rendering the current frame.
    pub camera: Camera,
}

impl Command for RenderCommand {
    fn apply(self: Box<Self>, game: &mut Game) {
        let graphics = &mut game.graphics.as_mut().unwrap();
        graphics.t_count = 0;

        let output = match graphics.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                eprintln!("Failed to get current texture: {e:?}");
                return;
            }
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let camera_bg = graphics.bind_groups.get(&crate::BindGroupLayoutKey::Camera).unwrap();
        let light_bg = graphics.bind_groups.get(&crate::BindGroupLayoutKey::Light).unwrap();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &graphics.depth_texture.view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(graphics.pipelines.get(&crate::RenderTag::PBR).unwrap());

            for model_data in &self.prepared_models {
                let buffer = graphics.buffers.entry(model_data.object_name.clone())
                    .and_modify(|b| {
                        b.ensure_capacity(&graphics.device, model_data.instance_data.len() * std::mem::size_of::<InstanceRaw>());
                        b.next();
                        b.write(&graphics.queue, bytemuck::cast_slice(&model_data.instance_data));
                    }).or_insert_with(|| {
                        Buffer::new(
                            &graphics.device,
                            model_data.instance_data.len().next_power_of_two() * std::mem::size_of::<InstanceRaw>(),
                            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                            BufferStrategy::Single,
                            &model_data.object_name,
                        )
                    });

                render_pass.set_vertex_buffer(1, buffer.current().slice(..));

                graphics.t_count += render_pass.draw_model_instanced(
                    &model_data.render_object.model,
                    camera_bg,
                    light_bg,
                    &model_data.mesh_ranges,
                );
            }
        }

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [graphics.config.width, graphics.config.height],
            pixels_per_point: graphics.window.scale_factor() as f32,
        };

        let gameview = GameView {
            graphics: graphics.clone(),
            scene: game.scene.create_snapshot(),
            time: game.time.clone(),
        };

        game.gui.as_mut()
            .expect("ERROR: egui is not initialized")
            .draw(
                &graphics.device,
                &graphics.queue,
                &mut encoder,
                &graphics.window,
                &view, screen_descriptor,
                &game.scene.render_gui,
                &gameview,
            );

        graphics.queue.submit(Some(encoder.finish()));
        output.present();
    }
}
