use glfw::{Context, WindowEvent};
use std::sync::mpsc::Receiver;
use super::utils::*;

pub struct Window {
    glfw: glfw::Glfw,
    window_handle: glfw::Window,
    events: Receiver<(f64, WindowEvent)>,
    last_frame: f32,
    pub delta_time: f32,
    cursor_pos: (f64, f64),
    scroll_offset: (f64, f64),
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

        unsafe { 
            gl::Enable(gl::DEPTH_TEST);
        }

        Window {
            glfw,
            window_handle: window,
            events,
            last_frame: 0.0,
            delta_time: 0.0,
            cursor_pos: (0.0, 0.0),
            scroll_offset: (0.0, 0.0),
        }
    }

    pub fn should_close(&self) -> bool {
        self.window_handle.should_close()
    }

    pub fn update(&mut self) {
        let current_frame = self.glfw.get_time() as f32;
        self.delta_time = current_frame - self.last_frame;
        self.last_frame = current_frame;

        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                WindowEvent::FramebufferSize(width, height) => {
                    unsafe { gl::Viewport(0, 0, width, height) }
                }
                WindowEvent::CursorPos(xpos, ypos) => {
                    self.cursor_pos = (xpos, ypos);
                }
                WindowEvent::Scroll(xoffset, yoffset) => {
                    self.scroll_offset = (xoffset, yoffset);
                }
                _ => {}
            }
        }

        self.glfw.poll_events();
        self.window_handle.swap_buffers();
    }

    pub fn get_size(&self) -> (u32, u32) {
        let (width, height) = self.window_handle.get_framebuffer_size();
        (width as u32, height as u32)
    }

    pub fn clear(r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            gl::ClearColor(r, g, b, a);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn key_pressed (&self, key: char) -> bool {
        if let Some(glfw_key) = char_to_glfw_key(key) {
            if self.window_handle.get_key(glfw_key) == glfw::Action::Press {
                return true;
            }
        }
        false
    }

    pub fn get_cursor_pos(&self) -> (f64, f64) {
        self.cursor_pos
    }

    pub fn get_scroll_offset(&self) -> (f64, f64) {
        self.scroll_offset
    }
}
