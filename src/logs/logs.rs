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


use crate::Game;
use crate::Command;

/// Represents a categorized log buffer used for runtime diagnostics,
/// GUI output, and structured logging within the engine.
///
/// The buffer stores log messages tagged with a `LogKind`
/// indicating their category or severity.
///
/// This component is typically inserted into the `Game`'s `ComponentMap`
/// and accessed by systems or GUI panels (e.g., terminal console).
#[derive(Default)]
pub struct Logs {
    buffer: Vec<(LogKind, String)>,
}

impl Logs {
    /// Appends a new log entry to the buffer with the given kind and message.
    ///
    /// # Parameters
    /// - `kind`: The type of log (info, error, etc.)
    /// - `msg`: Any type convertible into a `String`, representing the message text.
    ///
    /// # Example
    /// ```
    /// logs.log(LogKind::Info, "Engine started.");
    /// ```
    pub fn log(&mut self, kind: LogKind, msg: impl Into<String>) {
        self.buffer.push((kind, msg.into()));
    }

    /// Consumes the current buffer contents, returning all log entries and
    /// clearing the internal buffer.
    ///
    /// # Returns
    /// A vector of all `(LogKind, String)` entries that were in the buffer.
    ///
    /// # Example
    /// ```
    /// let entries = logs.drain();
    /// assert!(logs.all().is_empty());
    /// ```
    pub fn drain(&mut self) -> Vec<(LogKind, String)> {
        std::mem::take(&mut self.buffer)
    }

    /// Returns an immutable reference to all current log entries in the buffer.
    ///
    /// This can be used to display logs without clearing them.
    ///
    /// # Returns
    /// A slice of `(LogKind, String)` pairs representing the buffered logs.
    ///
    /// # Example
    /// ```
    /// for (kind, msg) in logs.all() {
    ///     println!("[{:?}] {}", kind, msg);
    /// }
    /// ```
    pub fn all(&self) -> &[(LogKind, String)] {
        &self.buffer
    }
}

/// Represents the severity or category of a log entry.
///
/// This enum is used to classify messages for filtering, coloring,
/// or routing to appropriate destinations.
#[derive(Debug, Clone)]
pub enum LogKind {
    Info,
    Warning,
    Error,
    Input,
    Output,
    Debug,
}

/// A concrete [`Command`] implementation used to emit log messages
/// into the global [`Logs`] component within the engine.
///
/// `LogCommand` is typically created by the [`log!`] macro or manually
/// when structured logging is needed from outside the main thread or system.
///
/// This command queues a `(LogKind, String)` pair to be appended
/// to the `Logs` buffer during the game update cycle.
///
/// # Fields
/// - `kind`: The type of log message (e.g., Info, Error, Debug).
/// - `msg`: The textual content of the log.
pub struct LogCommand {
    pub kind: LogKind,
    pub msg: String,
}

impl Command for LogCommand {
    fn apply(self: Box<Self>, game: &mut Game) {
        if let Ok(logs) = game.components.get_mut::<Logs>() {
            logs.log(self.kind, self.msg);
        }
    }
}

/// Logs a message of a given kind via the global `COMMAND_SENDER`
/// by dispatching a `LogCommand`.
///
/// This macro queues a command that will insert a log message
/// into the `Logs` component during the next update tick.
///
/// # Parameters
/// - `$kind`: The [`LogKind`] variant to classify the message.
/// - `$msg+`: The format string and arguments (like `println!`).
///
/// # Requirements
/// - `COMMAND_SENDER` must be initialized and globally accessible via `OnceCell`.
#[macro_export]
macro_rules! log {
    ( $kind:expr, $($msg:tt)+ ) => {{
        if let Some(sender) = $crate::COMMAND_SENDER.get() {
            let _ = sender.send(Box::new($crate::LogCommand {
                kind: $kind.clone(),
                msg: format!($($msg)+),
            }));
        } else {
            eprintln!("COMMAND_SENDER is not initialized");
        }
    }};
}
