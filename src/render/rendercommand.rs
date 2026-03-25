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

use crate::render::gpu_resources::GpuResources;
use crate::render::pass::PassContext;
use crate::render::pass::PassData;
use crate::render::pass_id::PassId;
use crate::render::render_resources::RenderResources;
use crate::render::render_state::RenderState;
use crate::render::targets::RenderTargetPool;
use crate::render::window_state::WindowState;
use crate::Camera;
use crate::Command;
use crate::EguiRenderer;
use crate::Game;
use crate::GameView;
use crate::GameWindow;
use crate::GpuLight;
use crate::WorldScene;
use egui_wgpu::ScreenDescriptor;

#[derive(Debug)]
pub struct ExecuteRender {
    pub camera_entity: crate::Entity,
    pub camera_transform: crate::Transform,
    pub camera_uniform: Box<[u8]>,
    pub collected_data: Vec<(PassId, PassData)>,
    pub lights: Vec<GpuLight>,
}

impl ExecuteRender {
    pub fn new(
        camera_entity: crate::Entity,
        camera_transform: crate::Transform,
        camera_uniform: Box<[u8]>,
        collected_data: Vec<(PassId, PassData)>,
        lights: Vec<GpuLight>,
    ) -> Self {
        Self {
            camera_entity,
            camera_transform,
            camera_uniform,
            collected_data,
            lights,
        }
    }
}

impl Command for ExecuteRender {
    fn apply(self: Box<Self>, game: &mut Game) {
        let (output, view) = {
            let Ok(window) = game.components.get::<WindowState>() else {
                return;
            };
            match window.surface.get_current_texture() {
                Ok(frame) => {
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    (frame, view)
                }
                Err(e) => {
                    crate::log!(
                        crate::LogKind::Error,
                        "Failed to get surface texture: {e:?}"
                    );
                    return;
                }
            }
        };

        let encoder = {
            let Ok(gpu) = game.components.get::<GpuResources>() else {
                return;
            };
            gpu.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                })
        };

        let mut encoder = encoder;

        let projection = {
            let Ok(state) = game.components.get::<RenderState>() else {
                return;
            };
            state.projection.clone()
        };

        {
            let Ok(mut scene) = game.components.get_mut::<WorldScene>() else {
                return;
            };
            if let Some(camera) = scene.world.get_mut::<Camera>(self.camera_entity) {
                camera.update_view_proj(&self.camera_transform, &projection);
            }

            if !scene.pending_model_data.is_empty() {
                if let (Ok(gpu), Ok(res)) = (
                    game.components.get::<GpuResources>(),
                    game.components.get::<RenderResources>(),
                ) {
                    upload_pending_models(&*gpu, &*res, &mut scene);
                }
            }
        }

        {
            let Ok(gpu) = game.components.get::<GpuResources>() else {
                return;
            };
            let Ok(res) = game.components.get::<RenderResources>() else {
                return;
            };
            if let Some(camera_buffer) = res.get_buffer("camera") {
                gpu.queue
                    .write_buffer(&camera_buffer.current(), 0, &self.camera_uniform);
            }
        }

        let data_to_execute = self.collected_data;

        let gameview = GameView::new(game.components.clone());

        let _ = game.components.with::<RenderState, _>(|render_state| {
            render_state.t_count = 0;

            let (camera_bg, light_bg, shadow_camera_bg) = {
                let registry = render_state.registry.read().unwrap();
                (
                    registry
                        .get_bind_group(crate::render::registry::BindGroupId::Camera)
                        .cloned(),
                    registry
                        .get_bind_group(crate::render::registry::BindGroupId::Light)
                        .cloned(),
                    registry
                        .get_bind_group(crate::render::registry::BindGroupId::ShadowCamera)
                        .cloned(),
                )
            };

            let (camera_bg, light_bg, shadow_camera_bg) =
                match (camera_bg, light_bg, shadow_camera_bg) {
                    (Some(c), Some(l), Some(s)) => (c, l, s),
                    _ => return,
                };

            let passes_to_run: Vec<_> = {
                let registry = render_state.registry.read().unwrap();
                data_to_execute
                    .iter()
                    .filter_map(|(id, data)| registry.get_pass(id).map(|p| (p, data)))
                    .collect()
            };

            let mut passes_to_run: Vec<_> = passes_to_run
                .into_iter()
                .map(|(pass, data)| (pass.clone(), pass.id(), pass.order(), data))
                .collect();

            passes_to_run.sort_by_key(|(_, _, order, _)| *order);

            let gpu = match game.components.get::<GpuResources>() {
                Ok(g) => g,
                Err(_) => return,
            };
            let window = match game.components.get::<WindowState>() {
                Ok(w) => w,
                Err(_) => return,
            };
            let resources = match game.components.get::<RenderResources>() {
                Ok(r) => r,
                Err(_) => return,
            };

            let depth_view = render_state.depth_texture.view.clone();
            let config_width = window.config.width;
            let config_height = window.config.height;

            let scene = game.components.get::<WorldScene>().unwrap();

            let gpu_queue = gpu.queue.clone();

            let mut target_pool = RenderTargetPool::new();

            let mut ctx = PassContext {
                encoder: &mut encoder,
                scene: &*scene,
                camera_bind_group: &camera_bg,
                light_bind_group: &light_bg,
                shadow_camera_bind_group: &shadow_camera_bg,
                lights: &self.lights,
                targets: &mut target_pool,
                screen_view: &view,
                depth_view: &depth_view,
                screen_size: (config_width, config_height),
                shadow_view: None,
            };

            let mut shadow_view = None;

            for (pass, pass_id, _order, data) in passes_to_run {
                pass.execute(
                    &mut ctx,
                    &*gpu,
                    &window.config,
                    &*resources,
                    render_state,
                    data,
                );
                if pass_id == crate::render::pass_id::PassId::SHADOW {
                    if let Some(shadow_target) =
                        ctx.targets.get(crate::render::targets::OutputHandle(0))
                    {
                        shadow_view = Some(shadow_target.view.clone());
                    }
                    ctx.shadow_view = shadow_view.clone();
                }
            }

            resources.update_buffer("light", |light_buffer| {
                let mut gpu_lights = self.lights.clone();
                gpu_lights.resize(crate::MAX_LIGHTS, GpuLight::NULL);
                light_buffer.write(&gpu_queue, bytemuck::cast_slice(&gpu_lights));
            });
        });

        let game_window = match game.components.get::<GameWindow>() {
            Ok(w) => w,
            Err(e) => {
                crate::log!(crate::LogKind::Error, "UI: failed to get GameWindow: {}", e);
                return;
            }
        };

        let screen_descriptor = {
            let Ok(window) = game.components.get::<WindowState>() else {
                return;
            };
            ScreenDescriptor {
                size_in_pixels: [(&*window).config.width, (&*window).config.height],
                pixels_per_point: game_window.scale_factor() as f32,
            }
        };

        let (gpu_device, gpu_queue) = match game.components.get::<GpuResources>() {
            Ok(g) => (g.device.clone(), g.queue.clone()),
            Err(e) => {
                crate::log!(
                    crate::LogKind::Error,
                    "UI: failed to get GpuResources: {}",
                    e
                );
                return;
            }
        };

        let mut scene = match game.components.get_mut::<WorldScene>() {
            Ok(s) => s,
            Err(e) => {
                crate::log!(crate::LogKind::Error, "UI: failed to get WorldScene: {}", e);
                return;
            }
        };

        let _ = game.components.with::<EguiRenderer, _>(|gui| {
            gui.draw(
                &gpu_device,
                &gpu_queue,
                &mut encoder,
                game_window.window(),
                &view,
                screen_descriptor,
                &mut scene.render_gui,
                &gameview,
            );
        });

        gpu_queue.submit(Some(encoder.finish()));
        output.present();
    }

    fn priority(&self) -> crate::CommandPriority {
        crate::CommandPriority::High
    }
}

fn upload_pending_models(gpu: &GpuResources, resources: &RenderResources, scene: &mut WorldScene) {
    let layout = match resources
        .layouts
        .get_opt::<crate::render::layout::TextureLayout>()
    {
        Some(l) => l.clone(),
        None => return,
    };

    let device = gpu.device.clone();
    let queue = gpu.queue.clone();

    let pending: Vec<_> = scene.drain_pending_model_data().collect();

    for (model3d, model_data) in pending {
        let layout = layout.clone();
        let device = device.clone();
        let queue = queue.clone();

        std::thread::spawn(move || {
            let resources =
                crate::model::SimpleGpuResources::new((*device).clone(), (*queue).clone());
            match model_data.into_gpu(&resources, &layout) {
                Ok(model) => {
                    crate::send_command(crate::CommandFunction {
                        run: Box::new(move |game| {
                            let Ok(mut scene) = game.components.get_mut::<WorldScene>() else {
                                return;
                            };
                            scene.add_model3d(model3d, model);
                        }),
                    });
                }
                Err(e) => {
                    crate::log!(
                        crate::LogKind::Error,
                        "Failed to upload model '{}': {}",
                        model3d.path,
                        e
                    );
                }
            }
        });
    }
}
