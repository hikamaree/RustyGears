// SPDX-License-Identifier: GPL-3.0-or-later

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
use std::collections::HashSet;

use rayon::iter::IndexedParallelIterator;
use rayon::iter::ParallelIterator;
use rayon::iter::IntoParallelRefIterator;

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
    ///
    /// # Errors
    /// This method currently panics on failure to send a command.
    /// In a production environment, it's recommended to handle this gracefully (e.g., via `if let Err(e) = ...`).
    fn render(&mut self, game: &GameView) {
        fn group_contiguous_ranges(indices: &[u32]) -> Vec<Range<u32>> {
            if indices.is_empty() {
                return Vec::new();
            }

            let mut ranges = Vec::new();
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

        let camera = game.scene.active_camera()
            .expect("no camera found");
        let prepared_models: Vec<ModelRenderData> = game.scene.render_objects.par_iter()
            .filter_map(|(object_name, render_object)| {
                let key = (crate::RenderTag::PBR, object_name.to_string());
                let instance_ids = game.scene.render_tag_object_to_instances.get(&key)?;
                if instance_ids.is_empty() {
                    return None;
                }

                let all_instances: Vec<InstanceRaw> = instance_ids.iter()
                    .filter_map(|id| game.scene.instances.get(id))
                    .map(|instance| instance.to_raw())
                    .collect();

                let transforms: Vec<_> = all_instances
                    .iter()
                    .map(|raw| Matrix4::from(raw.model))
                    .collect();

                let mesh_results: Vec<_> = render_object.model.meshes
                    .par_iter()
                    .enumerate()
                    .filter_map(|(mesh_index, mesh)| {
                        let center_local = mesh.bounding_sphere.center.extend(1.0);
                        let radius_base = mesh.bounding_sphere.radius;

                        let visible_indices: Vec<u32> = transforms
                            .iter()
                            .enumerate()
                            .filter_map(|(i, model)| {
                                let center_world = model * center_local;
                                let center = center_world.truncate();

                                let max_scale = model.x.magnitude()
                                    .max(model.y.magnitude())
                                    .max(model.z.magnitude());

                                let radius = radius_base * max_scale;

                                camera.can_see(center, radius).then_some(i as u32)
                            })
                        .collect();

                        (!visible_indices.is_empty())
                            .then(|| (mesh_index, visible_indices))
                    })
                .collect();

                if mesh_results.is_empty() {
                    return None;
                }

                let mut used_indices: HashSet<usize> = HashSet::new();
                for (_, indices) in &mesh_results {
                    for &i in indices {
                        used_indices.insert(i as usize);
                    }
                }

                let mut used_indices_sorted: Vec<usize> = used_indices.into_iter().collect();
                used_indices_sorted.sort_unstable();

                let index_map: HashMap<usize, u32> = used_indices_sorted
                    .par_iter()
                    .enumerate()
                    .map(|(new_idx, &old_idx)| (old_idx, new_idx as u32))
                    .collect();

                let instance_data: Arc<[InstanceRaw]> = used_indices_sorted
                    .par_iter()
                    .map(|&i| all_instances[i].clone())
                    .collect::<Vec<_>>()
                    .into();

                let mesh_ranges: Vec<_> = mesh_results
                    .into_iter()
                    .map(|(mesh_index, old_indices)| {
                        let new_indices: Vec<u32> = old_indices
                            .into_iter()
                            .filter_map(|old| index_map.get(&(old as usize)).copied())
                            .collect();

                        MeshRenderRange {
                            mesh_index,
                            visible_instance_ranges: group_contiguous_ranges(&new_indices),
                        }
                    })
                .collect();

                Some(ModelRenderData {
                    render_object: render_object.clone(),
                    instance_data,
                    mesh_ranges,
                    object_name: object_name.clone(),
                })
            })
        .collect();

        let cmd = RenderCommand {
            prepared_models,
            camera: camera.clone(),
        };

        self.sender.as_ref().unwrap().send(Box::new(cmd)).expect("usro se render");
    }
}

impl Gear for Render {
    fn setup(&mut self, _game: &mut Game, sender: Sender<Box<dyn Command>>) {
        self.sender = Some(sender);
    }

    fn update(&mut self, game: GameView) {
        game.graphics.window.request_redraw();
        self.render(&game);
    }
}
