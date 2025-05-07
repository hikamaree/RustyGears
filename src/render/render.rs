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

use std::sync::Arc;
use std::ops::Range;
use std::collections::HashMap;

use crossbeam::channel::Sender;

use cgmath::EuclideanSpace;
use cgmath::InnerSpace;
use cgmath::Matrix4;
use cgmath::Vector3;

use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;


#[inline]
fn group_contiguous_ranges(indices: &[u32]) -> Vec<Range<u32>> {
    if indices.is_empty() {
        return Vec::new();
    }

    let mut ranges = Vec::with_capacity(4);
    let mut start = indices[0];
    let mut prev = start;

    for &i in &indices[1..] {
        if i != prev + 1 {
            ranges.push(start..prev + 1);
            start = i;
        }
        prev = i;
    }

    ranges.push(start..prev + 1);
    ranges
}

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
    /// Prepares all visible models and instances from the scene and sends a [`RenderCommand`].
    ///
    /// This method:
    /// - Uses the active camera to perform frustum culling.
    /// - Collects visible instances of models tagged for rendering.
    /// - Optimizes draw calls by grouping instances into contiguous ranges.
    /// - Sends the resulting [`RenderCommand`] via the internal command channel.
    ///
    /// # Panics
    /// Panics if:
    /// - No active camera is found in the scene.
    /// - The internal command sender is not initialized (`sender` is `None`).
    ///
    /// # Parameters
    /// - `game`: A read-only view of the current game state, including the scene and camera.

    fn render(&mut self, game: &GameView) {
        let camera = game.scene.active_camera().expect("no camera found");

        let lod_and_mesh_results: Vec<(
            Vec<((RenderTag, String, usize), Vec<InstanceRaw>)>,
            Vec<((RenderTag, String, usize), Vec<(usize, Vec<u32>)>)>
        )> = game
            .scene
            .render_tag_object_to_instances
            .par_iter()
            .filter_map(|((tag, object_name), instance_ids)| {
                let render_object = game.scene.render_objects.get(object_name)?;
                if instance_ids.is_empty() {
                    return None;
                }

                let instances: Vec<InstanceRaw> = instance_ids
                    .iter()
                    .filter_map(|id| game.scene.instances.get(id))
                    .map(|inst| inst.raw)
                    .collect();

                let lod_count = render_object.lods.len();
                let lod_groups: HashMap<usize, Vec<InstanceRaw>> = instances
                    .into_iter()
                    .map(|raw| {
                        let pos = Vector3::new(raw.model[3][0], raw.model[3][1], raw.model[3][2]);
                        let dist = (camera.position.to_vec() - pos).magnitude();
                        let lod_index = match dist {
                            d if d < 10.0 => 0,
                            d if d < 30.0 => 1.min(lod_count - 1),
                            d if d < 100.0 => 2.min(lod_count - 1),
                            _ => lod_count - 1,
                        };
                        (lod_index, raw)
                    })
                .fold(HashMap::new(), |mut acc, (lod_index, raw)| {
                    acc.entry(lod_index).or_default().push(raw);
                    acc
                });

                let mut lod_entries = Vec::new();
                let mut mesh_entries = Vec::new();

                for (lod_index, group) in &lod_groups {
                    let key = (tag.clone(), object_name.clone(), *lod_index);
                    lod_entries.push((key.clone(), group.clone()));

                    let lod_model = &render_object.lods[*lod_index];
                    let mesh_data: Vec<(usize, Vec<u32>)> = lod_model
                        .meshes
                        .iter()
                        .enumerate()
                        .map(|(mesh_index, mesh)| {
                            let center_local = mesh.bounding_sphere.center.extend(1.0);
                            let radius_base = mesh.bounding_sphere.radius;

                            let visible_indices: Vec<u32> = group
                                .iter()
                                .enumerate()
                                .filter_map(|(i, inst_raw)| {
                                    let model_mat = Matrix4::from(inst_raw.model);
                                    let center_world = model_mat * center_local;

                                    let max_scale = model_mat.x.truncate().magnitude()
                                        .max(model_mat.y.truncate().magnitude())
                                        .max(model_mat.z.truncate().magnitude());

                                    let radius = radius_base * max_scale;

                                    if camera.can_see4(center_world, radius) {
                                        Some(i as u32)
                                    } else {
                                        None
                                    }
                                })
                            .collect();

                            (mesh_index, visible_indices)
                        })
                    .collect();

                    if !mesh_data.is_empty() {
                        mesh_entries.push((key.clone(), mesh_data));
                    }
                }

                Some((lod_entries, mesh_entries))
            })
        .collect();

        let mut lod_map = HashMap::new();
        let mut mesh_map = HashMap::new();
        for (lod_entries, mesh_entries) in lod_and_mesh_results {
            lod_map.extend(lod_entries);
            mesh_map.extend(mesh_entries);
        }

        let mut batches_map: HashMap<RenderTag, Vec<ModelRenderData>> = HashMap::new();

        for ((tag, object_name, lod_index), instances) in lod_map {
            let mesh_results: &Vec<(usize, Vec<u32>)> = match mesh_map.get(&(tag.clone(), object_name.clone(), lod_index)) {
                Some(m) => m,
                None => continue,
            };

            let mut used_indices: Vec<usize> = mesh_results
                .iter()
                .flat_map(|(_, indices)| indices.iter().map(|&i| i as usize))
                .collect();
            used_indices.sort_unstable();
            used_indices.dedup();

            let index_map: HashMap<usize, u32> = used_indices
                .iter()
                .enumerate()
                .map(|(new_idx, &old_idx)| (old_idx, new_idx as u32))
                .collect();

            let instance_vec: Vec<InstanceRaw> = used_indices.iter().map(|&i| instances[i]).collect();
            let instance_data: Arc<[InstanceRaw]> = instance_vec.into();

            let lod_model = match game.scene.render_objects.get(&object_name) {
                Some(obj) => &obj.lods[lod_index],
                None => continue,
            };
            let valid_mesh_count = lod_model.meshes.len();

            let mut mesh_ranges = Vec::with_capacity(mesh_results.len());
            for (mesh_index, old_indices) in mesh_results {
                if *mesh_index >= valid_mesh_count {
                    continue;
                }

                let mut new_indices = Vec::with_capacity(old_indices.len());
                for old in old_indices {
                    if let Some(&new_idx) = index_map.get(&(*old as usize)) {
                        new_indices.push(new_idx);
                    }
                }

                mesh_ranges.push(MeshRenderRange {
                    mesh_index: *mesh_index,
                    visible_instance_ranges: group_contiguous_ranges(&new_indices),
                });
            }

            batches_map.entry(tag.clone()).or_default().push(ModelRenderData {
                instance_data,
                mesh_ranges,
                object_name,
                lod_index,
            });
        }

        let batches: Vec<RenderBatch> = batches_map
            .into_iter()
            .map(|(tag, prepared_models)| RenderBatch { tag, prepared_models })
            .collect();

        if let Err(e) = self.sender.as_ref().unwrap().send(Box::new(RenderCommand { batches })) {
            eprintln!("Failed to send RenderCommand: {}", e);
        } else {
            game.graphics.window.request_redraw();
        }
    }
}

impl Gear for Render {
    fn setup(&mut self, _game: &mut Game, sender: Sender<Box<dyn Command>>) {
        self.sender = Some(sender);
    }

    fn update(&mut self, game: GameView) {
        self.render(&game);
    }
}
