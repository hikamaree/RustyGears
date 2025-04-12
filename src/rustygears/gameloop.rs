use winit::event::DeviceEvent;
use winit::event::DeviceId;
use winit::event::WindowEvent;
use winit::window::WindowId;
use winit::event_loop::ActiveEventLoop;
use winit::application::ApplicationHandler;
use winit::keyboard::PhysicalKey;
use winit::event::KeyEvent;

use crate::GearEvent;
use crate::Game;

impl ApplicationHandler for Game {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if window_id != self.graphics.window.id() {
            return;
        }

        match event {
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),

            winit::event::WindowEvent::Resized(physical_size) => {
                self.graphics.resize(physical_size);
                Game::dispatch_event(self, GearEvent::WindowResize(physical_size));
            }

            winit::event::WindowEvent::RedrawRequested => {
                Game::dispatch_event(self, GearEvent::RenderRequested());
            }

            winit::event::WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(key), state, .. }, .. } => {
                Game::dispatch_event(self, GearEvent::KeyboardInput(key, state));
            }

            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                Game::dispatch_event(self, GearEvent::MouseMotion(delta.0, delta.1));
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.time.update();
        Game::dispatch_event(self, GearEvent::Update());
    }

    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        self.graphics.window.set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .expect("failed to grab cursor");
        self.graphics.window.set_cursor_visible(false);
    }
}
