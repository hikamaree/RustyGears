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

use crate::Command;
use crate::Game;

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

/// Spawns a new entity into the scene with the given components,
/// and returns its entity ID synchronously.
///
/// # Example
/// ```
/// let entity = spawn_entity!(
///     Transform::default(),
///     Model3d { path: "tree".into() }
/// );
/// ```
#[macro_export]
macro_rules! spawn_entity {
    ( $( $comp:expr ),* $(,)? ) => {{
        $crate::send_command_with_result(move |game| {
            let mut scene = match game.components.get_mut::<$crate::WorldScene>() {
                Ok(scene) => scene,
                Err(_) => todo!(),
            };

            let mut builder = scene.spawn();
            $(
                builder = builder.with($comp);
            )*
            builder.build()
        })
    }};
}

/// Adds one or more components to an existing entity.
///
/// This macro sends a command to the ECS world to attach additional components
/// to the specified entity. Each component is inserted individually.
///
/// # Example
/// ```
/// insert_components!(entity, Transform::default(), MyComponent { ... });
/// ```
#[macro_export]
macro_rules! add_components {
    ( $entity:expr, $( $comp:expr ),* $(,)? ) => {{
        let entity = $entity;
        $crate::send_command($crate::CommandFunction {
            run: Box::new(move |game| {
                let Ok(mut scene) = game.components.get_mut::<$crate::WorldScene>() else { return; };
                $(
                    scene.world.insert(entity, $comp);
                )*
            }),
        });
    }};
}

/// Adds a delta to the position of the given entity's [`Transform`] component.
#[macro_export]
macro_rules! update_entity_position {
    ( $entity:expr, $delta:expr ) => {{
        let entity = $entity;
        let delta = $delta;
        $crate::send_command($crate::CommandFunction {
            run: Box::new(move |game| {
                let Ok(mut scene) = game.components.get_mut::<$crate::WorldScene>() else {
                    return;
                };
                if let Some(t) = scene.world.get_mut::<$crate::Transform>(entity) {
                    t.position += delta;
                }
            }),
        });
    }};
}

/// Applies a quaternion delta to the rotation of the entity's [`Transform`] component.
#[macro_export]
macro_rules! update_entity_rotation {
    ( $entity:expr, $delta:expr ) => {{
        let entity = $entity;
        let delta = $delta;
        $crate::send_command($crate::CommandFunction {
            run: Box::new(move |game| {
                if let Some(t) = game.scene().world.get_mut::<$crate::Transform>(entity) {
                    t.rotation = delta * t.rotation;
                }
            }),
        });
    }};
}

/// Sets the given entity as the active camera in the scene.
#[macro_export]
macro_rules! set_default_camera {
    ( $camera:expr ) => {{
        let camera = $camera;
        $crate::send_command($crate::CommandFunction {
            run: Box::new(move |game| {
                let Ok(mut scene) = game.components.get_mut::<$crate::WorldScene>() else { return; };
                scene.set_active_camera(camera);
            }),
        });
    }};
}

/// Sets or inserts a new transform for an entity in the scene.
///
/// If the entity already has a transform, it is overwritten. Otherwise, it is inserted.
#[macro_export]
macro_rules! set_instance_transform {
    ( $entity:expr, $transform:expr ) => {{
        let entity = $entity;
        let transform = $transform;
        $crate::send_command($crate::CommandFunction {
            run: Box::new(move |game| {
                let Ok(scene) = game.components.get_mut::<$crate::WorldScene>() else { return; };
                if let Some(t) = scene.world.get_mut::<$crate::Transform>(entity) {
                    *t = transform;
                } else {
                    game.scene().world.insert(entity, transform);
                }
            }),
        });
    }};
}

/// Adds a `RenderObject` to the current scene and returns its corresponding `Model3d` handle.
///
/// This macro registers a renderable object (with its LODs) into the scene’s
/// `models3d` map and returns a `Model3d` that can be used for components like `Model3d`.
///
/// # Arguments
/// - `$name`: A string literal or expression representing the model name (e.g. `"tree"`)
/// - `$object`: An already constructed `RenderObject`
///
/// # Example
/// ```
/// let tree_model = add_render_object!("tree", tree_render_object);
/// commands.spawn().insert(tree_model);
/// ```
#[macro_export]
macro_rules! add_model3d {
    ( $name:expr, $object:expr ) => {{
        match $crate::send_command_with_result(move |game| {
            let Ok(mut scene) = game.components.get_mut::<$crate::WorldScene>() else {
                return Err("No WorldScene registered".to_string());
            };

            let model = $crate::Model3d {
                path: $name.to_string(),
            };

            scene.add_model3d(model.clone(), $object);

            Ok(model)
        }).await {
            Some(Ok(model)) => Some(model),
            Some(Err(e)) => {
                log!($crate::LogKind::Error, "Failed to insert render object: {}", e);
                None
            }
            None => {
                log!($crate::LogKind::Error, "send_command_with_result failed to execute");
                None
            }
        }
    }};
}

/// Loads a `.obj` model and registers it in the scene’s render registry,
/// returning the corresponding `Model3d` handle.
///
/// This macro first loads the model using [`Model::from_obj`], then constructs a
/// [`RenderObject`] from it, and finally calls [`add_model3d!`] to insert it into the ECS.
///
/// # Arguments
/// - `$path`: Relative path to the `.obj` file (e.g., `"tree/tree.obj"`)
/// - `$game`: A `&GameView` reference used for GPU access
///
/// # Returns
/// - `Some(Model3d)` on success
/// - `None` on error (with logging)
///
/// # Example
/// ```
/// let Some(model3d) = load_obj_model!("tree/tree.obj", game).await else {
///     return;
/// };
/// ```
///
#[macro_export]
macro_rules! load_obj_model {
    ( $path:expr, $game:expr ) => {{
        let model_result = $crate::Model::from_obj($path, $game).await;
        let model = match model_result {
            Ok(m) => m,
            Err(e) => {
                log!($crate::LogKind::Error, "Failed to load model '{}': {}", $path, e);
                Model::default()
            }
        };

        $crate::add_model3d!($path, model)
    }};
}
