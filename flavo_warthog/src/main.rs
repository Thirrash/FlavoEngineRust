use std::error::Error;
use std::sync::Mutex;
use flavo_engine::game::Game;
use flavo_engine::renderer::{create_event_loop, Renderer};
use flavo_engine::renderer::vulkan::VulkanRenderer;
use flavo_engine::renderer::window::GameWindow;

fn main() -> Result<(), Box<dyn Error>> {
    flavo_engine::logger::initialize().expect("Couldn't initialize flavo_engine::logger");

    let event_loop = create_event_loop();
    let mut renderer: Box<dyn Renderer> = Box::new(VulkanRenderer::new(&event_loop)?);
    let renderer_lock = Mutex::new(&mut renderer);
    let mut game = Game::new(&renderer_lock);
    let window = GameWindow::new(event_loop, &mut game, &renderer_lock);
    window.run_event_loop()?;

    return Ok(());
}
