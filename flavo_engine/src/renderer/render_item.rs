use crate::renderer::vertex_format::VertexSimple;
use std::{sync::Arc};
use vulkano::{buffer::{BufferUsage, ImmutableBuffer}, device::{Queue}};

pub struct RenderItem {
    pub vertex_buffer: Arc<ImmutableBuffer<[VertexSimple]>>,
    pub index_buffer: Arc<ImmutableBuffer<[u32]>>,
    pub num_indices: u32
}

impl RenderItem {
    #[profiling::function]
    pub fn new(queue: &Arc<Queue>, vertices: Vec<VertexSimple>, indices: Vec<u32>) -> RenderItem {
        let num_indices = indices.len();

        let (vtx_buffer, _) = ImmutableBuffer::from_iter(
            vertices.into_iter(),
            BufferUsage::vertex_buffer(),
            queue.clone()
        ).unwrap();

        let (idx_buffer, _) = ImmutableBuffer::from_iter(
            indices.into_iter(),
            BufferUsage::index_buffer(),
            queue.clone()
        ).unwrap();

        return RenderItem {
            vertex_buffer: vtx_buffer,
            index_buffer: idx_buffer,
            num_indices: num_indices as u32
        };
    }
}
