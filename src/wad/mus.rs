use std::{fs::File, sync::Arc};

use bytemuck::{Pod, Zeroable};
use rodio::buffer::SamplesBuffer;
use rustysynth::*;

pub struct Music {
    pub sample_buffer: SamplesBuffer,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MUSHeader {
    pub signature: [u8; 4],
    pub song_length: u16,
    pub song_offset: u16,
    _primary_channels: u16,
    _secondary_channels: u16,
    _num_instruments: u16,
}

const MUS_HEADER_SIZE: usize = std::mem::size_of::<MUSHeader>();

impl Music {
    pub fn new(lump: &[u8]) {
        let header: MUSHeader = *bytemuck::from_bytes(&lump[0..MUS_HEADER_SIZE]);
        if header.signature != *b"MUS\x1A" {
            panic!("MUS lump is not a valid MUS file (invalid signature)");
        }

        let mut sf2 = File::open("soundfont.sf2").expect("Failed to open soundfont.sf2");
        let soundfont = Arc::new(SoundFont::new(&mut sf2).expect("Failed to load SoundFont"));

        let synth_settings = SynthesizerSettings::new(44100);
        let mut synth =
            Synthesizer::new(&soundfont, &synth_settings).expect("Failed to create Synthesizer");
    }
}

/**
 * Parses a MUS event which starts at the beginning of bytes
 * Returns the length in bytes of the parsed event
 */
pub fn parse_mus_event(synth: &mut Synthesizer, bytes: &[u8]) -> usize {
    let event_head = bytes[0];
    let delay_present = event_head & 0x80;
    let event_type = (event_head & 0x70) >> 4;
    let channel = (event_head & 0x0F) as i32;

    let event_offset = match event_type {
        0 => {
            let event = bytes[1];
            synth.note_off(channel, event as i32);
            1
        },
        1 => {
            let event = bytes[1];
            if event & 0x80 != 0 {
                synth.note_on(channel, (event & 0x7F) as i32, 127);
                1
            } else {
                let volume = bytes[2];
                synth.note_on(channel, (event & 0x7F) as i32, volume as i32);
                2
            }
        },
        other => panic!("Invalid MUS event type {}", other),
    };

    // Handle delay based on bytes

    0
}
