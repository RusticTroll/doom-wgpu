mod linedef;
mod sidedef;
mod thing;

pub use linedef::*;
pub use sidedef::*;
pub use thing::*;

use super::LumpInfo;
use bytemuck::{Pod, Zeroable};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Map {
    pub things: Vec<MapThing>,
    pub linedefs: Vec<LineDef>,
    pub sidedefs: Vec<SideDef>,
}

impl Map {
    pub(super) fn new(file: &[u8], all_lump_info: &mut VecDeque<LumpInfo>) -> Self {
        let things_info = get_and_check_map_lump(all_lump_info, "THINGS");
        let linedefs_info = get_and_check_map_lump(all_lump_info, "LINEDEFS");
        let sidedefs_info = get_and_check_map_lump(all_lump_info, "SIDEDEFS");

        Self {
            things: get_map_data(file, things_info),
            linedefs: get_map_data(file, linedefs_info),
            sidedefs: get_map_data(file, sidedefs_info),
        }
    }
}

fn get_and_check_map_lump(all_lump_info: &mut VecDeque<LumpInfo>, expected_name: &str) -> LumpInfo {
    let lump_info = all_lump_info.pop_front().expect(&format!(
        "Expected {} lump, but there were no more lumps",
        expected_name
    ));

    if !lump_info.name.starts_with(expected_name.as_bytes()) {
        panic!(
            "Expected {} lump, but found lump of type {}",
            expected_name,
            String::from_utf8(lump_info.name.to_vec()).unwrap(),
        );
    }

    lump_info
}

fn get_map_data<T>(file: &[u8], lump_info: LumpInfo) -> Vec<T>
where
    T: Pod + Zeroable,
{
    let start_offset = lump_info.file_position as usize;
    let end_offset = start_offset + lump_info.size as usize;

    let data = &file[start_offset..end_offset].to_vec();

    bytemuck::cast_slice(data).to_vec()
}
