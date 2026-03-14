use super::patch::Patch;

pub enum Lump {
    Patch(Patch)
}

pub struct Wad {
    pub lumps: Vec<Lump>,
}

impl Wad {
    pub fn load(file_name: &str) {
        
    }
}