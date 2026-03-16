#[derive(Debug)]
pub struct Sound {
    pub sample_rate: u16,
    pub samples: Vec<u8>,
}

impl Sound {
    pub fn new(lump: &[u8]) -> Self {
        let format: u16 = *bytemuck::from_bytes(&lump[0..2]);
        if format != 3 {
            panic!("Sound lump is format {}, expected format 3", format);
        }

        let sample_rate =  *bytemuck::from_bytes(&lump[2..4]);
        let sample_count = bytemuck::from_bytes::<u32>(&lump[4..8]) - 32;

        Self {
            sample_rate,
            samples: lump[0x18..0x18 + sample_count as usize].to_vec(),
        }
    }
}