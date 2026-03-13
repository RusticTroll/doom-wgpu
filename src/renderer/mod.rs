mod pipeline;
mod state;
mod texture;

pub use crate::renderer::state::RenderState;

pub fn load_binary(file_name: &str) -> Vec<u8> {
    return std::fs::read(file_name).unwrap();
}
