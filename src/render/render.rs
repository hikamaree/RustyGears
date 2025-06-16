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

use crate::Camera;
use crate::Graphics;
use crate::Model3d;
use crate::RenderBatch;
use crate::RenderTag;
use crate::Command;
use crate::RenderCommand;
use crate::ModelRenderData;
use crate::MeshRenderRange;
use crate::Game;
use crate::GameView;
use crate::Gear;
use crate::InstanceRaw;
use crate::Transform;
use crate::WorldScene;

use std::sync::Arc;
use std::collections::HashMap;

use crossbeam::channel::Sender;

use cgmath::InnerSpace;
use cgmath::Matrix4;

/// A gear responsible for preparing and sending render commands to the renderer.
///
/// The `Render` gear collects visible models from the scene based on the active camera's view
/// frustum and sends a [`RenderCommand`] containing instance data for rendering.
///
/// It performs visibility checks (frustum culling) on a per-mesh basis and groups instances
/// into contiguous draw ranges to optimize GPU instancing. The result is a list of [`ModelRenderData`]
/// objects, one for each visible renderable object.
///
/// # Note
/// This gear should typically be the **only** producer of [`RenderCommand`]s in the system.
/// If you are writing a custom rendering system, you may choose to replace or extend this behavior,
/// but under normal usage, avoid emitting [`RenderCommand`]s from other gears.
///
/// # Fields
/// - `sender`: A command sender used to pass prepared [`RenderCommand`]s to the main game loop or renderer.
#[derive(Debug, Default)]
pub struct Render {
    pub sender: Option<Sender<Box<dyn Command>>>,
}

impl Render {
    /// Prepares and submits a [`RenderCommand`] containing all visible model instances in the scene.
    ///
    /// This method performs the following steps:
    /// - Extracts the active camera and uses it for visibility determination (if present).
    /// - Iterates over all [`ModelInstance`] components in the world and selects the appropriate LOD
    ///   based on the instance's distance from the camera.
    /// - Applies frustum culling using each mesh's bounding sphere.
    /// - Separates opaque and transparent meshes based on their [`RenderTag`].
    /// - Groups visible opaque instances by `(model_name, lod_index, mesh_index)` for efficient instanced rendering.
    /// - Sorts transparent instances back-to-front based on their depth along the camera's viewing direction,
    ///   ensuring correct rendering order for alpha blending.
    /// - Constructs [`RenderBatch`] structures containing instance and mesh range data.
    /// - Sends a [`RenderCommand`] through the internal channel for execution in the rendering thread.
    ///
    /// If no active camera is present, the function will submit an empty `RenderCommand`
    /// and trigger a redraw, but no objects will be rendered.
    ///
    /// # Parameters
    /// - `game`: A read-only reference to the current [`GameView`], which contains the scene graph,
    ///   active camera, and shared rendering resources./
    fn render(&mut self, game: &GameView) {
        let Some(sender) = self.sender.as_ref() else {
            return;
        };

        let Some(scene) = game.get::<WorldScene>() else {
            return;
        };

        let Some(graphics) = game.get::<Graphics>() else {
            return;
        };

        let Some(camera_entity) = scene.active_camera else {
            if let Ok(_) = sender.send(Box::new(RenderCommand { batches: vec![] })) {
                graphics.window.request_redraw();
            }
            return;
        };

        let Some(camera) = scene.world.get::<Camera>(camera_entity) else {
            if let Ok(_) = sender.send(Box::new(RenderCommand { batches: vec![] })) {
                graphics.window.request_redraw();
            }
            return;
        };

        let camera_transform = scene.get_camera_transform(camera_entity);

        let mut opaque_batches = vec![];
        let mut transparent_instances = vec![];

        for (_ent, model3d, transform) in scene.world.query2::<Model3d, Transform>() {
            let Some(render_obj) = scene.render_objects.get(&model3d) else {
                continue;
            };

            let dist = (camera_transform.position - transform.position).magnitude();
            let lod_index = match dist {
                d if d < 100.0 => 0,
                d if d < 300.0 => 1.min(render_obj.lods.len() - 1),
                d if d < 500.0 => 2.min(render_obj.lods.len() - 1),
                _ => render_obj.lods.len() - 1,
            };

            let lod_model = &render_obj.lods[lod_index];
            let raw = transform.raw();

            for (mesh_index, mesh) in lod_model.meshes.iter().enumerate() {
                let tag = &mesh.render_tag;

                let center_local = mesh.bounding_sphere.center.extend(1.0);
                let model_mat = Matrix4::from(raw.model);
                let center_world = model_mat * center_local;

                let scale = model_mat.x.truncate().magnitude()
                    .max(model_mat.y.truncate().magnitude())
                    .max(model_mat.z.truncate().magnitude());

                let radius = mesh.bounding_sphere.radius * scale;

                if !camera.can_see4(center_world, radius) {
                    continue;
                }

                match tag {
                    RenderTag::Opaque => {
                        opaque_batches.push((
                                model3d,
                                lod_index,
                                mesh_index,
                                raw,
                        ));
                    }
                    RenderTag::SortedTransparent => {
                        let delta = center_world.truncate() - camera_transform.position;
                        let depth = camera.forward.dot(delta);

                        transparent_instances.push((
                                depth,
                                model3d,
                                lod_index,
                                mesh_index,
                                raw,
                        ));
                    }
                    _ => {}
                }
            }
        }

        let mut opaque_group_map: HashMap<(Model3d, usize, usize), Vec<InstanceRaw>> = HashMap::new();
        for (model3d, lod, mesh_idx, raw) in opaque_batches {
            opaque_group_map.entry((model3d.clone(), lod, mesh_idx)).or_default().push(raw);
        }

        let mut opaque_render_data = vec![];
        for ((model3d, lod, mesh_idx), instances) in opaque_group_map {
            opaque_render_data.push(RenderBatch {
                tag: RenderTag::Opaque,
                prepared_models: vec![ModelRenderData {
                    model3d,
                    lod_index: lod,
                    instance_data: Arc::from(instances.clone()),
                    mesh_ranges: vec![MeshRenderRange {
                        mesh_index: mesh_idx,
                        visible_instance_ranges: vec![0..instances.len() as u32],
                    }],
                }],
            });
        }

        transparent_instances.sort_by(|a, b| b.0.total_cmp(&a.0));

        let transparent_render_data = transparent_instances.into_iter().map(|(_, model3d, lod, mesh_idx, raw)| {
            RenderBatch {
                tag: RenderTag::SortedTransparent,
                prepared_models: vec![ModelRenderData {
                    model3d: model3d.clone(),
                    lod_index: lod,
                    instance_data: Arc::from([raw]),
                    mesh_ranges: vec![MeshRenderRange {
                        mesh_index: mesh_idx,
                        visible_instance_ranges: vec![0..1],
                    }],
                }],
            }
        });

        let batches: Vec<RenderBatch> = opaque_render_data
            .into_iter()
            .chain(transparent_render_data)
            .collect();

        if let Ok(_) = sender.send(Box::new(RenderCommand { batches })) {
            graphics.window.request_redraw();
        }
    }
}

impl Gear for Render {
    fn setup(&mut self, _game: &mut Game, sender: Sender<Box<dyn Command>>) {
        self.sender = Some(sender);
    }

    fn update(&mut self, mut game: GameView) {
        self.render(&mut game);
    }
}
