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

use std::collections::HashMap;

use crate::Camera;
use crate::CameraControl;
use crate::Entity;
use crate::EntityBuilder;
use crate::Gui;
use crate::Model;
use crate::ModelData;
use crate::Transform;
use crate::World;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Model3d {
    pub path: String,
}

/// Represents the state of a scene in the game engine using an ECS-based architecture.
///
/// `WorldScene` holds all entities, components, renderable objects,
/// active camera information, and GUI elements. It acts as the central
/// container for both logic and rendering data tied to the current scene.
///
/// This structure is intended to be used per scene or level and is
/// responsible for managing entities, components, and render metadata.
///
/// # Fields
/// - `world`: ECS `World` containing all entities and their components.
/// - `render_objects`: A map from string identifiers to `RenderObject` values,
///   which contain geometry and LOD information for rendering.
/// - `active_camera`: The currently active camera entity, if any.
/// - `render_gui`: A list of GUI components to be rendered, each implementing the `Gui` trait.
#[derive(Default)]
pub struct WorldScene {
    pub world: World,
    pub models3d: HashMap<Model3d, Model>,
    pub pending_model_data: HashMap<Model3d, ModelData>,
    pub active_camera: Option<Entity>,
    pub render_gui: Vec<Box<dyn Gui + Send + Sync>>,
}

impl WorldScene {
    /// Adds a new `RenderObject` to the scene.
    ///
    /// # Parameters
    /// - `name`: A unique string identifier for the render object.
    /// - `object`: The `RenderObject` to add.
    pub fn add_model3d(&mut self, name: Model3d, model: Model) {
        self.models3d.insert(name, model);
    }

    /// Adds CPU-side model data that needs GPU upload.
    ///
    /// # Parameters
    /// - `name`: A unique string identifier for the model.
    /// - `data`: The CPU-side model data.
    pub fn add_pending_model_data(&mut self, name: Model3d, data: ModelData) {
        self.pending_model_data.insert(name, data);
    }

    /// Drains all pending model data for GPU upload.
    ///
    /// # Returns
    /// - Iterator of (Model3d, ModelData) pairs.
    pub fn drain_pending_model_data(
        &mut self,
    ) -> std::collections::hash_map::Drain<'_, Model3d, ModelData> {
        self.pending_model_data.drain()
    }

    /// Retrieves a reference to a render object by name.
    ///
    /// # Parameters
    /// - `name`: The name of the render object to retrieve.
    ///
    /// # Returns
    /// - `Some(&RenderObject)` if found, or `None` otherwise.
    pub fn get_model3d(&self, name: &Model3d) -> Option<&Model> {
        self.models3d.get(name)
    }

    /// Spawns a new entity in the scene's world.
    ///
    /// This method returns an `EntityBuilder` that allows for fluent-style construction
    /// of the entity with its components before finalizing.
    ///
    /// # Returns
    /// - `EntityBuilder` for chaining component insertions.
    pub fn spawn(&mut self) -> EntityBuilder<'_> {
        let entity = Entity::new();
        EntityBuilder {
            world: &mut self.world,
            entity,
        }
    }

    /// Sets the active camera for the scene if the provided entity has a `Camera` component.
    ///
    /// # Parameters
    /// - `entity`: The entity to set as the active camera.
    pub fn set_active_camera(&mut self, entity: Entity) {
        if self.world.has::<Camera>(entity) {
            self.active_camera = Some(entity);
        }
    }

    /// Adds a new camera to the world.
    ///
    /// If no active camera is currently set, this camera becomes the active one.
    ///
    /// # Parameters
    /// - `camera`: The camera component to add.
    ///
    /// # Returns
    /// - The created `Entity` handle.
    pub fn add_camera(&mut self, camera: Camera) -> Entity {
        let entity = self.world.spawn();
        self.world.insert(entity, camera);
        if self.active_camera.is_none() {
            self.active_camera = Some(entity);
        }
        entity
    }

    /// Retrieves a reference to the active camera, if one exists.
    ///
    /// # Returns
    /// - `Some(&Camera)` if an active camera is set and found, or `None` otherwise.
    pub fn active_camera(&self) -> Option<&Camera> {
        self.active_camera.and_then(|e| self.world.get::<Camera>(e))
    }

    /// Retrieves a mutable reference to the active camera, if one exists.
    ///
    /// # Returns
    /// - `Some(&mut Camera)` if an active camera is set and found, or `None` otherwise.
    pub fn active_camera_mut(&mut self) -> Option<&mut Camera> {
        self.active_camera
            .and_then(|e| self.world.get_mut::<Camera>(e))
    }

    /// Retrieves a reference to a specific camera component by entity.
    ///
    /// # Parameters
    /// - `entity`: The entity to look up.
    ///
    /// # Returns
    /// - `Some(&Camera)` if found, or `None` otherwise.
    pub fn get_camera(&self, entity: Entity) -> Option<&Camera> {
        self.world.get::<Camera>(entity)
    }

    /// Retrieves a mutable reference to a specific camera component by entity.
    ///
    /// # Parameters
    /// - `entity`: The entity to look up.
    ///
    /// # Returns
    /// - `Some(&mut Camera)` if found, or `None` otherwise.
    pub fn get_camera_mut(&mut self, entity: Entity) -> Option<&mut Camera> {
        self.world.get_mut::<Camera>(entity)
    }

    /// Computes the world-space transform for the given camera entity.
    ///
    /// If the camera has an associated `CameraControl` component, its `handler`
    /// will be used to generate the transform dynamically.
    /// Otherwise, the method falls back to returning the `Transform` component of the entity.
    ///
    /// If neither component is present, the identity transform is returned.
    ///
    /// # Parameters
    /// - `entity`: The entity representing the camera.
    ///
    /// # Returns
    /// - `Transform` representing the camera's world-space transform.
    pub fn get_camera_transform(&self, entity: Entity) -> Transform {
        if let Some(control) = self.world.get::<CameraControl>(entity) {
            return control.handler.get_camera_transform(&self.world, entity);
        }

        match self.world.get::<Transform>(entity) {
            Some(transform) => transform.clone(),
            None => Transform::identity(),
        }
    }

    /// Adds a GUI element to the scene.
    ///
    /// GUI elements are drawn after the main 3D scene and are used for overlays,
    /// HUDs, editors, and more. GUI objects must implement the `Gui` trait.
    ///
    /// # Parameters
    /// - `gui`: The GUI component to add.
    ///
    /// # Returns
    /// - A mutable reference to self, enabling method chaining.
    pub fn add_gui<T: Gui + 'static>(&mut self, gui: T) -> &mut Self {
        self.render_gui.push(Box::new(gui));
        self
    }
}
