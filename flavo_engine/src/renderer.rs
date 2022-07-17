pub mod buffer;
pub mod vertex_format;
pub mod vulkan;
pub mod window;

use std::error::Error;
use winit::event_loop::EventLoop;

pub trait Renderer {
    fn schedule_resize(&mut self);
    fn update(&mut self) -> Result<(), Box<dyn Error>>;

    fn add_vertices(&mut self, vertices: Vec<vertex_format::VertexSimple>);
}

pub fn create_event_loop() -> EventLoop<()> {
    return EventLoop::new();
}
