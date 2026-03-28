use crate::Fixed;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
/// Vanilla DOOM Vertex
pub struct Vertex {
    pub x_position: i16,
    pub y_position: i16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
/// ZDoom Extended Nodes Vertex
pub struct ZVertex {
    pub x_position: Fixed,
    pub y_position: Fixed,
}

impl From<Vertex> for ZVertex {
    fn from(value: Vertex) -> Self {
        Self {
            x_position: Fixed::from(value.x_position),
            y_position: Fixed::from(value.y_position),
        }
    }
}
