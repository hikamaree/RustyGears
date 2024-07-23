use rusty_gears::*;

const SCR_WIDTH: u32 = 1280;
const SCR_HEIGHT: u32 = 720;

pub fn main() {
    let mut camera = Camera {
        Position: Point3::new(0.0, 0.0, 3.0),
        ..Camera::default()
    };

    let mut last_x: f32 = SCR_WIDTH as f32 / 2.0;
    let mut last_y: f32 = SCR_HEIGHT as f32 / 2.0;

    let mut window = Window::new(SCR_WIDTH, SCR_HEIGHT, "RustyGears");

    let shader = Shader::new("shaders/vertex_shader.vs", "shaders/fragment_shader.fs");
    let depth_shader = Shader::new("shaders/depth_shader.vs", "shaders/depth_shader.fs");

    let ambient_light = AmbientLight::new(vec3(0.2, 0.2, 0.2), 1.5);
    let light_source1 = LightSource::new(vec3(2.0, 2.0, 2.0), vec3(1.0, 1.0, 1.0), 1.0);
    let light_source2 = LightSource::new(vec3(-2.0, -2.0, -2.0), vec3(1.0, 1.0, 1.0), 1.0);

    let big_block = Model::new("resources/models/big_block/big_block.obj", vec3(0.0, 0.0, 0.0));
    let ball = Model::new("resources/models/ball/ball.obj", vec3(0.0, 0.0, 0.0));

    let fog = Fog::new(vec3(0.2, 0.2, 0.2), 0.1);

    let mut scene = Scene::new();
    scene.add_model(big_block);
    scene.add_model(ball);
    scene.add_light_source(light_source1);
    scene.add_light_source(light_source2);
    scene.set_ambient_light(ambient_light);
    scene.set_fog(fog);

    while !window.should_close() {
        Window::clear(0.1, 0.1, 0.1, 1.0);
        window.process_events(&mut last_x, &mut last_y, &mut camera);
        window.process_input(&mut camera);

        scene.render_depth_map(&depth_shader);

        scene.render(&shader, &camera, window.get_size());

        window.update();
    }
}
