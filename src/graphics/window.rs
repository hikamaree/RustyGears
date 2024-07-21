use glfw::{Action, Context, Key, WindowEvent};
use std::sync::mpsc::Receiver;
use super::camera::*;
use super::camera::Camera_Movement::*;

pub struct Window {
    glfw: glfw::Glfw,
    window_handle: glfw::Window,
    events: Receiver<(f64, WindowEvent)>,
    last_frame: f32
}

impl Window {
    pub fn new(width: u32, height: u32, title: &str) -> Window {
        let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

        let (mut window, events) = glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window!");

        window.make_current();
        window.set_framebuffer_size_polling(true);
        window.set_key_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_scroll_polling(true);
        window.set_cursor_mode(glfw::CursorMode::Disabled);
        gl::load_with(|s| window.get_proc_address(s) as *const _);

        let last_frame: f32 = 0.0;

        unsafe { 
            gl::Enable(gl::DEPTH_TEST);
        }

        Window {
            glfw,
            window_handle: window,
            events,
            last_frame
        }
    }

    pub fn should_close(&self) -> bool {
        self.window_handle.should_close()
    }

    pub fn update(&mut self) {
        self.glfw.poll_events();
        self.window_handle.swap_buffers();
    }

    pub fn clear(r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            gl::ClearColor(r, g, b, a);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn process_input(&mut self, camera: &mut Camera) {
        let current_frame = self.glfw.get_time() as f32;
        let delta_time = current_frame - self.last_frame;
        self.last_frame = current_frame;


        if self.window_handle.get_key(Key::Escape) == Action::Press {
            self.window_handle.set_should_close(true)
        }

        if self.window_handle.get_key(Key::W) == Action::Press {
            camera.ProcessKeyboard(FORWARD, delta_time);
        }
        if self.window_handle.get_key(Key::S) == Action::Press {
            camera.ProcessKeyboard(BACKWARD, delta_time);
        }
        if self.window_handle.get_key(Key::A) == Action::Press {
            camera.ProcessKeyboard(LEFT, delta_time);
        }
        if self.window_handle.get_key(Key::D) == Action::Press {
            camera.ProcessKeyboard(RIGHT, delta_time);
        }
    }

    pub fn process_events(&self, last_x: &mut f32, last_y: &mut f32, camera: &mut Camera) {
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    unsafe { gl::Viewport(0, 0, width, height) }
                }
                glfw::WindowEvent::CursorPos(xpos, ypos) => {
                    let (xpos, ypos) = (xpos as f32, ypos as f32);

                    let xoffset = xpos - *last_x;
                    let yoffset = *last_y - ypos;

                    *last_x = xpos;
                    *last_y = ypos;

                    camera.ProcessMouseMovement(xoffset, yoffset, true);
                }
                glfw::WindowEvent::Scroll(_xoffset, yoffset) => {
                    camera.ProcessMouseScroll(yoffset as f32);
                }
                _ => {}
            }
        }
    }
}
