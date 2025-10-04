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
use crate::GameView;
use crate::Model;
use crate::Model3d;
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
        let entity = $crate::Entity::new();
        $crate::send_command($crate::CommandFunction {
            run: Box::new(move |game| {
                let Ok(mut scene) = game.components.get_mut::<$crate::WorldScene>() else { return; };
                $(
                    scene.world.insert(entity, $comp);
                )*
            }),
        });
        entity
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

pub fn add_model3d(name: &str, object: Model) -> Model3d {
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
pub fn load_obj_model(path: &str, game: &GameView) -> Model3d {
    let path_string = path.to_string();
    let game_clone = game.clone();

    let model_handle = Model3d { path: path_string.clone() };
    println!("Start background loading: {}", path_string);

    let path_clone = path_string.clone();

    // let handle = tokio::runtime::Handle::current();
    std::thread::spawn(move || {
        // handle.spawn(async move {
            match Model::from_obj(&path_clone, &game_clone) {
                Ok(model) => {
                    add_model3d(&path_clone, model);
                }
                Err(e) => {
                    crate::log!(crate::LogKind::Error, "Failed to load model '{}': {}", path_clone, e);
                }
            }
            println!("Finished loading: {}", path_clone);
        // });
    });
    println!("majmun zavrsio");

    model_handle
}
