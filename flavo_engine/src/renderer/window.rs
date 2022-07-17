use std::{error::Error, sync::Mutex};
use winit::{event_loop::{EventLoop, ControlFlow}, event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode}, platform::run_return::EventLoopExtRunReturn};
use crate::{renderer::Renderer, game::Game};

pub struct GameWindow<'a> {
    event_loop: EventLoop<()>,
    game: &'a mut Game<'a>,
    renderer: &'a Mutex<&'a mut Box<dyn Renderer>>
}

impl<'a> GameWindow<'a> {
    pub fn new(event_loop: EventLoop<()>, game: &'a mut Game<'a>, renderer: &'a Mutex<&'a mut Box<dyn Renderer>>) -> GameWindow<'a> {
        return GameWindow {
            event_loop: event_loop,
            game: game,
            renderer: renderer
        };
    }

    pub fn run_event_loop(mut self) -> Result<(), Box<dyn Error>> {
        self.event_loop.run_return(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                },
                WindowEvent::Resized(_) => {
                    self.renderer.lock().expect("Couldn't acquire lock").schedule_resize();
                },
                WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode: Some(key_code), .. }, .. } => {
                    match key_code {
                        VirtualKeyCode::Escape => {
                            *control_flow = ControlFlow::Exit;
                        },
                        _ => ()
                    }
                },
                _ => ()
            },
            Event::MainEventsCleared => {
                self.game.update().expect("Game update failed");
                self.renderer.lock().expect("Couldn't acquire lock").update().expect("Render update failed");
                self.game.synchronize().expect("Sync window update failed");
            }
            _ => (),
        });

        return Ok(());
    }
}
