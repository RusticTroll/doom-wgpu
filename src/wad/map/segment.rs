use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Segment {
    pub starting_vertex_index: u16,
    pub ending_vertex_index: u16,
    pub angle: i16,
    pub linedef_number: u16,
    pub direction: i16,
    pub offset: i16,
}
