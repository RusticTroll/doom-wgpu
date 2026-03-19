use std::{collections::VecDeque, fs::File, sync::Arc};

use bytemuck::{Pod, Zeroable};
use rodio::{buffer::SamplesBuffer, nz};
use rustysynth::*;

#[derive(Debug)]
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

const SAMPLES_PER_TICK: usize = 44100 / 140;

impl Music {
    pub fn new(lump: &[u8]) -> Music {
        let header: MUSHeader = *bytemuck::from_bytes(&lump[0..MUS_HEADER_SIZE]);
        if header.signature != *b"MUS\x1A" {
            panic!("MUS lump is not a valid MUS file (invalid signature)");
        }

        let mut sf2 = File::open("soundfont.sf2").expect("Failed to open soundfont.sf2");
        let soundfont = Arc::new(SoundFont::new(&mut sf2).expect("Failed to load SoundFont"));
        let mut synth = Synthesizer::new(&soundfont, &SynthesizerSettings::new(44100))
            .expect("Failed to create Synthesizer");

        let song_end = (header.song_offset + header.song_length) as usize;
        let mut events = VecDeque::from_iter(lump[header.song_offset as usize..song_end].iter());
        let mut last_velocity = vec![0; 16];
        let mut left_samples = Vec::<f32>::new();
        let mut right_samples = Vec::<f32>::new();

        while events.len() > 0 {
            // Parse events until we get a delay
            // Once we reach a delay, render the waveform for `delay` ticks worth of samples
            if let Some(delay) = parse_mus_event(&mut synth, &mut last_velocity, &mut events) {
                let mut left_samples_for_tick = vec![0_f32; (SAMPLES_PER_TICK * delay)];
                let mut right_samples_for_tick = vec![0_f32; (SAMPLES_PER_TICK * delay)];
                synth.render(&mut left_samples_for_tick, &mut right_samples_for_tick);

                left_samples.append(&mut left_samples_for_tick);
                right_samples.append(&mut right_samples_for_tick);
            }
        }

        Self {
            sample_buffer: SamplesBuffer::new(nz!(1), nz!(44100), left_samples),
        }
    }
}

/**
 * Parses a MUS event which starts at the beginning of bytes
 * Returns the length in bytes of the parsed event
 */
pub fn parse_mus_event(
    synth: &mut Synthesizer,
    last_velocity: &mut Vec<i32>,
    bytes: &mut VecDeque<&u8>,
) -> Option<usize> {
    let event_head = bytes
        .pop_front()
        .expect("Tried to parse MUS event, but there were no bytes left");
    let delay_present = event_head & 0x80 != 0;
    let event_type = (event_head & 0x70) >> 4;
    let channel = (event_head & 0x0F) as i32;

    // println!(
    //     "Event head: {:#X}\n\tDelay: {}\n\tType: {}\n\tChannel: {}",
    //     *event_head, delay_present, event_type, channel
    // );

    match event_type {
        // Release Note
        0 => {
            let event = bytes
                .pop_front()
                .expect("Tried to parse Release Note event, but there were no bytes left");
            synth.note_off(channel, *event as i32);
        },
        // Play Note
        1 => {
            let event = bytes
                .pop_front()
                .expect("Tried to parse Play Note event, but there were no bytes left");
            if event & 0x80 == 0 {
                synth.note_on(
                    channel,
                    (event & 0x7F) as i32,
                    last_velocity[channel as usize],
                );
            } else {
                let velocity = bytes.pop_front().expect(
                    "Tried to parse Play Note event's velocity, but there were no bytes left",
                );
                synth.note_on(channel, (event & 0x7F) as i32, *velocity as i32);
                last_velocity[channel as usize] = *velocity as i32;
            }
        },
        // Pitch Bend
        2 => {
            let bend_amount = bytes
                .pop_front()
                .expect("Tried to parse Bend Pitch event, but there were no bytes left");
            synth.process_midi_message(
                channel,
                0xE0,
                (bend_amount << 7) as i32,
                (bend_amount >> 1) as i32,
            );
        },
        // System Event
        3 => {
            let event_id = bytes
                .pop_front()
                .expect("Tried to parse System event, but there were no bytes left");
            match event_id {
                // All Sounds Off
                10 => synth.process_midi_message(channel, 0xB0, 120, 0),
                // All Notes Off
                11 => synth.process_midi_message(channel, 0xB0, 123, 0),
                // Mono (One note per channel)
                12 => synth.process_midi_message(channel, 0xB0, 126, 0),
                // Poly (Multiple notes per channel)
                13 => synth.process_midi_message(channel, 0xB0, 127, 0),
                // Reset all controllers
                14 => synth.process_midi_message(channel, 0xB0, 121, 0),
                _ => {},
            }
        },
        // Controller
        4 => {
            let controller_number = bytes.pop_front().expect(
                "Tried to parse Controller event Controller Number, but there were no bytes left",
            );
            let value = *bytes
                .pop_front()
                .expect("Tried to parse Controller event Value, but there were no bytes left")
                as i32;

            match controller_number {
                // Change Instrument
                0 => synth.process_midi_message(channel, 0xC0, value, 0),
                // Bank Select
                1 => synth.process_midi_message(channel, 0xB0, 0, value),
                // Modulation
                2 => synth.process_midi_message(channel, 0xB0, 1, value),
                // Volume
                3 => synth.process_midi_message(channel, 0xB0, 7, value),
                // Pan
                4 => synth.process_midi_message(channel, 0xB0, 10, value),
                // Expression
                5 => synth.process_midi_message(channel, 0xB0, 11, value),
                // Reverb Depth
                6 => synth.process_midi_message(channel, 0xB0, 91, value),
                // Chorus Depth
                7 => synth.process_midi_message(channel, 0xB0, 93, value),
                // Sustain Pedal
                8 => synth.process_midi_message(channel, 0xB0, 64, value),
                // Soft Pedal
                9 => synth.process_midi_message(channel, 0xB0, 67, value),
                _ => {},
            }
        },
        5 => {
            println!("5")
        },
        6 => {
            println!("6")
        },
        7 => {
            println!("7")
        },
        other => panic!("Invalid MUS event type {}", other),
    };

    if delay_present {
        let mut total_delay: usize = 0;

        loop {
            let delay = *bytes
                .pop_front()
                .expect("Tried to parse delay, but there were no bytes left");
            total_delay = total_delay * 128 + (delay & 0x7F) as usize;
            if delay & 0x80 == 0 {
                break;
            }
        }

        //println!("Current Delay: {}", total_delay);

        Some(total_delay)
    } else {
        None
    }
}
