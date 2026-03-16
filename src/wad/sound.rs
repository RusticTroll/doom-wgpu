#[derive(Debug)]
pub struct Sound {
    pub sample_rate: u16,
    pub samples: Vec<u8>,
}

impl Sound {
    pub fn new(lump: &[u8]) -> Self {
        let format: u16 = bytemuck::try_pod_read_unaligned(&lump[0..2]).expect("Failed to read sound format");
        if format != 3 {
            panic!("Sound lump is format {}, expected format 3", format);
        }

        let sample_rate =  bytemuck::try_pod_read_unaligned(&lump[2..4]).expect("Failed to read sound format");
        let sample_count = bytemuck::try_pod_read_unaligned::<u32>(&lump[4..8]).expect("Failed to read sound format") - 32;

        Self {
            sample_rate,
            samples: lump[0x18..0x18 + sample_count as usize].to_vec(),
        }
    }
}