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

use cgmath::{
    Vector3,
    Point3,
};
use crate::collision_box::BoundingBox;

#[allow(dead_code)]
pub struct Octree {
    boundary: BoundingBox,
    capacity: usize,
    bodies: Vec<usize>,
    children: Option<[Box<Octree>; 8]>,
}

#[allow(dead_code)]
impl Octree {
    pub fn new(boundary: BoundingBox, capacity: usize) -> Self {
        Octree {
            boundary,
            capacity,
            bodies: Vec::new(),
            children: None,
        }
    }

    pub fn insert(&mut self, index: usize, position: Point3<f32>) -> bool {
        if !self.boundary.contains(&position) {
            return false;
        }

        if self.bodies.len() < self.capacity {
            self.bodies.push(index);
            return true;
        }

        if self.children.is_none() {
            self.subdivide();
        }

        for child in self.children.as_mut().unwrap().iter_mut() {
            if child.insert(index, position) {
                return true;
            }
        }

        false
    }

    fn subdivide(&mut self) {
        let half_size = self.boundary.size() / 2.0;
        let center = self.boundary.center();

        let offsets = [
            Vector3::new(-half_size.x, -half_size.y, -half_size.z),
            Vector3::new(half_size.x, -half_size.y, -half_size.z),
            Vector3::new(-half_size.x, half_size.y, -half_size.z),
            Vector3::new(half_size.x, half_size.y, -half_size.z),
            Vector3::new(-half_size.x, -half_size.y, half_size.z),
            Vector3::new(half_size.x, -half_size.y, half_size.z),
            Vector3::new(-half_size.x, half_size.y, half_size.z),
            Vector3::new(half_size.x, half_size.y, half_size.z),
        ];

        let children = [
            Box::new(Octree::new(
                BoundingBox::new(center + offsets[0], center + offsets[0] + half_size),
                self.capacity,
            )),
            Box::new(Octree::new(
                BoundingBox::new(center + offsets[1], center + offsets[1] + half_size),
                self.capacity,
            )),
            Box::new(Octree::new(
                BoundingBox::new(center + offsets[2], center + offsets[2] + half_size),
                self.capacity,
            )),
            Box::new(Octree::new(
                BoundingBox::new(center + offsets[3], center + offsets[3] + half_size),
                self.capacity,
            )),
            Box::new(Octree::new(
                BoundingBox::new(center + offsets[4], center + offsets[4] + half_size),
                self.capacity,
            )),
            Box::new(Octree::new(
                BoundingBox::new(center + offsets[5], center + offsets[5] + half_size),
                self.capacity,
            )),
            Box::new(Octree::new(
                BoundingBox::new(center + offsets[6], center + offsets[6] + half_size),
                self.capacity,
            )),
            Box::new(Octree::new(
                BoundingBox::new(center + offsets[7], center + offsets[7] + half_size),
                self.capacity,
            )),
        ];

        self.children = Some(children);
    }

    pub fn query(&self, range: &BoundingBox, found: &mut Vec<usize>) {
        if !self.boundary.intersects(range) {
            return;
        }

        for &index in &self.bodies {
            found.push(index);
        }

        if let Some(children) = &self.children {
            for child in children.iter() {
                child.query(range, found);
            }
        }
    }
}

