use crate::renderer::vertex_format::VertexSimple;
use std::{sync::Arc, mem::size_of, error::Error};
use vulkano::{buffer::{CpuAccessibleBuffer, BufferUsage, BufferContents}, device::Device};

// Exceeding this limit will disallow further allocations
// #TODO: Chunked allocator 
const MAX_BUFFER_SIZE: usize = 100 * 1024 * 1024;
const MAX_NUM_VERTICES: usize = MAX_BUFFER_SIZE / size_of::<VertexSimple>();

pub struct RenderBuffer {
    vertex_buffer: Vec<VertexSimple>,
    num_used: usize
}

pub struct BufferSpan<T>
where T: BufferContents + ?Sized {
    pub buffer: Arc<CpuAccessibleBuffer<T>>,
    pub num_used: usize
}

impl RenderBuffer {
    pub fn new() -> RenderBuffer {
        let vtx_buffer = vec![VertexSimple::default(); MAX_NUM_VERTICES];
        RenderBuffer {
            vertex_buffer: vtx_buffer,
            num_used: 0
        }
    }

    pub fn add_vertices(&mut self, vertices: Vec<VertexSimple>) {
        let new_used = self.num_used + vertices.len();
        if new_used > MAX_NUM_VERTICES {
            return;
        }

        self.vertex_buffer[self.num_used..new_used].copy_from_slice(&vertices.as_slice());
        self.num_used = new_used;
    }

    pub fn create_vertex_buffer(&self, device: &Arc<Device>) -> Result<BufferSpan<[VertexSimple]>, Box<dyn Error>> {
        let vtx_buffer = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::vertex_buffer(),
            false,
            self.vertex_buffer.to_vec()
        )?;
        
        return Ok(BufferSpan::<[VertexSimple]> {
            buffer: vtx_buffer,
            num_used: self.num_used
        });
    }

    pub fn has_any_data(&self) -> bool {
        return self.num_used > 0;
    }
}
