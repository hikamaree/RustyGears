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

use cgmath::InnerSpace;
use cgmath::Matrix4;
use cgmath::Vector3;

#[derive(Debug, Clone)]
pub struct Aabb {
    pub half_extents: Vector3<f32>,
}

#[derive(Debug, Clone)]
pub struct Sphere {
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub struct Capsule {
    pub radius: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct TriangleMesh {
    pub vertices: Vec<Vector3<f32>>,
    pub indices: Vec<[u32; 3]>,
}

#[derive(Debug, Clone)]
pub enum ColliderShape {
    Aabb(Aabb),
    Sphere(Sphere),
    Capsule(Capsule),
    Mesh(TriangleMesh),
}


pub fn intersect_sphere_sphere(a: &Sphere, ta: &Matrix4<f32>, b: &Sphere, tb: &Matrix4<f32>) -> Option<Vector3<f32>> {
    let pa = ta.w.truncate();
    let pb = tb.w.truncate();
    let delta = pb - pa;
    let dist2 = delta.magnitude2();
    let r = a.radius + b.radius;

    if dist2 <= r * r && dist2 != 0.0 {
        Some(delta.normalize())
    } else {
        None
    }
}

pub fn intersect_sphere_aabb(s: &Sphere, ts: &Matrix4<f32>, aabb: &Aabb, taabb: &Matrix4<f32>) -> Option<Vector3<f32>> {
    let center = ts.w.truncate();
    let box_center = taabb.w.truncate();
    let rel = center - box_center;
    let clamped = Vector3::new(
        rel.x.clamp(-aabb.half_extents.x, aabb.half_extents.x),
        rel.y.clamp(-aabb.half_extents.y, aabb.half_extents.y),
        rel.z.clamp(-aabb.half_extents.z, aabb.half_extents.z),
    );

    let closest = box_center + clamped;
    let delta = center - closest;
    let dist2 = delta.magnitude2();

    if dist2 <= s.radius * s.radius && dist2 != 0.0 {
        Some(delta.normalize())
    } else {
        None
    }
}

pub fn intersect_aabb_aabb(a: &Aabb, ta: &Matrix4<f32>, b: &Aabb, tb: &Matrix4<f32>) -> Option<Vector3<f32>> {
    let ca = ta.w.truncate();
    let cb = tb.w.truncate();
    let delta = cb - ca;

    let amin = ca - a.half_extents;
    let amax = ca + a.half_extents;
    let bmin = cb - b.half_extents;
    let bmax = cb + b.half_extents;

    let overlap_x = (amax.x - bmin.x).min(bmax.x - amin.x);
    let overlap_y = (amax.y - bmin.y).min(bmax.y - amin.y);
    let overlap_z = (amax.z - bmin.z).min(bmax.z - amin.z);

    if amax.x < bmin.x || amin.x > bmax.x ||
       amax.y < bmin.y || amin.y > bmax.y ||
       amax.z < bmin.z || amin.z > bmax.z {
        return None;
    }

    let (axis, _) = [
        (Vector3::unit_x() * delta.x.signum(), overlap_x),
        (Vector3::unit_y() * delta.y.signum(), overlap_y),
        (Vector3::unit_z() * delta.z.signum(), overlap_z),
    ].iter().cloned().min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();

    Some(axis)
}

pub fn intersect_sphere_capsule(s: &Sphere, ts: &Matrix4<f32>, c: &Capsule, tc: &Matrix4<f32>) -> Option<Vector3<f32>> {
    let sphere_center = ts.w.truncate();
    let cap_base = tc.w.truncate() - Vector3::unit_y() * (c.height / 2.0);
    let cap_tip = tc.w.truncate() + Vector3::unit_y() * (c.height / 2.0);

    let closest = project_point_on_segment(sphere_center, cap_base, cap_tip);
    let delta = sphere_center - closest;
    let dist2 = delta.magnitude2();
    let r = s.radius + c.radius;

    if dist2 <= r * r && dist2 != 0.0 {
        Some(delta.normalize())
    } else {
        None
    }
}

pub fn intersect_capsule_capsule(c1: &Capsule, t1: &Matrix4<f32>, c2: &Capsule, t2: &Matrix4<f32>) -> Option<Vector3<f32>> {
    let a0 = t1.w.truncate() - Vector3::unit_y() * (c1.height / 2.0);
    let a1 = t1.w.truncate() + Vector3::unit_y() * (c1.height / 2.0);
    let b0 = t2.w.truncate() - Vector3::unit_y() * (c2.height / 2.0);
    let b1 = t2.w.truncate() + Vector3::unit_y() * (c2.height / 2.0);

    let (p1, p2) = closest_points_on_segments(a0, a1, b0, b1);
    let delta = p1 - p2;
    let dist2 = delta.magnitude2();
    let r = c1.radius + c2.radius;

    if dist2 <= r * r && dist2 != 0.0 {
        Some(delta.normalize())
    } else {
        None
    }
}

pub fn intersect_aabb_capsule(aabb: &Aabb, ta: &Matrix4<f32>, cap: &Capsule, tc: &Matrix4<f32>) -> Option<Vector3<f32>> {
    let box_center = ta.w.truncate();
    let base = tc.w.truncate() - Vector3::unit_y() * (cap.height / 2.0);
    let tip = tc.w.truncate() + Vector3::unit_y() * (cap.height / 2.0);

    let closest = closest_point_segment_aabb(base, tip, box_center, aabb.half_extents);
    let delta = (base + tip) * 0.5 - closest;
    let dist2 = delta.magnitude2();

    if dist2 <= cap.radius * cap.radius && dist2 != 0.0 {
        Some(delta.normalize())
    } else {
        None
    }
}


fn project_point_on_segment(p: Vector3<f32>, a: Vector3<f32>, b: Vector3<f32>) -> Vector3<f32> {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.magnitude2()).clamp(0.0, 1.0);
    a + ab * t
}

fn closest_points_on_segments(a0: Vector3<f32>, a1: Vector3<f32>, b0: Vector3<f32>, b1: Vector3<f32>) -> (Vector3<f32>, Vector3<f32>) {
    let pa = (a0 + a1) * 0.5;
    let pb = (b0 + b1) * 0.5;
    (pa, pb)
}

fn closest_point_segment_aabb(a: Vector3<f32>, b: Vector3<f32>, center: Vector3<f32>, extents: Vector3<f32>) -> Vector3<f32> {
    let midpoint = (a + b) * 0.5;
    let rel = midpoint - center;
    let clamped = Vector3::new(
        rel.x.clamp(-extents.x, extents.x),
        rel.y.clamp(-extents.y, extents.y),
        rel.z.clamp(-extents.z, extents.z),
    );
    center + clamped
}
