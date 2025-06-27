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

use std::collections::HashMap;
use std::cell::UnsafeCell;
use std::any::Any;
use std::any::TypeId;
use std::sync::Arc;

/// A type-erased container for a component, stored behind `Arc<UnsafeCell<...>>` to allow
/// interior mutability without requiring `&mut self` access to `ComponentMap`.
///
/// Each `BoxedComponent` is expected to contain a unique component type,
/// stored as a concrete instance of `T: Any + Send + Sync`.
type BoxedComponent = Arc<UnsafeCell<dyn Any + Send + Sync>>;

/// A central registry for globally accessible components in the game engine.
///
/// `ComponentMap` allows storing and retrieving type-unique components in a type-safe manner,
/// using runtime type IDs as keys. It enables both immutable and mutable access to components,
/// and supports temporary scoped mutable access via a closure.
///
/// # Safety
/// Internally uses `UnsafeCell` for interior mutability and performs `Any` type casting via `unsafe`
/// blocks. All usage assumes the user ensures no aliasing of mutable references.
///
/// # Example
/// ```rust
/// game.components.insert(Time::new());
///
/// game.components.with::<Time, _>(|time| {
///     time.update();
/// });
/// ```
pub struct ComponentMap {
    components: HashMap<TypeId, BoxedComponent>,
}

impl ComponentMap {
    /// Creates an empty `ComponentMap` with no registered components.
    ///
    /// # Returns
    /// A new `ComponentMap` instance.
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Inserts a new component of type `T` into the map, replacing any existing component of the same type.
    ///
    /// Components are stored under their `TypeId`, and only one instance of a given type can exist in the map.
    ///
    /// # Arguments
    /// * `value` - A value of type `T` implementing `Send + Sync + 'static`.
    pub fn insert<T: 'static + Send + Sync>(&mut self, value: T) {
        let type_id = TypeId::of::<T>();
        let component = Arc::new(UnsafeCell::new(value)) as BoxedComponent;
        self.components.insert(type_id, component);
    }

    /// Returns an immutable reference to the component of type `T`, if it exists.
    ///
    /// # Errors
    /// Returns a `String` error if the component is not found or has the wrong type.
    pub fn get<T: 'static + Send + Sync>(&self) -> Result<&T, String> {
        let type_id = TypeId::of::<T>();

        let arc = self.components.get(&type_id)
            .ok_or_else(|| format!("Component of type {} not found", std::any::type_name::<T>()))?;

        let raw = arc.get();

        unsafe {
            (&*raw).downcast_ref::<T>()
                .ok_or_else(|| format!("Component type mismatch for {}", std::any::type_name::<T>()))
        }
    }

    /// Returns a mutable reference to the component of type `T`, if it exists.
    ///
    /// # Safety
    /// This method returns a `&mut T` from behind an `UnsafeCell`. The caller must ensure
    /// that no other mutable or immutable references to the same component exist at the same time.
    ///
    /// # Errors
    /// Returns a `String` error if the component is not found or has the wrong type.
    pub fn get_mut<T: 'static + Send + Sync>(&self) -> Result<&mut T, String> {
        let type_id = TypeId::of::<T>();

        let arc = self.components.get(&type_id)
            .ok_or_else(|| format!("Component of type {} not found", std::any::type_name::<T>()))?;

        let raw = arc.get();

        unsafe {
            (&mut *raw).downcast_mut::<T>()
                .ok_or_else(|| format!("Component type mismatch for {}", std::any::type_name::<T>()))
        }
    }

    /// Returns a view of all registered components as a map from `TypeId` to static references.
    ///
    /// # Safety
    /// The returned references are cast as `'static`, but they rely on the assumption that
    /// the underlying components outlive all uses of this function's result.
    pub fn get_view(&self) -> HashMap<TypeId, &'static (dyn Any + Send + Sync + 'static)> {
        let mut view = HashMap::new();

        for (type_id, arc_cell_any) in &self.components {
            let ptr = arc_cell_any.get();

            let reference: &'static (dyn Any + Send + Sync) = unsafe {
                &*ptr
            };

            view.insert(*type_id, reference);
        }

        view
    }

    /// Temporarily borrows a component mutably for the duration of the provided closure.
    ///
    /// This method avoids aliasing problems by confining the `&mut T` borrow to the closure scope,
    /// allowing safe usage even when other parts of the program also access `ComponentMap`.
    ///
    /// # Arguments
    /// * `f` - A closure that receives a mutable reference to the component of type `T`.
    ///
    /// # Returns
    /// Returns the result of the closure, or a `String` error if the component is missing or mismatched.
    ///
    /// # Example
    /// ```rust
    /// game.components.with::<Time, f32>(|time| {
    ///     time.delta_time()
    /// })?;
    /// ```
    pub fn with<T: 'static + Send + Sync, R>(
        &self,
        f: impl FnOnce(&mut T) -> R,
    ) -> Result<R, String> {
        let type_id = TypeId::of::<T>();

        let arc = self.components
            .get(&type_id)
            .ok_or_else(|| format!("Component {} not found", std::any::type_name::<T>()))?;

        let cell = arc.get();

        let ptr = unsafe { &mut *cell };

        let casted = ptr
            .downcast_mut::<T>()
            .ok_or_else(|| format!("Type mismatch for {}", std::any::type_name::<T>()))?;

        Ok(f(casted))
    }
}
