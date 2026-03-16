use crate::wad::Demo;
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
    ColorMap(Vec<[u8; 256]>),
    Demo(Demo),
    Palette(Vec<[[u8; 3]; 256]>),
    Patch(Patch),
    Ignored,
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

    pub fn get_colormap(&self) -> Vec<[u8; 256]> {
        let colormap_lump_index = self
            .lump_names
            .iter()
            .position(|x| x == "COLORMAP")
            .expect("No PLAYPAL lump found");
        match &self.lumps[colormap_lump_index] {
            Lump::ColorMap(map) => map.clone(),
            _ => panic!("COLORMAP lump is not a color map"),
        }
    }
}

const IGNORED_LUMPS: &[[u8; 8]] = &[
    *b"ENDOOM\0\0",
    *b"DMXGUS\0\0",
    *b"GENMIDI\0",
];

fn parse_lump(file: &Vec<u8>, info: &LumpInfo) -> Lump {
    if IGNORED_LUMPS.contains(&info.name) {
        return Lump::Ignored;
    }

    let file_position = info.file_position as usize;
    let data = &file[file_position..file_position + info.size as usize];

    if info.name == *b"PLAYPAL\0" {
        return read_palette(data);
    }

    if info.name == *b"COLORMAP" {
        return read_colormap(data);
    }

    if info.name.starts_with(b"DEMO") {
        return Lump::Demo(Demo::new(data))
    }

    Lump::Unknown
}

fn read_palette(data: &[u8]) -> Lump {
    let palette_slice: &[[[u8; 3]; 256]] =
        bytemuck::cast_slice(&data);
    Lump::Palette(palette_slice.to_vec())
}

fn read_colormap(data: &[u8]) -> Lump {
    let colormap_slice: &[[u8; 256]] =
        bytemuck::cast_slice(&data);
    Lump::ColorMap(colormap_slice.to_vec())
}
