use rusty_gears::*;
use std::process::Command;
use std::time::{Duration, Instant};

struct Fps {
    last_time: Instant,
    frame_count: u32
}

impl Fps {
    pub fn init() -> Self {
        Self {
            last_time: Instant::now(),
            frame_count: 0
        }
    }
    pub fn update(&mut self) {
        self.frame_count += 1;
        let elapsed = self.last_time.elapsed();
        if elapsed >= Duration::new(1, 0) {
            let fps = self.frame_count;
            self.frame_count = 0;
            self.last_time = Instant::now();
            Command::new("clear").status().expect("Failed to clear console");
            println!("FPS: {}", fps);
        }
    }
}

pub fn main() {
    let mut window = Window::new(1280, 720, "RustyGears");
    let camera = Camera::new();

    let shader = Shader::new("shaders/vertex_shader.glsl", "shaders/fragment_shader.glsl");
    let depth_shader = Shader::new("shaders/depth_vertex_shader.glsl", "shaders/depth_fragment_shader.glsl");

    let ambient_light = AmbientLight::new(vec3(0.2, 0.2, 0.2), 2.0);
    let light_source1 = LightSource::new(vec3(-5.0, 10.0, -10.0), vec3(1.0, 1.0, 1.0), 1.0);

    let fog = Fog::new(vec3(0.2, 0.2, 0.2), 0.0);

    let big_block = Model::create("resources/models/plane/plane.obj", vec3(10.0, 0.0, 10.0));
    //let car = Model::create("resources/models/car/Avent_sport.obj", vec3(0.0, 0.2, 0.0));
    let ball = Model::create("resources/models/ball/ball.obj", vec3(0.0, 15.0, 0.0));
    let bullet = Model::create("resources/models/bullet/bullet.obj", vec3(1.5, 50.0, 1.1));
    let block = Model::create("resources/models/block/block.obj", vec3(1.5, 50.0, 1.1));
    //block.borrow_mut().set_rotation(Quaternion::from_angle_z(Deg(-30.0)));

    let bbc = Object::new()
        .add_model(big_block.clone())
        .set_body(RigidBody::from_model_with_bounding_boxes(&big_block.borrow(), 1000000.0));

    let sbc = Character::new()
        .add_model(block.clone())
        .set_body(RigidBody::from_model_with_bounding_boxes(&block.borrow(), 10.0));

    let sphere = Character::new()
        .add_model(ball.clone())
        .set_body(RigidBody::from_model_with_spheres(&ball.borrow(), 10.0));

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
        .add(&sphere);

    let mut pucaj = false;

    let mut fps = Fps::init();
    while !window.should_close() {
        fps.update();
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
            let mut b = Character::new()
                .add_model(bullet.clone())
                .set_body(RigidBody::from_model_with_spheres(&bullet.borrow(), 1.0))
                .set_position(camera.position())
                .set_velocity(camera.front() * 50.0);
            scene.borrow_mut().add(&b);
        }
        if window.key_released('F') && pucaj {
            pucaj = false;
        }

        let (xpos, ypos) = window.get_cursor_pos();
        camera.rotate(xpos, ypos, true);

        let (_xoffset, yoffset) = window.get_scroll_offset();
        camera.zoom(yoffset);
        window.update();
        //println!("Block position: {:?}", block.borrow_mut().position);
        //println!("Ball position: {:?}", ball.borrow_mut().position);
    }
}
