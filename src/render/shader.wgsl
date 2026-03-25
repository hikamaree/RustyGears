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

// Vertex shader
struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    light_count: f32,
    _padding: vec3<f32>,
};
@group(1) @binding(0)
var<uniform> camera: Camera;

struct GpuLight {
    position: vec3<f32>,
    radius: f32,
    color: vec3<f32>,
    intensity: f32,
    direction: vec3<f32>,
    light_type: u32,
    spot_angles: vec2<f32>,
    _padding: vec2<u32>,
};
@group(2) @binding(0)
var<uniform> lights: array<GpuLight, 64>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) world_normal: vec3<f32>,
    @location(3) world_view_pos: vec3<f32>,
}

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
	let model_matrix = mat4x4<f32>(
		instance.model_matrix_0,
		instance.model_matrix_1,
		instance.model_matrix_2,
		instance.model_matrix_3,
	);
	let normal_matrix = mat3x3<f32>(
		instance.normal_matrix_0,
		instance.normal_matrix_1,
		instance.normal_matrix_2,
	);

	let world_position = (model_matrix * vec4<f32>(model.position, 1.0)).xyz;
	let world_normal = normalize(normal_matrix * model.normal);
	let world_view_pos = camera.view_pos.xyz;

	var out: VertexOutput;
	out.clip_position = camera.view_proj * vec4<f32>(world_position, 1.0);
	out.tex_coords = model.tex_coords;
	out.world_position = world_position;
	out.world_normal = world_normal;
	out.world_view_pos = world_view_pos;
	return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var t_normal: texture_2d<f32>;
@group(0) @binding(3)
var s_normal: sampler;
@group(0) @binding(4)
var t_alpha: texture_2d<f32>;
@group(0) @binding(5)
var s_alpha: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let object_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);

	let alpha_sample = textureSample(t_alpha, s_alpha, in.tex_coords);
	if (alpha_sample.r < 0.1) {
		discard;
	}

	let world_normal = normalize(in.world_normal);

	let view_dir = normalize(in.world_view_pos - in.world_position);

	var color_accum = vec3<f32>(0.05, 0.05, 0.05);

	let num_lights = u32(camera.light_count);
	for (var i: u32 = 0u; i < num_lights; i = i + 1u) {
		let light = lights[i];

		if (light.intensity < 0.01) {
			continue;
		}

		var light_dir: vec3<f32>;
		var attenuation: f32 = 1.0;

		if (light.light_type == 0u) {
			let to_light = light.position - in.world_position;
			let dist = length(to_light);
			light_dir = normalize(to_light);
			let dist_norm = dist / light.radius;
			attenuation = max(1.0 - dist_norm * dist_norm, 0.0);
		} else if (light.light_type == 1u) {
			light_dir = normalize(-light.direction);
			attenuation = 1.0;
		} else if (light.light_type == 2u) {
			let to_light = light.position - in.world_position;
			let dist = length(to_light);
			light_dir = normalize(to_light);

			let dist_norm = dist / light.radius;
			let dist_attenuation = max(1.0 - dist_norm * dist_norm, 0.0);

			let spot_dir = normalize(-light.direction);
			let theta = dot(light_dir, spot_dir);
			let inner_angle = light.spot_angles.x;
			let outer_angle = light.spot_angles.y;

			let epsilon = inner_angle - outer_angle;
			let spot_intensity = clamp((theta - outer_angle) / epsilon, 0.0, 1.0);

			attenuation = dist_attenuation * spot_intensity;
		} else {
			continue;
		}

		let diff = max(dot(world_normal, light_dir), 0.0);
		let diffuse = light.color * diff * light.intensity * attenuation;

		color_accum += diffuse;
	}

	let final_color = color_accum * object_color.rgb;
	return vec4<f32>(final_color, object_color.a);
}
