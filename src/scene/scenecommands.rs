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
            let scene = match game.components.get_mut::<$crate::WorldScene>() {
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

/// Adds a delta to the position of the given entity's [`Transform`] component.
#[macro_export]
macro_rules! update_entity_position {
    ( $entity:expr, $delta:expr ) => {{
        let entity = $entity;
        let delta = $delta;
        $crate::send_command($crate::CommandFunction {
            run: Box::new(move |game| {
                let Ok(scene) = game.components.get_mut::<$crate::WorldScene>() else { return; };
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

/// Adds a new component to an existing entity in the scene.
#[macro_export]
macro_rules! add_component {
    ( $entity:expr, $component:expr ) => {{
        let entity = $entity;
        let component = $component;
        $crate::send_command($crate::CommandFunction {
            run: Box::new(move |game| {
                let Ok(scene) = game.components.get_mut::<$crate::WorldScene>() else { return; };
                scene.world.insert(entity, component);
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
                let Ok(scene) = game.components.get_mut::<$crate::WorldScene>() else { return; };
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
