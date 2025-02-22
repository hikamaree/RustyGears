use crate::{
    window::State, DrawLight, DrawModel, GearEvent
};

use tokio::runtime::Runtime;
use winit::{
    event::{
        DeviceEvent, Event, KeyEvent, WindowEvent
    },
    event_loop::EventLoop, keyboard::PhysicalKey
};

use std::{iter, sync::{Arc, Mutex}};

use super::Game;


fn render(state: &State) -> Result<(), wgpu::SurfaceError> {
    let render_data = &state.render_data;
    let output = state.surface.get_current_texture()?;
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &render_data.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_vertex_buffer(1, render_data.instance_buffer.slice(..));
        render_pass.set_pipeline(&render_data.light_render_pipeline);
        render_pass.draw_light_model(
            &render_data.obj_light,
            &render_data.camera_bind_group,
            &render_data.light_bind_group,
        );

        render_pass.set_pipeline(&render_data.render_pipeline);
        render_pass.draw_model_instanced(
            &render_data.obj_model,
            0..render_data.instances.len() as u32,
            &render_data.camera_bind_group,
            &render_data.light_bind_group,
        );
    }
    state.queue.submit(iter::once(encoder.finish()));
    output.present();

    Ok(())
}

pub(crate) struct GameLoop;

impl GameLoop {
    pub fn new() -> Self {
        Self {
        }
    }

    pub fn run(&mut self, game: &mut Game) -> &mut Self {
        let rt = Runtime::new().unwrap();
        rt.block_on(self.run_loop(game));
        self
    }

    async fn run_loop(&mut self, game: &mut Game) {
        let event_loop = EventLoop::new().unwrap();
        let title = env!("CARGO_PKG_NAME");
        let window = winit::window::WindowBuilder::new()
            .with_title(title)
            .build(&event_loop)
            .unwrap();
        window.set_cursor_grab(winit::window::CursorGrabMode::Locked).expect("nema cursor");
        window.set_cursor_visible(false);
        
        let state = State::new(&window, game.cameras.active_camera().expect("no camera found")).await;
        let state = Arc::new(Mutex::new(state));

        let game = Arc::new(Mutex::new(std::mem::take(game)));

        event_loop.run(move |event, control_flow| {
            match event {
                Event::NewEvents(_) => {
                    game.lock().unwrap().time.update();
                    Game::dispatch_event(game.clone(), GearEvent::Update());
                    state.lock().unwrap().update(game.lock().unwrap().cameras.active_camera().expect("no camera found"));
                    state.lock().unwrap().window().request_redraw();
                }

                Event::DeviceEvent { event, .. } => {
                    match event {
                        DeviceEvent::MouseMotion { delta } => {
                            Game::dispatch_event(game.clone(), GearEvent::MouseMotion(delta.0, delta.1));
                        }
                        _ => {}
                    }
                }

                Event::WindowEvent { ref event, window_id, } if window_id == state.lock().unwrap().window().id() => {
                    match event {
                        WindowEvent::CloseRequested => control_flow.exit(),

                        WindowEvent::Resized(physical_size) => {
                            state.lock().unwrap().resize(*physical_size);
                        }

                        WindowEvent::RedrawRequested => {
                            Game::dispatch_event(game.clone(), GearEvent::RenderRequested());
                            let _ = render(&state.lock().unwrap());
                        }

                        WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(key), state, .. }, .. } => {
                            Game::dispatch_event(game.clone(), GearEvent::KeyboardInput(*key, *state));
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }).expect("majmuneee");
    }
}
