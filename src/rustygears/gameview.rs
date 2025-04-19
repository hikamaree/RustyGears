use crate::Game;
use crate::Gear;
use crate::Graphics;
use crate::Time;
use crate::Scene;
use std::sync::Arc;
use std::sync::Mutex;
use std::any::Any;
use std::collections::HashMap;

/// A read-only view of the current game state, provided to [`Gear`]s when handling events.
///
/// `GameView` allows gears to inspect parts of the game world such as other gears, graphics state, timing information,
/// and the scene. It provides safe access without allowing direct mutation of the game.
///
/// Typically passed to [`Gear::handle_event`].

pub struct GameView<'a> {
    /// A reference to the collection of all gears in the game, indexed by ID.
    pub gears: &'a HashMap<String, Arc<Mutex<dyn Gear>>>,

    /// A reference to the graphics context, used for rendering-related information or operations.
    pub graphics: &'a Graphics,

    /// A reference to the time subsystem, providing timing and delta-time information.
    pub time: &'a Time,

    /// A reference to the current scene, which may contain entities or spatial information.
    pub scene: &'a Scene,
}

impl<'a> GameView<'a> {

    /// Creates a [`GameView`] from a reference to the current [`Game`] instance.
    ///
    /// This is typically used internally when dispatching events to gears.
    ///
    /// # Panics
    /// Panics if the `graphics` field of the game is not initialized.
    
    pub fn create(game: &'a Game) -> Self {
        Self {
            gears: &game.gears,
            graphics: &game.graphics.as_ref().expect("ERROR: Graphics is not initialized"),
            time: &game.time,
            scene: &game.scene,
        }
    }

    /// Retrieves a gear by ID and invokes a function on it, if the gear exists and matches the expected type.
    ///
    /// # Type Parameters
    /// - `T`: The expected concrete type of the gear.
    /// - `R`: The return type of the function `f`.
    ///
    /// # Parameters
    /// - `id`: The string identifier of the gear.
    /// - `f`: A function to invoke on the gear of type `T`.
    ///
    /// # Returns
    /// Returns `Some(R)` if the gear exists and the type matches. Returns `None` otherwise.
    ///
    /// # Example
    /// ```
    /// view.use_gear::<MyGear, _>("my_gear", |gear| {
    ///     gear.do_something();
    /// });
    /// ```

    pub fn use_gear<T: Gear + 'static, R>(&self, id: &str, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        let gear = self.gears.get(id)?;
        let mut lock = gear.lock().unwrap();
        let any = &mut *lock as &mut dyn Any;
        let typed_gear = any.downcast_mut::<T>()?;
        Some(f(typed_gear))
    }
}
