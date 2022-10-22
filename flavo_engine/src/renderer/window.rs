use std::{error::Error, sync::Mutex};
use winit::{event_loop::{EventLoop, ControlFlow}, event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode}, platform::run_return::EventLoopExtRunReturn};
use crate::{game::Game};

use super::vulkan::VulkanRenderer;

pub struct GameWindow<'a> {
    event_loop: EventLoop<()>,
    game: &'a mut Game<'a>,
    renderer: &'a Mutex<&'a mut VulkanRenderer>
}

impl<'a> GameWindow<'a> {
    pub fn new(event_loop: EventLoop<()>, game: &'a mut Game<'a>, renderer: &'a Mutex<&'a mut VulkanRenderer>) -> GameWindow<'a> {
        return GameWindow {
            event_loop: event_loop,
            game: game,
            renderer: renderer
        };
    }

    pub fn run_event_loop(mut self) -> Result<(), Box<dyn Error>> {
        //optick::start_capture();
        profiling::register_thread!("Main Thread");
        let mut can_process = false;
        self.event_loop.run_return(move |event, _, control_flow| match event {
            Event::Resumed => {
                can_process = true;
            },
            Event::Suspended => {
                can_process = false;
            },
            Event::WindowEvent { event, .. } => {
                if can_process {
                    match event {
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
                    }
                }
            },
            Event::MainEventsCleared => {
                profiling::finish_frame!();
                if can_process {
                    self.game.update().expect("Game update failed");
                    self.renderer.lock().expect("Couldn't acquire lock").update().expect("Render update failed");
                    self.game.synchronize().expect("Sync window update failed");
                }
            }
            _ => (),
        });

        //optick::stop_capture("flavo_profile");
        return Ok(());
    }
}
