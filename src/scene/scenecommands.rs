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

use cgmath::Quaternion;
use cgmath::Vector3;

use crate::Command;
use crate::Entity;
use crate::Game;
use crate::GameView;
use crate::Model;
use crate::Model3d;
use crate::Transform;
use crate::WorldScene;

/// A simple boxed function that mutably operates on the [`Game`] instance.
///
/// This is useful for fire-and-forget commands that don't return a result.
pub struct CommandFunction {
    pub run: Box<dyn FnOnce(&mut Game) + Send + Sync + 'static>,
}

impl Command for CommandFunction {
    fn apply(self: Box<Self>, game: &mut Game) {
        (self.run)(game);
    }
}

impl GameView {
    /// Spawns a new entity with the given set of components.
    ///
    /// Creates a unique [`Entity`], schedules component insertion into the
    /// active [`WorldScene`], and returns the created entity handle.
    ///
    /// The new entity and its components become available during the next update cycle.
    ///
    /// # Example
    /// ```
    /// let entity = game.spawn_entity((Transform::default(), Health { value: 100 }));
    /// ```
    pub fn spawn_entity(&self, components: impl ComponentsTuple) -> Entity {
        let entity = crate::Entity::new();
        let comps = components.into_vec();

        crate::send_command(crate::CommandFunction {
            run: Box::new(move |game| {
                let Ok(mut scene) = game.components.get_mut::<crate::WorldScene>() else {
                    return;
                };
                for comp in comps {
                    comp.insert_into(&mut scene.world, entity);
                }
            }),
        });

        entity
    }

    /// Adds one or more components to an existing entity.
    ///
    /// If a component of the same type already exists, it will be replaced.
    ///
    /// # Example
    /// ```
    /// game.add_components(&player, (Velocity { x: 1.0, y: 0.0 },));
    /// ```
    pub fn add_components(&self, entity: &Entity, components: impl ComponentsTuple) {
        let entity = entity.clone();
        let comps = components.into_vec();

        crate::send_command(crate::CommandFunction {
            run: Box::new(move |game| {
                let Ok(mut scene) = game.components.get_mut::<crate::WorldScene>() else {
                    return;
                };
                for comp in comps {
                    comp.insert_into(&mut scene.world, entity);
                }
            }),
        });
    }

    /// Sets the specified entity as the active camera in the current scene.
    ///
    /// # Example
    /// ```
    /// game.set_default_camera(&camera_entity);
    /// ```
    pub fn set_default_camera(&self, camera: &Entity) {
        let camera = camera.clone();
        crate::send_command(crate::CommandFunction {
            run: Box::new(move |game| {
                let Ok(mut scene) = game.components.get_mut::<crate::WorldScene>() else {
                    return;
                };
                scene.set_active_camera(camera);
            }),
        });
    }

    /// Rotates an entity’s transform by the given quaternion delta.
    ///
    /// If the entity has a [`Transform`] component, its rotation is updated.
    ///
    /// # Example
    /// ```
    /// game.update_entity_rotation(&entity, Quaternion::from_axis_angle(Vector3::Y, 0.1));
    /// ```
    pub fn update_entity_rotation(&self, entity: &Entity, delta: Quaternion<f32>) {
        let entity = entity.clone();
        crate::send_command(crate::CommandFunction {
            run: Box::new(move |game| {
                let Ok(mut scene) = game.components.get_mut::<crate::WorldScene>() else {
                    return;
                };
                if let Some(t) = scene.world.get_mut::<crate::Transform>(entity) {
                    t.rotation = delta * t.rotation;
                }
            }),
        });
    }

    /// Moves an entity by the given position offset.
    ///
    /// If the entity has a [`Transform`] component, its position is updated.
    ///
    /// # Example
    /// ```
    /// game.update_entity_position(&entity, Vector3::new(1.0, 0.0, 0.0));
    /// ```
    pub fn update_entity_position(&self, entity: &Entity, delta: Vector3<f32>) {
        let entity = entity.clone();
        crate::send_command(crate::CommandFunction {
            run: Box::new(move |game| {
                let Ok(mut scene) = game.components.get_mut::<crate::WorldScene>() else {
                    return;
                };
                if let Some(t) = scene.world.get_mut::<crate::Transform>(entity.clone()) {
                    t.position += delta;
                }
            }),
        });
    }

    /// Sets the complete transform of an entity.
    ///
    /// If the entity has a [`Transform`] component, it will be replaced.
    /// Otherwise, a new one is inserted.
    ///
    /// # Example
    /// ```
    /// game.set_entity_transform(&entity, &Transform::from_xyz(0.0, 5.0, 0.0));
    /// ```
    pub fn set_entity_transform(&self, entity: &Entity, transform: &Transform) {
        let entity = entity.clone();
        let transform = transform.clone();
        crate::send_command(crate::CommandFunction {
            run: Box::new(move |game| {
                let Ok(mut scene) = game.components.get_mut::<crate::WorldScene>() else {
                    return;
                };
                if let Some(t) = scene.world.get_mut::<crate::Transform>(entity) {
                    *t = transform;
                } else {
                    scene.world.insert(entity, transform);
                }
            }),
        });
    }

    /// Adds a 3D model to the current scene and returns its corresponding handle.
    ///
    /// Registers a renderable object in the world’s render registry
    /// and returns a [`Model3d`] handle that can be stored as a component.
    ///
    /// # Example
    /// ```
    /// let model = game.add_model3d("tree", tree_model);
    /// entity.insert(model);
    /// ```
    pub fn add_model3d(&self, name: &str, object: Model) -> Model3d {
        let model = Model3d {
            path: name.to_string(),
        };

        let ret = model.clone();

        crate::send_command(crate::CommandFunction {
            run: Box::new(move |game| {
                let Ok(mut scene) = game.components.get_mut::<WorldScene>() else {
                    return;
                };
                scene.add_model3d(model.clone(), object);
            }),
        });

        ret
    }

    /// Loads a `.obj` model from disk and registers it in the scene.
    ///
    /// The model is loaded on a background thread. GPU upload happens automatically
    /// when Graphics processes pending models.
    ///
    /// Returns a [`Model3d`] handle immediately, which can be attached to entities.
    ///
    /// # Example
    /// ```
    /// let tree_model = game.load_obj_model("assets/tree.obj");
    /// entity.insert(tree_model);
    /// ```
    pub fn load_obj_model(&self, path: &str) -> Model3d {
        let path = path.to_string();
        let _game = self.clone();

        let model_handle = Model3d { path: path.clone() };
        let model_handle_clone = model_handle.clone();

        std::thread::spawn(move || {
            match crate::ModelData::from_obj(&path) {
                Ok(model_data) => {
                    crate::send_command(crate::CommandFunction {
                        run: Box::new(move |game| {
                            let Ok(mut scene) = game.components.get_mut::<crate::WorldScene>()
                            else {
                                return;
                            };
                            scene.add_pending_model_data(model_handle_clone.clone(), model_data);
                        }),
                    });
                }
                Err(e) => {
                    crate::log!(
                        crate::LogKind::Error,
                        "Failed to load model '{}': {}",
                        path,
                        e
                    );
                }
            }
            crate::log!(crate::LogKind::Info, "Finished loading: {}", path);
        });

        model_handle
    }
}

/// Represents a type-erased component insertion operation.
///
/// Used internally to insert heterogenous component tuples
/// into the ECS world at runtime.
pub trait ComponentInsert: Send + Sync {
    fn insert_into(self: Box<Self>, world: &mut crate::World, entity: crate::Entity);
}

impl<T: crate::Component> ComponentInsert for T {
    fn insert_into(self: Box<Self>, world: &mut crate::World, entity: crate::Entity) {
        world.insert(entity, *self);
    }
}

/// Represents a tuple of components that can be converted
/// into a vector of boxed [`ComponentInsert`] trait objects.
///
/// This enables flexible syntax such as:
/// ```
/// game.spawn_entity((Transform::default(), Health { value: 100 }));
/// ```
pub trait ComponentsTuple {
    fn into_vec(self) -> Vec<Box<dyn ComponentInsert>>;
}

impl<A: crate::Component> ComponentsTuple for (A,) {
    fn into_vec(self) -> Vec<Box<dyn ComponentInsert>> {
        vec![Box::new(self.0)]
    }
}

impl<A: crate::Component, B: crate::Component> ComponentsTuple for (A, B) {
    fn into_vec(self) -> Vec<Box<dyn ComponentInsert>> {
        vec![Box::new(self.0), Box::new(self.1)]
    }
}

impl<A: crate::Component, B: crate::Component, C: crate::Component> ComponentsTuple for (A, B, C) {
    fn into_vec(self) -> Vec<Box<dyn ComponentInsert>> {
        vec![Box::new(self.0), Box::new(self.1), Box::new(self.2)]
    }
}

impl<A: crate::Component, B: crate::Component, C: crate::Component, D: crate::Component>
    ComponentsTuple for (A, B, C, D)
{
    fn into_vec(self) -> Vec<Box<dyn ComponentInsert>> {
        vec![
            Box::new(self.0),
            Box::new(self.1),
            Box::new(self.2),
            Box::new(self.3),
        ]
    }
}
