use rusty_gears::*;
use std::process::Command;
use std::time::{Duration, Instant};

pub fn main() {
    let mut window = Window::new(1280, 720, "RustyGears");
    let camera = Camera::create();

    let shader = Shader::new("shaders/vertex_shader.glsl", "shaders/fragment_shader.glsl");
    let depth_shader = Shader::new("shaders/depth_vertex_shader.glsl", "shaders/depth_fragment_shader.glsl");

    let ambient_light = AmbientLight::new(vec3(0.2, 0.2, 0.2), 2.0);
    let light_source1 = LightSource::new(vec3(-5.0, 10.0, -10.0), vec3(1.0, 1.0, 1.0), 1.0);
    let light_source2 = LightSource::new(vec3(5.0, 10.0, 10.0), vec3(1.0, 1.0, 1.0), 1.0);

    let big_block = Model::create("resources/models/big_block/big_block.obj", vec3(0.0, -10.45, 0.0));
    let car = Model::create("resources/models/car/Avent_sport.obj", vec3(0.0, 0.0, 0.0));
    let ball = Model::create("resources/models/ball/ball.obj", vec3(1.5, 3.0, 0.0));
    let block = Model::create("resources/models/block/block.obj", vec3(0.0, 7.0, 0.0));

    let fog = Fog::new(vec3(0.2, 0.2, 0.2), 0.005);

    let scene = Scene::create();
    window.set_scene(scene.clone());

    {
        let mut s = scene.borrow_mut();
        s.set_camera(camera.clone());

        s.add_model(ball.clone());
        s.add_model(block.clone());
        s.add_model(car.clone());
        s.add_model(big_block.clone());

        s.add_light_source(light_source1);
        s.add_light_source(light_source2);
        s.set_ambient_light(ambient_light);
        s.set_fog(fog);

        s.set_depth_shader(depth_shader);
        s.set_shader(shader);
    }

    let mut last_time = Instant::now();
    let mut frame_count = 0;

    while !window.should_close() {

        frame_count += 1;
        let elapsed = last_time.elapsed();
        if elapsed >= Duration::new(1, 0) {
            let fps = frame_count;
            frame_count = 0;
            last_time = Instant::now();
            Command::new("clear").status().expect("Failed to clear console");
            println!("FPS: {}", fps);
        }

        {
            let mut cam = camera.borrow_mut();
            if window.key_pressed('W') {
                cam.move_forward(window.delta_time);
            }
            if window.key_pressed('A') {
                cam.move_left(window.delta_time);
            }
            if window.key_pressed('S') {
                cam.move_backward(window.delta_time);
            }
            if window.key_pressed('D') {
                cam.move_right(window.delta_time);
            }

            let (xpos, ypos) = window.get_cursor_pos();
            cam.rotate(xpos as f32, ypos as f32, true);

            let (_xoffset, yoffset) = window.get_scroll_offset();
            cam.zoom(yoffset as f32);
        }
        Window::clear(0.7, 0.8, 0.9, 1.0);
        window.update();
    }
}
