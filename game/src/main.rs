use rusty_gears::*;

pub fn main() {
    let mut window = Window::new(1280, 720, "RustyGears");
    let mut camera = Camera::default();

    let shader = Shader::new("shaders/vertex_shader.glsl", "shaders/fragment_shader.glsl");
    let depth_shader = Shader::new("shaders/depth_vertex_shader.glsl", "shaders/depth_fragment_shader.glsl");

    let ambient_light = AmbientLight::new(vec3(0.2, 0.2, 0.2), 2.0);
    let light_source1 = LightSource::new(vec3(-5.0, 10.0, -10.0), vec3(1.0, 1.0, 1.0), 1.0);
    //let light_source2 = LightSource::new(vec3(5.0, 10.0, 10.0), vec3(1.0, 1.0, 1.0), 0.0);

    let plane = Model::new("resources/models/plane/plane.obj", vec3(0.0, -0.56, 0.0));
    let car = Model::new("resources/models/car/Avent_sport.obj", vec3(0.0, 0.0, 0.0));
    let ball = Model::new("resources/models/ball/ball.obj", vec3(1.5, 3.0, 0.0));
    let block = Model::new("resources/models/block/block.obj", vec3(0.0, 7.0, 0.0));

    let fog = Fog::new(vec3(0.2, 0.2, 0.2), 0.05);

    let mut scene = Scene::new();
    scene.add_model(ball);
    scene.add_model(block);
    scene.add_model(car);
    scene.add_model(plane);

    scene.add_light_source(light_source1);
    //scene.add_light_source(light_source2);
    scene.set_ambient_light(ambient_light);
    scene.set_fog(fog);


    use std::time::{Duration, Instant};

    let mut last_time = Instant::now();
    let mut frame_count = 0;

    while !window.should_close() {

        frame_count += 1;

        let elapsed = last_time.elapsed();
        if elapsed >= Duration::new(1, 0) {
            let fps = frame_count;
            frame_count = 0;
            last_time = Instant::now();
            println!("FPS: {}", fps);
        }


        Window::clear(0.7, 0.8, 0.9, 1.0);

        if window.key_pressed('W') {
            camera.move_forward(window.delta_time);
        }
        if window.key_pressed('A') {
            camera.move_left(window.delta_time);
        }
        if window.key_pressed('S') {
            camera.move_backward(window.delta_time);
        }
        if window.key_pressed('D') {
            camera.move_right(window.delta_time);
        }

        let (xpos, ypos) = window.get_cursor_pos();
        camera.rotate(xpos as f32, ypos as f32, true);

        let (_xoffset, yoffset) = window.get_scroll_offset();
        camera.zoom(yoffset as f32);

        scene.render_depth_map(&depth_shader);
        scene.render(&shader, &camera,  window.get_size());

        window.update();
    }
}
