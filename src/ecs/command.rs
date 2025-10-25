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

use tokio::sync::oneshot;
use once_cell::sync::OnceCell;
use crossbeam::channel::Sender;
use crate::Game;

use std::any::Any;

/// Defines execution priority levels for [`Command`]s.
///
/// Commands are executed in order of their priority, from lowest to highest.
/// This allows systems that depend on one another (e.g., physics before camera, camera before render)
/// to control their update order.
///
/// # Variants
/// - `Low`: Executed before all normal and high-priority commands.
/// - `Normal`: Default priority for most commands.
/// - `High`: Executed last; typically used for rendering or post-processing updates.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum CommandPriority {
    /// Executed before all normal and high-priority commands.
    Low,
    /// Default priority for most commands.
    Normal,
    /// Executed last; typically used for rendering or post-processing.
    High,
}

/// A trait that represents a game command that can be executed on a [`Game`] instance.
///
/// Commands are units of work or messages that modify the game state in a controlled manner.
/// They are typically queued by gameplay systems or threads (such as physics, input, or AI),
/// then executed in a central update phase on the main thread.
///
/// Each command:
/// - Is type-erased (`dyn Command`), so different command types can coexist in the same queue.
/// - Is thread-safe (`Send + Sync`).
/// - Has an execution [`priority`](Command::priority) used to control order of application.
///
/// # Example
/// ```
/// struct MoveEntity {
///     entity: Entity,
///     new_position: Vec3,
/// }
///
/// impl Command for MoveEntity {
///     fn apply(self: Box<Self>, game: &mut Game) {
///         if let Some(transform) = game.world.get_mut::<Transform>(self.entity) {
///             transform.position = self.new_position;
///         }
///     }
///
///     fn priority(&self) -> CommandPriority {
///         CommandPriority::Low // Run before camera or rendering
///     }
/// }
/// ```
///
/// # Usage
/// Commands are usually pushed into a thread-safe queue:
/// ```
/// game.queue_command(Box::new(MoveEntity { entity, new_position }));
/// ```
///
/// The main game loop can then drain and execute them in priority order:
/// ```
/// commands.sort_by_key(|c| c.priority());
/// for cmd in commands {
///     cmd.apply(&mut game);
/// }
/// ```
pub trait Command: Any + Send + Sync + 'static {
    /// Applies this command to the given [`Game`] instance.
    ///
    /// This method consumes the boxed command, ensuring one-time application.
    ///
    /// # Parameters
    /// - `game`: The mutable reference to the active [`Game`] instance whose state will be modified.
    fn apply(self: Box<Self>, game: &mut Game);

    /// Returns the execution priority of this command.
    ///
    /// The game loop uses this value to sort commands before applying them.
    /// Default priority is [`CommandPriority::Normal`].
    fn priority(&self) -> CommandPriority {
        CommandPriority::Normal
    }
}

/// Global command sender used for scheduling actions on the main game thread.
///
/// `COMMAND_SENDER` is a globally accessible channel sender used to submit boxed [`Command`] objects
/// for deferred execution on the main game thread. It must be initialized once—typically during
/// game startup—using [`OnceCell::set`].
///
/// This is primarily used to bridge background systems (such as gears or GUI handlers)
/// with the central ECS state by submitting commands from any thread safely.
///
/// # Panics
/// Accessing `COMMAND_SENDER` before initialization will return `None`.
static COMMAND_SENDER: OnceCell<Sender<Box<dyn Command>>> = OnceCell::new();

pub(crate) fn init_command_sender(command_sender: Sender<Box<dyn Command>>) {
    if let Err(e) = COMMAND_SENDER.set(command_sender.clone()) {
        eprintln!("{:?}", e);
    };
}

/// Sends a [`Command`] to the main thread using the global [`COMMAND_SENDER`] channel.
///
/// This function provides a safe wrapper for sending commands without directly accessing
/// the global sender. It will fail silently if the sender has not been initialized,
/// and print a warning to `stderr`.
///
/// # Arguments
/// * `cmd` – An object that implements the [`Command`] trait and owns its data.
///
/// # Example
/// ```rust
/// send_command(ExampleCommand { ... });
/// ```
pub fn send_command(cmd: impl Command + 'static) {
    if let Some(sender) = COMMAND_SENDER.get() {
        let _ = sender.send(Box::new(cmd));
    } else {
        eprintln!("COMMAND_SENDER is not initialized");
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
pub async fn send_command_with_result<T: Send + 'static>(
    f: impl FnOnce(&mut Game) -> T + Send + Sync + 'static
) -> Option<T> {
    let sender = COMMAND_SENDER.get()?;
    let (tx, rx) = oneshot::channel();

    let cmd = CommandWithResult {
        run: Box::new(f),
        respond_to: tx,
    };

    let _ = sender.send(Box::new(cmd));
    rx.await.ok()
}
