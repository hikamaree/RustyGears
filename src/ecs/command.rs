use crate::Game;

/// A trait that represents a game command that can be executed on a [`Game`] instance.
///
/// Implementors of this trait define logic to modify game state.

pub trait Command {

    /// Applies the command to the given game instance.
    ///
    /// This method consumes the boxed command.
    ///
    /// # Parameters
    /// - `game`: A mutable reference to the [`Game`] instance to modify.

    fn apply(self: Box<Self>, game: &mut Game);
}
