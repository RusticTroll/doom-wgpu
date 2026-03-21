mod thing;

pub use thing::*;

#[derive(Debug)]
pub struct Map {
    pub things: Vec<MapThing>,
}

impl Map {
    pub fn new(things_data: &[u8]) -> Self {
        Self {
            things: bytemuck::cast_slice(things_data).to_vec(),
        }
    }
}
