use rusty_gears::*;

pub fn main() {
    let mut window = Window::new(1280, 720, "RustyGears");
    let mut camera = Camera::default();

    let shader = Shader::new("shaders/vertex_shader.vs", "shaders/fragment_shader.fs");
    let depth_shader = Shader::new("shaders/depth_shader.vs", "shaders/depth_shader.fs");

    let ambient_light = AmbientLight::new(vec3(0.2, 0.2, 0.2), 2.5);
    let light_source1 = LightSource::new(vec3(10.0, 10.0, 0.0), vec3(1.0, 1.0, 1.0), 10.0);
    let light_source2 = LightSource::new(vec3(-2.0, -2.0, -2.0), vec3(1.0, 1.0, 1.0), 1.0);

    let big_block = Model::new("resources/models/big_block/big_block.obj", vec3(0.0, 0.0, 0.0));
    let ball = Model::new("resources/models/block/block.obj", vec3(0.0, 0.0, 0.0));

    let fog = Fog::new(vec3(0.2, 0.2, 0.2), 0.2);

    let mut scene = Scene::new();
    scene.add_model(big_block);
    scene.add_model(ball);
    scene.add_light_source(light_source1);
    scene.add_light_source(light_source2);
    scene.set_ambient_light(ambient_light);
    scene.set_fog(fog);

    while !window.should_close() {
        Window::clear(1.0, 1.0, 1.0, 1.0);

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
