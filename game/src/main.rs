#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

extern crate gl;
use std::ffi::CStr;

use rusty_gears::*;


use cgmath::{Matrix4, vec3,  Deg, perspective, Point3};


const SCR_WIDTH: u32 = 1280;
const SCR_HEIGHT: u32 = 720;

pub fn main() {
    let mut camera = Camera {
        Position: Point3::new(0.0, 0.0, 3.0),
        ..Camera::default()
    };

    let mut lastX: f32 = SCR_WIDTH as f32 / 2.0;
    let mut lastY: f32 = SCR_HEIGHT as f32 / 2.0;

    let mut window = Window::new(SCR_WIDTH, SCR_HEIGHT, "RustyGears");

    let shader = Shader::new("shaders/vertex_shader.vs", "shaders/fragment_shader.fs");

    let cube = Model::new("resources/models/block/block.obj");

    while !window.should_close() {
        window.process_events(&mut lastX, &mut lastY, &mut camera);

        window.process_input(&mut camera);

        Window::clear(0.2, 0.2, 0.2, 1.0);

        unsafe {
            shader.useProgram();

            let projection: Matrix4<f32> = perspective(Deg(camera.Zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32 , 0.1, 100.0);
            shader.setMat4(c_str!("projection"), &projection);

            let view = camera.GetViewMatrix();
            shader.setMat4(c_str!("view"), &view);

            cube.Draw(&shader, vec3(0.0, 0.0, 0.0));
        }

        window.update();
    }
}
