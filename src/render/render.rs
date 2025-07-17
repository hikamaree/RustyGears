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

use crate::RenderObject;
use crate::Graphics;
use crate::send_command;
use crate::Camera;
use crate::CommandFunction;
use crate::Model3d;
use crate::RenderBatch;
use crate::RenderTag;
use crate::RenderCommand;
use crate::ModelRenderData;
use crate::MeshRenderRange;
use crate::GameView;
use crate::Gear;
use crate::InstanceRaw;
use crate::Transform;
use crate::WorldScene;

use std::sync::Arc;
use std::collections::HashMap;

use cgmath::InnerSpace;
use cgmath::Matrix4;

use rayon::iter::ParallelIterator;
use rayon::iter::IntoParallelIterator;

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
pub struct Render;

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
        let Some(scene) = game.get::<WorldScene>() else {
            return;
        };

        let Some(camera_entity) = scene.active_camera else {
            return;
        };

        let Some(camera) = scene.world.get::<Camera>(camera_entity) else {
            return;
        };

        let camera_transform = scene.get_camera_transform(camera_entity);

        let view_matrix = camera.calc_matrix(&camera_transform);

        let (opaque_batches, mut transparent_instances): (Vec<_>, Vec<_>) = scene
            .world
            .query2::<RenderObject, Transform>()
            .into_par_iter()
            .map(|(_ent, render_obj, transform)| {
                let dist = (camera_transform.position - transform.position).magnitude();
                let lod_index = match dist {
                    d if d < 100.0 => 0,
                    d if d < 300.0 => 1.min(render_obj.lods.len() - 1),
                    d if d < 500.0 => 2.min(render_obj.lods.len() - 1),
                    _ => render_obj.lods.len() - 1,
                };

                let model3d = render_obj.lods[lod_index].clone();

                let Some(lod_model) = scene.get_model3d(&model3d) else {
                    return (vec![], vec![]);
                };
                let raw = transform.raw();
                let model_mat = Matrix4::from(raw.model);

                let mut local_opaque = Vec::new();
                let mut local_transparent = Vec::new();

                for (mesh_index, mesh) in lod_model.meshes.iter().enumerate() {
                    let center_local = mesh.bounding_sphere.center.extend(1.0);
                    let center_world = model_mat * center_local;

                    let scale = model_mat.x.truncate().magnitude()
                        .max(model_mat.y.truncate().magnitude())
                        .max(model_mat.z.truncate().magnitude());

                    let radius = mesh.bounding_sphere.radius * scale;

                    if !camera.can_see4(center_world, radius) {
                        continue;
                    }

                    match mesh.render_tag {
                        RenderTag::Opaque => {
                            local_opaque.push((model3d.clone(), lod_index, mesh_index, raw));
                        }
                        RenderTag::SortedTransparent => {
                            let center_view = view_matrix * center_world;
                            let depth = -center_view.z;

                            local_transparent.push((depth, model3d.clone(), lod_index, mesh_index, raw));
                        }
                        _ => {}
                    }
                }

                (local_opaque, local_transparent)
            }).reduce(|| (Vec::new(), Vec::new()),
                |mut acc, (opaque, transparent)| {
                    acc.0.extend(opaque);
                    acc.1.extend(transparent);
                    acc
                },
            );

        let mut opaque_group_map: HashMap<(Model3d, usize, usize), Vec<InstanceRaw>> = HashMap::with_capacity(opaque_batches.len() / 2);
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

        send_command(RenderCommand { batches });

        send_command(CommandFunction {
            run: Box::new(move |game| {
                let Ok(scene) = game.components.get_mut::<WorldScene>() else {
                    return;
                };

                let Ok(graphics) = game.components.get_mut::<Graphics>() else {
                    return;
                };

                let Some(camera_entity) = scene.active_camera else {
                    return;
                };

                let final_transform = scene.get_camera_transform(camera_entity);

                if let Some(camera) = scene.world.get_mut::<Camera>(camera_entity) {
                    camera.update_view_proj(&final_transform, &graphics.projection);
                    graphics.update(camera);
                }
            }),
        });
    }
}

impl Gear for Render {
    async fn update(&mut self, mut game: GameView) {
        self.render(&mut game);
    }
}
