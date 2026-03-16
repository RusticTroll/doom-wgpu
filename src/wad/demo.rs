use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct DemoHeader {
    game_version: u8,
    skill_level: u8,
    episode: u8,
    map: u8,
    multiplayer_mode: u8,
    respawn: u8,
    fast: u8,
    no_monsters: u8,
    pov_player: u8,
    p1_present: u8,
    p2_present: u8,
    p3_present: u8,
    p4_present: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct DemoTic {
    stride: u8,
    strafe: u8,
    turn: u8,
    flags: u8,
}

#[derive(Clone, Debug)]
pub struct Demo {
    header: DemoHeader,
    tics: Vec<DemoTic>,
}

const DEMO_HEADER_SIZE: usize = std::mem::size_of::<DemoHeader>();

impl Demo {
    pub fn new(lump: &[u8]) -> Self {
        let header: DemoHeader = *bytemuck::from_bytes(&lump[..DEMO_HEADER_SIZE]);
        let tics: Vec<DemoTic> =
            bytemuck::cast_slice(&lump[DEMO_HEADER_SIZE..lump.len() - 1]).to_vec();

        Self { header, tics }
    }
}
