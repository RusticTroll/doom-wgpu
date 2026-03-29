mod demo;
mod map;
mod patch;
mod sound;

pub use demo::*;
pub use map::*;
pub use patch::*;
pub use sound::*;

use bytemuck::{Pod, Zeroable};
use regex::Regex;
use std::{
    collections::{HashMap, VecDeque},
    sync::LazyLock,
};

static MAP_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:MAP\d\d|E\dM\d)").unwrap());

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct LumpInfo {
    pub file_position: i32,
    pub size: i32,
    pub name: [u8; 8],
}

#[derive(Debug)]
pub enum Lump {
    ColorMap(Vec<[u8; 256]>),
    Demo(Demo),
    Map(Map),
    Music(Vec<u8>),
    Palette(Vec<[[u8; 3]; 256]>),
    Patch(Patch),
    Sound(Sound),
    SoundPCSpeaker,
    Ignored,
    Unknown,
}

#[derive(Debug)]
pub struct Wad {
    lumps: HashMap<String, Lump>,
}

impl Wad {
    pub fn load(file_name: &str) -> Self {
        let mut info = ajbsp::buildinfo_t::new();
        println!("{:#?}", info);
        ajbsp::set_info(&mut info);
        ajbsp::open_wad(file_name);
        ajbsp::create_xwa("temp.xwa");
        let level_count = ajbsp::levels_in_wad();
        println!("Level Count: {}", level_count);
        for index in 0..level_count {
            ajbsp::build_level(index).unwrap();
        }
        ajbsp::finish_xwa();
        ajbsp::close_wad();

        let file =
            std::fs::read(file_name).expect(&format!("Failed to read file from '{}'", file_name));

        if file[1..4] != *b"WAD" {
            panic!("File '{}' is not a WAD file", file_name);
        }

        let num_lumps: i32 = *bytemuck::from_bytes(&file[4..8]);
        let info_table_offset: i32 = *bytemuck::from_bytes(&file[8..12]);
        let mut all_lump_info = VecDeque::<LumpInfo>::with_capacity(num_lumps as usize);
        all_lump_info.extend(bytemuck::cast_slice(&file[info_table_offset as usize..]));

        let mut lumps = HashMap::with_capacity(num_lumps as usize);

        while let Some(info) = all_lump_info.pop_front() {
            let lump_name = String::from_utf8(info.name.to_vec())
                .expect(&format!("Failed to get name for lump {:?}", info));
            if lumps.contains_key(&lump_name) {
                continue;
            }
            if MAP_NAME_REGEX.is_match(&lump_name) {
                let map = Map::new(&file, &mut all_lump_info);
                lumps.insert(lump_name, Lump::Map(map));
            } else {
                lumps.insert(lump_name, parse_lump(&file, &info));
            }
        }

        Self { lumps }
    }

    fn get_lump(&self, name: &str) -> &Lump {
        self.lumps
            .get(&format!("{name:\0<8}"))
            .expect(&format!("No lump named '{name}' found"))
    }

    pub fn get_palette(&self) -> Vec<[[u8; 3]; 256]> {
        let palette_lump = self.get_lump("PLAYPAL");
        match palette_lump {
            Lump::Palette(palette) => palette.clone(),
            _ => panic!("PLAYPAL lump is not a palette"),
        }
    }

    pub fn get_patch(&self, name: &str) -> &Patch {
        let patch_lump = self.get_lump(name);
        match patch_lump {
            Lump::Patch(patch) => patch,
            other => panic!(
                "Lump {name} was expected to be a patch but is a {}",
                std::any::type_name_of_val(other)
            ),
        }
    }

    pub fn get_colormap(&self) -> Vec<[u8; 256]> {
        let colormap_lump = self.get_lump("COLORMAP");
        match colormap_lump {
            Lump::ColorMap(map) => map.clone(),
            _ => panic!("COLORMAP lump is not a color map"),
        }
    }

    pub fn get_sound(&self, name: &str) -> Sound {
        let sound_lump = self.get_lump(name);
        match sound_lump {
            Lump::Sound(sound) => sound.clone(),
            other => panic!(
                "Lump {name} was expected to be a sound but is a {}",
                std::any::type_name_of_val(other)
            ),
        }
    }

    pub fn get_music(&self, name: &str) -> &Vec<u8> {
        let music_lump = self.get_lump(name);
        match music_lump {
            Lump::Music(music) => &music,
            other => panic!(
                "Lump {name} was expected to be music but is a {}",
                std::any::type_name_of_val(other)
            ),
        }
    }
}

const IGNORED_LUMPS: &[[u8; 8]] = &[*b"ENDOOM\0\0", *b"DMXGUS\0\0", *b"GENMIDI\0"];

fn parse_lump(file: &Vec<u8>, info: &LumpInfo) -> Lump {
    if IGNORED_LUMPS.contains(&info.name) {
        return Lump::Ignored;
    }

    let file_position = info.file_position as usize;
    let data = &file[file_position..file_position + info.size as usize].to_vec();

    if info.name == *b"PLAYPAL\0" {
        return read_palette(data);
    }

    if info.name == *b"COLORMAP" {
        return read_colormap(data);
    }

    if info.name == *b"TITLEPIC" {
        return Lump::Patch(Patch::new(data));
    }

    if info.name.starts_with(b"DEMO") {
        return Lump::Demo(Demo::new(data));
    }

    if info.name.starts_with(b"DS") {
        return Lump::Sound(Sound::new(data));
    }

    if info.name.starts_with(b"DP") {
        return Lump::SoundPCSpeaker;
    }

    if info.name.starts_with(b"D_") {
        return Lump::Music(data.to_vec());
    }

    Lump::Unknown
}

fn read_palette(data: &[u8]) -> Lump {
    let palette_slice: &[[[u8; 3]; 256]] = bytemuck::cast_slice(&data);
    Lump::Palette(palette_slice.to_vec())
}

fn read_colormap(data: &[u8]) -> Lump {
    let colormap_slice: &[[u8; 256]] = bytemuck::cast_slice(&data);
    Lump::ColorMap(colormap_slice.to_vec())
}
