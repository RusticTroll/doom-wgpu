use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SideDef {
    pub x_offset: i16,
    pub y_offset: i16,
    pub upper_texture_name: [u8; 8],
    pub lower_texture_name: [u8; 8],
    pub middle_texture_name: [u8; 8],
    pub facing_sector: u16,
}
