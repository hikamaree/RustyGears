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

use std::ops::Range;
use std::sync::Arc;

use crate::InstanceRaw;
use crate::Model3d;

/// An enum representing different rendering tags for objects in the scene.
///
/// Render tags are used to classify and specify how an object should be rendered.
/// For example, objects may be rendered with different materials or effects, like PBR (Physically Based Rendering),
/// Unlit, Wireframe mode, etc. Custom tags can also be created with a string value.
///
/// # Variants
/// - `Opaque`: The default render tag for fully opaque geometry. Rendered first without sorting.
/// - `SortedTransparent`: Transparent objects that require depth-based sorting for correct rendering (e.g. glass).
/// - `WeightedTransparent`: Transparent objects rendered using weighted blended order-independent transparency.
/// - `Custom(String)`: A custom render tag identified by a string, useful for user-defined rendering passes or effects.
#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum RenderTag {
    Opaque,
    SortedTransparent,
    WeightedTransparent,
    Custom(String),
}

/// Specifies a contiguous range of visible instances for a particular mesh
/// within a model. This is used during rendering to issue instanced draw calls
/// only for visible instances.
///
/// # Fields
/// - `mesh_index`: Index of the mesh within the model's LOD.
/// - `visible_instance_ranges`: One or more contiguous ranges of visible instance indices.
#[derive(Debug)]
pub struct MeshRenderRange {
    pub mesh_index: usize,
    pub visible_instance_ranges: Vec<Range<u32>>,
}

impl Clone for MeshRenderRange {
    fn clone(&self) -> Self {
        Self {
            mesh_index: self.mesh_index,
            visible_instance_ranges: self.visible_instance_ranges.clone(),
        }
    }
}

/// Represents a single model to be rendered, along with its instance transforms
/// and per-mesh visibility data. Used by the renderer to issue draw calls.
///
/// # Fields
/// - `instance_data`: GPU-ready instance transforms for this model.
/// - `mesh_ranges`: Per-mesh visibility ranges, used for culling and draw call batching.
/// - `model3d`: The model identifier (resource handle).
/// - `lod_index`: Selected LOD level for this draw.
#[derive(Debug, Clone)]
pub struct ModelRenderData {
    pub instance_data: Arc<[InstanceRaw]>,
    pub mesh_ranges: Vec<MeshRenderRange>,
    pub model3d: Model3d,
    pub lod_index: usize,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BufferKey {
    pub model3d: Model3d,
    pub lod_index: usize,
    pub mesh_index: usize,
}

impl BufferKey {
    pub fn new(model3d: Model3d, lod_index: usize, mesh_index: usize) -> Self {
        Self {
            model3d,
            lod_index,
            mesh_index,
        }
    }
}

impl std::fmt::Display for BufferKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:lod{}:mesh{}",
            self.model3d.path, self.lod_index, self.mesh_index
        )
    }
}
