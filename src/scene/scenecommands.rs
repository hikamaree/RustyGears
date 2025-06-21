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

use crate::COMMAND_SENDER;
use crate::Command;
use crate::Game;

use tokio::sync::oneshot;

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

/// A command that computes a result from the game state and sends it back through a one-shot channel.
///
/// Used when a return value is needed from a system running on the main thread.
pub struct CommandWithResult<R> {
    pub run: Box<dyn FnOnce(&mut Game) -> R + Send + Sync>,
    pub respond_to: oneshot::Sender<R>,
}

impl<R: Send + 'static> Command for CommandWithResult<R> {
    fn apply(self: Box<Self>, game: &mut Game) {
        let result = (self.run)(game);
        let _ = self.respond_to.send(result);
    }
}

/// Sends a command to be executed on the main thread and retrieves a result synchronously (if available).
///
/// This is a helper for invoking [`CommandWithResult`] and receiving its result.
///
/// # Arguments
/// * `f` - A closure that reads the game state and returns a value.
///
/// # Returns
/// * `Some(value)` if the command ran successfully and returned a result.
/// * `None` if the global command sender is not available.
///
/// # Example
/// ```
/// let position = send_command_with_result(|game| {
///     game.scene().get_camera_position()
/// });
/// ```
pub fn send_command_with_result<T: Send + 'static>(
    f: impl FnOnce(&mut Game) -> T + Send + Sync + 'static
) -> Option<T> {
    let sender = COMMAND_SENDER.get()?;
    let (tx, mut rx) = oneshot::channel();

    let cmd = CommandWithResult {
        run: Box::new(f),
        respond_to: tx,
    };

    let _ = sender.send(Box::new(cmd));
    rx.try_recv().ok()
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
        if let Some(sender) = $crate::COMMAND_SENDER.get() {
            let entity = $entity;
            let delta = $delta;
            let _ = sender.send(Box::new($crate::CommandFunction {
                run: Box::new(move |game| {
                    let Ok(scene) = game.components.get_mut::<$crate::WorldScene>() else { return; };
                    if let Some(t) = scene.world.get_mut::<$crate::Transform>(entity) {
                        t.position += delta;
                    }
                }),
            }));
        }
    }};
}

/// Applies a quaternion delta to the rotation of the entity's [`Transform`] component.
#[macro_export]
macro_rules! update_entity_rotation {
    ( $entity:expr, $delta:expr ) => {{
        if let Some(sender) = $crate::COMMAND_SENDER.get() {
            let entity = $entity;
            let delta = $delta;
            let _ = sender.send(Box::new($crate::CommandFunction {
                run: Box::new(move |game| {
                    if let Some(t) = game.scene().world.get_mut::<$crate::Transform>(entity) {
                        t.rotation = delta * t.rotation;
                    }
                }),
            }));
        }
    }};
}

/// Adds a new component to an existing entity in the scene.
#[macro_export]
macro_rules! add_component {
    ( $entity:expr, $component:expr ) => {{
        if let Some(sender) = $crate::COMMAND_SENDER.get() {
            let entity = $entity;
            let component = $component;
            let _ = sender.send(Box::new($crate::CommandFunction {
                run: Box::new(move |game| {
                    let Ok(scene) = game.components.get_mut::<$crate::WorldScene>() else { return; };
                    scene.world.insert(entity, component);
                }),
            }));
        }
    }};
}

/// Sets the given entity as the active camera in the scene.
#[macro_export]
macro_rules! set_default_camera {
    ( $camera:expr ) => {{
        if let Some(sender) = $crate::COMMAND_SENDER.get() {
            let camera = $camera;
            let _ = sender.send(Box::new($crate::CommandFunction {
                run: Box::new(move |game| {
                    let Ok(scene) = game.components.get_mut::<$crate::WorldScene>() else { return; };
                    scene.set_active_camera(camera);
                }),
            }));
        }
    }};
}

/// Sets or inserts a new transform for an entity in the scene.
///
/// If the entity already has a transform, it is overwritten. Otherwise, it is inserted.
#[macro_export]
macro_rules! set_instance_transform {
    ( $entity:expr, $transform:expr ) => {{
        if let Some(sender) = $crate::COMMAND_SENDER.get() {
            let entity = $entity;
            let transform = $transform;
            let _ = sender.send(Box::new($crate::CommandFunction {
                run: Box::new(move |game| {
                    let Ok(scene) = game.components.get_mut::<$crate::WorldScene>() else { return; };
                    if let Some(t) = scene.world.get_mut::<$crate::Transform>(entity) {
                        *t = transform;
                    } else {
                        game.scene().world.insert(entity, transform);
                    }
                }),
            }));
        }
    }};
}
