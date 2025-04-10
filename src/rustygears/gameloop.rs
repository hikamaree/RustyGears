use crate::GearEvent;

use tokio::runtime::Runtime;
use winit::keyboard::PhysicalKey;
use winit::event_loop::EventLoop;
use winit::event::WindowEvent;
use winit::event::KeyEvent;
use winit::event::Event;
use winit::event::DeviceEvent;

use super::Game;

pub(crate) struct GameLoop;

impl GameLoop {
    pub(crate) fn run(game: Game, game_loop: EventLoop<()>) {
        let rt = Runtime::new().unwrap();
        rt.block_on(GameLoop::run_loop(game, game_loop));
    }

    async fn run_loop(mut game: Game, game_loop: EventLoop<()>) {
        game_loop.run(move |event, control_flow| {
            match event {
                Event::NewEvents(_) => {
                    game.time.update();
                    Game::dispatch_event(&mut game, GearEvent::Update());
                }

                Event::DeviceEvent { event, .. } => {
                    match event {
                        DeviceEvent::MouseMotion { delta } => {
                            Game::dispatch_event(&mut game, GearEvent::MouseMotion(delta.0, delta.1));
                        }
                        _ => {}
                    }
                }

                Event::WindowEvent { ref event, window_id, } if window_id == game.graphics.window.id() => {
                    match event {
                        WindowEvent::CloseRequested => control_flow.exit(),

                        WindowEvent::Resized(physical_size) => {
                            game.graphics.resize(*physical_size);
                            Game::dispatch_event(&mut game, GearEvent::WindowResize(*physical_size));
                        }

                        WindowEvent::RedrawRequested => {
                            Game::dispatch_event(&mut game, GearEvent::RenderRequested());
                        }

                        WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(key), state, .. }, .. } => {
                            Game::dispatch_event(&mut game, GearEvent::KeyboardInput(*key, *state));
                        }

                        _ => {}
                    }
                }
                _ => {}
            }
        }).expect("majmuneee");
    }
}
