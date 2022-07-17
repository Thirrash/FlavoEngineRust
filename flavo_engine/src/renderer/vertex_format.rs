use bytemuck::{Zeroable, Pod};

#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable, Pod)]
pub struct VertexSimple {
    pub position: [f32; 3]
}
