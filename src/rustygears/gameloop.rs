use crate::GearEvent;

use tokio::runtime::Runtime;
use winit::keyboard::PhysicalKey;
use winit::event_loop::EventLoop;
use winit::event::WindowEvent;
use winit::event::KeyEvent;
use winit::event::Event;
use winit::event::DeviceEvent;

use std::sync::Mutex;
use std::sync::Arc;

use super::Game;

pub(crate) struct GameLoop;

impl GameLoop {
    pub(crate) fn run(game: Game, game_loop: EventLoop<()>) {
        let rt = Runtime::new().unwrap();
        rt.block_on(GameLoop::run_loop(game, game_loop));
    }

    async fn run_loop(game: Game, game_loop: EventLoop<()>) {
        // let graphics = game.graphics;
        let game = Arc::new(Mutex::new(game));

        game_loop.run(move |event, control_flow| {
            match event {
                Event::NewEvents(_) => {
                    game.lock().unwrap().time.update();
                    Game::dispatch_event(game.clone(), GearEvent::Update());
                }

                Event::DeviceEvent { event, .. } => {
                    match event {
                        DeviceEvent::MouseMotion { delta } => {
                            Game::dispatch_event(game.clone(), GearEvent::MouseMotion(delta.0, delta.1));
                        }
                        _ => {}
                    }
                }

                Event::WindowEvent { ref event, window_id, } if window_id == game.lock().unwrap().graphics.window.id() => {
                    match event {
                        WindowEvent::CloseRequested => control_flow.exit(),

                        WindowEvent::Resized(physical_size) => {
                            game.lock().unwrap().graphics.resize(*physical_size);
                        }

                        WindowEvent::RedrawRequested => {
                            game.lock().unwrap().graphics.window.request_redraw();
                            Game::dispatch_event(game.clone(), GearEvent::RenderRequested());
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
