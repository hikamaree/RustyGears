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

use std::sync::Arc;
use crate::registry::RegisterPass;
use crate::Camera;
use crate::GameView;
use crate::Gear;
use crate::render::ExecuteRender;
use crate::WorldScene;

use crate::render::passes::WeightedPass;
use crate::render::passes::TransparentPass;
use crate::render::passes::ShadowPass;
use crate::render::passes::OpaquePass;

use crate::render::gpu_resources::GpuResources;
use crate::render::render_resources::RenderResources;
use crate::render::render_state::RenderState;
use crate::render::window_state::WindowState;
use crate::EguiRenderer;


#[derive(Debug, Default)]
pub struct Render;

impl Render {
    pub fn init(game: &mut crate::Game) {
        let (device, _queue, config, window) = {
            let gpu = match game.components.get::<GpuResources>() {
                Ok(g) => g,
                Err(_) => return,
            };
            let window = match game.components.get::<WindowState>() {
                Ok(w) => w,
                Err(_) => return,
            };
            let game_window = match game.components.get::<crate::GameWindow>() {
                Ok(w) => w,
                Err(_) => return,
            };
            (
                gpu.device.clone(),
                gpu.queue.clone(),
                window.config.clone(),
                game_window.window().clone(),
            )
        };

        let layouts = crate::render::layout::LayoutRegistry::new(&device);
        let render_resources = RenderResources::new(layouts);

        let render_config = crate::render::RenderConfig::default();

        let depth_texture = crate::Texture::create_depth_texture(
            &device,
            &config,
            "depth_texture",
        );
        let projection = render_config.projection(config.width, config.height);

        let render_state = RenderState::new(depth_texture, projection, render_config);

        let egui = EguiRenderer::new(
            &device,
            config.format,
            None,
            1,
            &window,
        );

        game.components.insert(render_resources);
        game.components.insert(render_state);
        game.components.insert(egui);

        const DEFAULT_CAMERA_BUFFER_SIZE: usize = 128;

        let Ok(resources) = game.components.get::<RenderResources>() else {
            crate::log!(crate::LogKind::Error, "Failed to get RenderResources");
            return;
        };
        let Ok(state) = game.components.get_mut::<RenderState>() else {
            crate::log!(crate::LogKind::Error, "Failed to get RenderState");
            return;
        };
        let Ok(gpu) = game.components.get::<GpuResources>() else {
            crate::log!(crate::LogKind::Error, "Failed to get GpuResources");
            return;
        };

        resources.create_buffer(
            &gpu.device,
            "camera",
            DEFAULT_CAMERA_BUFFER_SIZE,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            crate::BufferStrategy::Single,
        );

        resources.create_buffer(
            &gpu.device,
            "light",
            crate::MAX_LIGHTS * std::mem::size_of::<crate::GpuLight>(),
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            crate::BufferStrategy::Single,
        );

        resources.create_buffer(
            &gpu.device,
            "shadow_camera",
            DEFAULT_CAMERA_BUFFER_SIZE,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            crate::BufferStrategy::Single,
        );

        if let Some(bind_group) = resources.create_bind_group::<crate::render::layout::LightLayout>(
            &gpu.device,
            "light",
        ) {
            if let Ok(mut registry) = state.registry.write() {
                registry.register_bind_group(
                    crate::render::registry::BindGroupId::Light,
                    bind_group,
                );
            } else {
                crate::log!(crate::LogKind::Error, "Failed to acquire registry write lock");
            }
        }

        if let Some(bind_group) = resources.create_bind_group::<crate::render::layout::CameraLayout>(
            &gpu.device,
            "camera",
        ) {
            if let Ok(mut registry) = state.registry.write() {
                registry.register_bind_group(
                    crate::render::registry::BindGroupId::Camera,
                    bind_group,
                );
            } else {
                crate::log!(crate::LogKind::Error, "Failed to acquire registry write lock");
            }
        }

        if let Some(bind_group) = resources.create_bind_group::<crate::render::layout::CameraLayout>(
            &gpu.device,
            "shadow_camera",
        ) {
            if let Ok(mut registry) = state.registry.write() {
                registry.register_bind_group(
                    crate::render::registry::BindGroupId::ShadowCamera,
                    bind_group,
                );
            } else {
                crate::log!(crate::LogKind::Error, "Failed to acquire registry write lock");
            }
        }
    }

    pub fn collect_render_data(game: &GameView) -> Option<ExecuteRender> {
        let scene = game.get::<WorldScene>().ok()?;
        let camera_entity = scene.active_camera?;
        let camera_transform = scene.get_camera_transform(camera_entity);
        let camera = scene.world.get::<Camera>(camera_entity)?;

        let lights = crate::render::collect_lights(&scene);
        let camera_uniform = camera.get_uniform(lights.len() as u32);

        let state = game.get::<crate::render::RenderState>().ok()?;
        let pass_registry = state.registry.clone();

        let collected_data = {
            let Ok(registry) = pass_registry.read() else {
                crate::log!(crate::LogKind::Error, "Failed to acquire registry read lock");
                return None;
            };
            let order = registry.graph().execution_order();
            
            let mut collected = Vec::new();
            for pass_id in order {
                if let Some(pass) = registry.get_pass(pass_id) {
                    if pass.should_run(&scene) {
                        let data = pass.collect(&scene, camera, &camera_transform);
                        collected.push((*pass_id, data));
                    }
                }
            }
            collected
        };

        Some(ExecuteRender::new(
            camera_entity,
            camera_transform,
            camera_uniform,
            collected_data,
            lights,
        ))
    }
}

impl Gear for Render {
    async fn setup(&mut self, _game: &GameView) {
        crate::send_command(crate::CommandFunction {
            run: Box::new(move |game| {
                Self::init(game);
            }),
        });

        crate::send_command(RegisterPass { render_pass: Arc::new(ShadowPass::new()) });
        crate::send_command(RegisterPass { render_pass: Arc::new(OpaquePass::new()) });
        crate::send_command(RegisterPass { render_pass: Arc::new(WeightedPass::new()) });
        crate::send_command(RegisterPass { render_pass: Arc::new(TransparentPass::new()) });
        crate::send_command(crate::CommandFunction {
            run: Box::new(|game| {
                let Ok(state) = game.components.get_mut::<crate::render::RenderState>() else {
                    return;
                };
                let Ok(mut registry) = state.registry.write() else {
                    crate::log!(crate::LogKind::Error, "Failed to acquire registry write lock");
                    return;
                };
                if let Err(e) = registry.validate() {
                    crate::log!(crate::LogKind::Error, "Invalid render graph: {}", e);
                }
            }),
        });
    }

    async fn update(&mut self, game: GameView) {
        let Some(render_cmd) = Self::collect_render_data(&game) else {
            return;
        };
        crate::send_command(render_cmd);
    }
}
