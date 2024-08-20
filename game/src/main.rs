use rusty_gears::*;

pub fn main() {
    let mut window = Window::new(1280, 720, "RustyGears");
    let camera = Camera::new();

    let shader = Shader::new("shaders/vertex_shader.glsl", "shaders/fragment_shader.glsl");
    let depth_shader = Shader::new("shaders/depth_vertex_shader.glsl", "shaders/depth_fragment_shader.glsl");

    let ambient_light = AmbientLight::new(vec3(0.2, 0.2, 0.2), 2.0);
    let light_source1 = LightSource::new(vec3(-5.0, 10.0, -10.0), vec3(1.0, 1.0, 1.0), 1.0);

    let fog = Fog::new(vec3(0.2, 0.2, 0.2), 0.0);

    let big_block = Model::open("resources/models/plane/plane.obj");
    let car = Model::open("resources/models/car/Avent_sport.obj");
    let ball = Model::open("resources/models/ball/ball.obj");
    let bullet = Model::open("resources/models/bullet/bullet.obj");
    let block = Model::open("resources/models/block/block.obj");

    let bbc = Object::new()
        .add_model(big_block.clone())
        .set_body(RigidBody::from_model_with_bounding_boxes(&big_block))
        .set_mass(1000000.0);

    let mut lambo = Character::new()
        .add_model(car.clone())
        .set_body(RigidBody::from_model_with_bounding_boxes(&car))
        .set_gravity(false)
        .set_mass(10000.0)
        .set_position(vec3(5.0, 2.0, 5.0));

    let mut sphere = Character::new()
        .add_model(ball.clone())
        .set_body(RigidBody::from_model_with_spheres(&ball))
        .set_gravity(false)
        .set_mass(10.0)
        .set_position(vec3(0.0, 15.0, 0.0));

    let mut sbc = Character::new()
        .add_model(block.clone())
        .set_body(RigidBody::from_model_with_bounding_boxes(&block))
        .set_gravity(false)
        .set_mass(10.0)
        .set_position(vec3(1.5, 50.0, 1.1))
        .set_rotation(Quaternion::from_angle_z(Deg(-30.0)));


    let scene = Scene::create();
    window.set_scene(scene.clone());
    window.background_color(vec3(0.6, 0.7, 0.8));

    scene.borrow_mut()
        .set_depth_shader(depth_shader)
        .set_shader(shader)
        .add(&camera)
        .add(&light_source1)
        .add(&ambient_light)
        .add(&fog)
        .add(&bbc)
        .add(&sbc)
        .add(&sphere)
        .add(&lambo);

    let mut pucaj = false;

    while !window.should_close() {
        window.update();

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
        if window.key_pressed('F') && !pucaj {
            pucaj = true;
            let b = Character::new()
                .add_model(bullet.clone())
                .set_body(RigidBody::from_model_with_spheres(&bullet))
                .set_position(camera.position())
                .set_velocity(camera.front() * 50.0);
            scene.borrow_mut().add(&b);
        }
        if window.key_released('F') && pucaj {
            lambo.set_gravity(true);
            sphere.set_gravity(true);
            sbc.set_gravity(true);
            pucaj = false;
        }

        let (xpos, ypos) = window.get_cursor_pos();
        camera.rotate(xpos, ypos, true);

        let (_xoffset, yoffset) = window.get_scroll_offset();
        camera.zoom(yoffset);

    }
}
