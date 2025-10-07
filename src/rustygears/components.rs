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

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

type BoxedComponent = Arc<RwLock<Box<dyn Any + Send + Sync>>>;

/// A type-safe, thread-safe container for globally accessible components.
///
/// `ComponentMap` provides dynamic storage of components indexed by their `TypeId`,
/// allowing for safe shared (`get`) or exclusive (`get_mut`, `with`) access at runtime.
/// Internally, components are stored as boxed `Any` types wrapped in `Arc<RwLock<_>>`,
/// enabling interior mutability and concurrent access across systems.
///
/// # Examples
/// ```rust
/// let mut map = ComponentMap::new();
/// map.insert(MyComponent::default());
///
/// let my_ref = map.get::<MyComponent>().unwrap();
/// println!("{:?}", my_ref.some_field);
///
/// map.with::<MyComponent, _>(|comp| {
///     comp.some_field = 42;
/// }).unwrap();
/// ```
#[derive(Clone)]
pub struct ComponentMap {
    components: HashMap<TypeId, BoxedComponent>,
}

impl ComponentMap {
    /// Creates a new, empty `ComponentMap`.
    ///
    /// # Returns
    /// A new instance of `ComponentMap` without any registered components.
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Inserts a new component into the map.
    ///
    /// If a component of the same type already exists, it will be overwritten.
    ///
    /// # Arguments
    /// * `value` - A value of any `'static + Send + Sync` type to be stored.
    ///
    /// # Example
    /// ```rust
    /// map.insert(MyComponent::default());
    /// ```
    pub fn insert<T: 'static + Send + Sync>(&mut self, value: T) {
        let type_id = TypeId::of::<T>();
        let component = Arc::new(RwLock::new(Box::new(value) as Box<dyn Any + Send + Sync>));
        self.components.insert(type_id, component);
    }

    /// Retrieves a shared (read-only) reference to a stored component.
    ///
    /// # Type Parameters
    /// * `T` - The type of the component to retrieve.
    ///
    /// # Returns
    /// A `ReadGuardWrapper<T>` allowing safe access to the component, or an error string if not found.
    ///
    /// # Errors
    /// Returns an error if the component is not found or the read lock is poisoned.
    pub fn get<T: 'static + Send + Sync>(&'_ self) -> Result<ReadGuardWrapper<'_, T>, String> {
        let type_id = TypeId::of::<T>();
        let arc = self.components.get(&type_id)
            .ok_or_else(|| format!("Component {} not found", std::any::type_name::<T>()))?;
        
        let guard = arc.read()
            .map_err(|_| "Component is poisoned".to_string())?;

        Ok(ReadGuardWrapper {
            guard,
            _marker: std::marker::PhantomData,
        })
    }

    /// Retrieves a mutable (exclusive) reference to a stored component.
    ///
    /// # Type Parameters
    /// * `T` - The type of the component to retrieve mutably.
    ///
    /// # Returns
    /// A `WriteGuardWrapper<T>` allowing mutable access to the component, or an error string if not found.
    ///
    /// # Errors
    /// Returns an error if the component is not found or the write lock is poisoned.
    pub fn get_mut<T: 'static + Send + Sync>(&'_ self) -> Result<WriteGuardWrapper<'_, T>, String> {
        let type_id = TypeId::of::<T>();
        let arc = self.components.get(&type_id)
            .ok_or_else(|| format!("Component {} not found", std::any::type_name::<T>()))?;
        
        let guard = arc.write()
            .map_err(|_| "Component is poisoned".to_string())?;

        Ok(WriteGuardWrapper {
            guard,
            _marker: std::marker::PhantomData,
        })
    }

    /// Executes a closure with mutable access to the requested component.
    ///
    /// This is a convenient shorthand for calling `get_mut`, then applying a closure to the result.
    ///
    /// # Arguments
    /// * `f` - A function that takes a mutable reference to the requested component and returns a value.
    ///
    /// # Type Parameters
    /// * `T` - The type of the component.
    /// * `R` - The return type of the closure.
    ///
    /// # Returns
    /// The result of the closure if the component exists and lock succeeds, otherwise an error string.
    ///
    /// # Example
    /// ```rust
    /// map.with::<MyComponent, _>(|comp| {
    ///     comp.enabled = true;
    /// }).unwrap();
    /// ```
    pub fn with<T: 'static + Send + Sync, R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, String> {
        let mut guard = self.get_mut::<T>()?;
        Ok(f(&mut *guard))
    }
}

/// Wrapper around a read lock on a component of type `T`.
///
/// Dereferences directly to `&T`, after performing a safe downcast.
pub struct ReadGuardWrapper<'a, T> {
    guard: RwLockReadGuard<'a, Box<dyn Any + Send + Sync>>,
    _marker: std::marker::PhantomData<T>,
}

unsafe impl<'a, T: Send> Send for ReadGuardWrapper<'a, T> {}

impl<'a, T: 'static> Deref for ReadGuardWrapper<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.guard.downcast_ref::<T>()
            .expect("TypeId matched but downcast failed")
    }
}

/// Wrapper around a write lock on a component of type `T`.
///
/// Dereferences directly to `&mut T`, after performing a safe downcast.
pub struct WriteGuardWrapper<'a, T> {
    guard: RwLockWriteGuard<'a, Box<dyn Any + Send + Sync>>,
    _marker: std::marker::PhantomData<T>,
}

unsafe impl<'a, T: Send> Send for WriteGuardWrapper<'a, T> {}

impl<'a, T: 'static> Deref for WriteGuardWrapper<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.guard.downcast_ref::<T>()
            .expect("TypeId matched but downcast failed")
    }
}

impl<'a, T: 'static> DerefMut for WriteGuardWrapper<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.guard.downcast_mut::<T>()
            .expect("TypeId matched but downcast failed")
    }
}
