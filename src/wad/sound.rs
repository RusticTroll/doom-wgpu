use rodio::buffer::SamplesBuffer;
use std::{num::NonZero, sync::OnceLock};

#[derive(Debug)]
pub struct Sound {
    sample_buffer: SamplesBuffer,
}

static SOUND_MIXER: OnceLock<rodio::MixerDeviceSink> = OnceLock::new();

fn init_mixer() -> rodio::MixerDeviceSink {
    rodio::DeviceSinkBuilder::open_default_sink().expect("Failed to open audio sink")
}

impl Sound {
    pub fn new(lump: &[u8]) -> Self {
        let format: u16 = *bytemuck::from_bytes(&lump[0..2]);
        if format != 3 {
            panic!("Sound lump is format {}, expected format 3", format);
        }

        let sample_rate = *bytemuck::from_bytes::<u16>(&lump[2..4]);
        let sample_count = bytemuck::from_bytes::<u32>(&lump[4..8]) - 32;

        let samples_u8 = &lump[0x18..0x18 + sample_count as usize];
        let samples = samples_u8
            .iter()
            .map(|sample| dasp_sample::conv::u8::to_f32(*sample))
            .collect::<Vec<_>>();

        Self {
            sample_buffer: SamplesBuffer::new(
                NonZero::new(1).unwrap(),
                NonZero::new(sample_rate as u32).unwrap(),
                samples,
            ),
        }
    }

    pub fn play(&self) {
        let mixer = SOUND_MIXER.get_or_init(init_mixer);
        mixer.mixer().add(self.sample_buffer.clone());
    }
}
