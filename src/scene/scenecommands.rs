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

use tokio::sync::oneshot;

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

#[macro_export]
macro_rules! spawn_entity {
    ( $sender:expr, $( $comp:expr ),* $(,)? ) => {{
        $sender.send_command(move |game| {
            let mut builder = game.scene().spawn();
            $(
                builder = builder.with($comp);
            )*
            builder.build()
        })
    }};
}

#[macro_export]
macro_rules! update_entity_position {
    ( $sender:expr, $entity:expr, $delta:expr ) => {{
        let entity = $entity;
        let delta = $delta;
        $sender.send_command(move |game| {
            if let Some(t) = game.scene().world.get_mut::<Transform>(entity) {
                t.position += delta;
            }
        })
    }};
}

#[macro_export]
macro_rules! update_entity_rotation {
    ( $sender:expr, $entity:expr, $delta:expr ) => {{
        let entity = $entity;
        let delta = $delta;
        $sender.send_command(move |game| {
            if let Some(t) = game.scene().world.get_mut::<Transform>(entity) {
                t.rotation = delta * t.rotation;
            }
        })
    }};
}

#[macro_export]
macro_rules! add_component {
    ( $sender:expr, $entity:expr, $component:expr ) => {{
        let entity = $entity;
        let component = $component;
        $sender.send_command(move |game| {
            game.scene().world.insert(entity, component);
        })
    }};
}

#[macro_export]
macro_rules! set_default_camera {
    ( $sender:expr, $camera:expr ) => {{
        let camera = $camera;
        $sender.send_command(move |game| {
            game.scene().set_active_camera(camera);
        })
    }};
}

#[macro_export]
macro_rules! set_instance_transform {
    ( $sender:expr, $entity:expr, $transform:expr ) => {{
        let entity = $entity;
        let transform = $transform;
        $sender.send_command(move |game| {
            if let Some(t) = game.scene().world.get_mut::<Transform>(entity) {
                *t = transform;
            } else {
                game.scene().world.insert(entity, transform);
            }
        })
    }};
}
