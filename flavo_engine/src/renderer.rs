pub mod render_item;
pub mod render_queue;
pub mod vertex_format;
pub mod vulkan;
pub mod window;

use winit::event_loop::EventLoop;

pub fn create_event_loop() -> EventLoop<()> {
    return EventLoop::new();
}
