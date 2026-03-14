use super::patch::Patch;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct LumpInfo {
    pub file_position: i32,
    pub size: i32,
    pub name: [u8; 8],
}

#[derive(Debug)]
pub enum Lump {
    Palette(Vec<[[u8; 3]; 256]>),
    Patch(Patch),
    Unknown,
}

#[derive(Debug)]
pub struct Wad {
    lumps: Vec<Lump>,
    lump_names: Vec<String>,
}

impl Wad {
    pub fn load(file_name: &str) -> Self {
        let file =
            std::fs::read(file_name).expect(&format!("Failed to read file from '{}'", file_name));

        if file[1..4] != *b"WAD" {
            panic!("File '{}' is not a WAD file", file_name);
        }

        let num_lumps: i32 = *bytemuck::from_bytes(&file[4..8]);
        let info_table_offset: i32 = *bytemuck::from_bytes(&file[8..12]);
        let all_lump_info: &[LumpInfo] = bytemuck::cast_slice(&file[info_table_offset as usize..]);

        let mut lump_names = Vec::with_capacity(num_lumps as usize);
        let mut lumps = Vec::with_capacity(num_lumps as usize);

        for info in all_lump_info {
            lump_names.push(
                String::from_utf8(info.name.to_vec())
                    .expect(&format!("Failed to get name for lump {:?}", info)),
            );
            lumps.push(parse_lump(&file, info));
        }

        Self { lumps, lump_names }
    }

    pub fn get_palette(&self) -> Vec<[[u8; 3]; 256]> {
        let palette_lump_index = self
            .lump_names
            .iter()
            .position(|x| x == "PLAYPAL\0")
            .expect("No PLAYPAL lump found");
        match &self.lumps[palette_lump_index] {
            Lump::Palette(palette) => palette.clone(),
            _ => panic!("PLAYPAL lump is not a palette"),
        }
    }
}

fn parse_lump(file: &Vec<u8>, info: &LumpInfo) -> Lump {
    if info.name == *b"PLAYPAL\0" {
        return read_palette(file, info.file_position as usize);
    }

    Lump::Unknown
}

const PALETTE_SIZE: usize = std::mem::size_of::<[[[u8; 3]; 256]; 14]>();

fn read_palette(file: &Vec<u8>, offset: usize) -> Lump {
    let palette_slice: &[[[u8; 3]; 256]] =
        bytemuck::cast_slice(&file[offset..offset + PALETTE_SIZE]);
    Lump::Palette(palette_slice.to_vec())
}
