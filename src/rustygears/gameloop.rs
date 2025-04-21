use winit::window::WindowAttributes;
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
use crate::Graphics;

impl ApplicationHandler for Game {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let graphics = self.graphics
            .as_mut()
            .expect("ERROR: Graphics is not initialized");

        if window_id != graphics.window.id() {
            return;
        }


        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit()
            }

            winit::event::WindowEvent::Resized(physical_size) => {
                graphics.resize(physical_size);
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

        Game::dispatch_event(self, GearEvent::WindowEvent(event.clone()));
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
        let projection = &self.graphics.as_ref().unwrap().projection;
        self.scene.active_camera_mut()
            .expect("ERROR: no camera found")
            .update_view_proj(&projection);
        Game::dispatch_event(self, GearEvent::Update());
    }

    fn resumed(&mut self, game_loop: &ActiveEventLoop) {
        let title = env!("CARGO_PKG_NAME");

        let window_attributes = WindowAttributes::default()
            .with_title(title);

        #[allow(deprecated)]
        let window = game_loop
            .create_window(window_attributes)
            .expect("failed to create window");

        window.set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .expect("failed to grab cursor");
        window.set_cursor_visible(false);

        let graphics = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Graphics::new(window.into()));

        self.graphics = Some(graphics);

        while let Some(setup_fn) = self.setupfns.pop_front() {
            setup_fn(self);
        }
    }
}
