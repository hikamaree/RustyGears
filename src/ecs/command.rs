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

/// A trait that represents a game command that can be executed on a [`Game`] instance.
///
/// Implementors of this trait define logic to modify game state.
pub trait Command: Any + Send + Sync + 'static {

    /// Applies the command to the given game instance.
    ///
    /// This method consumes the boxed command.
    ///
    /// # Parameters
    /// - `game`: A mutable reference to the [`Game`] instance to modify.
    fn apply(self: Box<Self>, game: &mut Game);
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
