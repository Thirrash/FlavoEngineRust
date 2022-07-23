use std::error::Error;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use super::render_item::RenderItem;

pub struct RenderQueue {
    render_items: Vec<RenderItem>
}

impl RenderQueue {
    pub fn new() -> RenderQueue {
        RenderQueue {
            render_items: vec![]
        }
    }

    pub fn add_render_item(&mut self, render_item: RenderItem) {
        self.render_items.push(render_item);
    }

    #[profiling::function]
    pub fn draw_all(&self, builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<(), Box<dyn Error>> {
        for render_item in &self.render_items {
            builder
                .bind_vertex_buffers(0, render_item.vertex_buffer.clone())
                .bind_index_buffer(render_item.index_buffer.clone())
                .draw_indexed(render_item.num_indices, 1, 0, 0, 0)?;
        }

        return Ok(());
    }

    pub fn has_any_data(&self) -> bool {
        return self.render_items.len() > 0;
    }
}
