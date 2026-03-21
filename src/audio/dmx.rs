use bytemuck::{Pod, Zeroable};
use rodio::*;
use rustysynth::*;
use std::{collections::VecDeque, fs::File, sync::Arc};

const MUS_HEADER_SIZE: usize = std::mem::size_of::<MUSHeader>();

const SAMPLES_PER_TICK: usize = 44100 / 140;

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

pub struct StreamedDMX {
    data: Vec<u8>,
    data_offset: usize,
    last_velocity: Vec<i32>,
    synth: Synthesizer,
    samples: VecDeque<rodio::Sample>,
    repeat: bool,
}

impl StreamedDMX {
    pub fn new(lump: &[u8], repeat: bool) -> Self {
        let header: MUSHeader = *bytemuck::from_bytes(&lump[0..MUS_HEADER_SIZE]);
        if header.signature != *b"MUS\x1A" {
            panic!("MUS lump is not a valid MUS file (invalid signature)");
        }

        let mut sf2 = File::open("soundfont.sf2").expect("Failed to open soundfont.sf2");
        let soundfont = Arc::new(
            SoundFont::new(&mut sf2)
                .expect("Successfully opened soundfont.sf2, but failed to load SoundFont"),
        );
        let synth = Synthesizer::new(&soundfont, &SynthesizerSettings::new(44100))
            .expect("Failed to create Synthesizer");

        let song_end = (header.song_offset + header.song_length) as usize;
        let mut data = Vec::<u8>::new();
        data.extend_from_slice(&lump[header.song_offset as usize..song_end]);

        Self {
            data,
            data_offset: 0,
            last_velocity: vec![0; 16],
            synth,
            samples: VecDeque::new(),
            repeat,
        }
    }

    fn get_byte(&mut self) -> u8 {
        let next_byte = self.data[self.data_offset];
        if self.repeat {
            self.data_offset = (self.data_offset + 1) % self.data.len();
        } else {
            self.data_offset += 1;
        }

        next_byte
    }

    /// Parses the next event
    ///
    /// Returns [Some(usize)](Some) if this event is followed by a delay, otherwise [None]
    fn parse_mus_event(&mut self) -> Option<usize> {
        let event_head = self.get_byte();
        let delay_present = event_head & 0x80 != 0;
        let event_type = (event_head & 0x70) >> 4;
        let mut channel = (event_head & 0x0F) as i32;

        if channel == 15 {
            channel = 9;
        } else if channel >= 9 {
            channel += 1;
        }

        match event_type {
            // Release Note
            0 => {
                let event = self.get_byte();
                self.synth.note_off(channel, event as i32);
            },
            // Play Note
            1 => {
                let event = self.get_byte();
                if event & 0x80 == 0 {
                    self.synth.note_on(
                        channel,
                        (event & 0x7F) as i32,
                        self.last_velocity[channel as usize],
                    );
                } else {
                    let velocity = self.get_byte();
                    self.synth
                        .note_on(channel, (event & 0x7F) as i32, (velocity & 0x7F) as i32);
                    self.last_velocity[channel as usize] = (velocity & 0x7F) as i32;
                }
            },
            // Pitch Bend
            2 => {
                let bend_amount = self.get_byte();
                self.synth.process_midi_message(
                    channel,
                    0xE0,
                    (bend_amount << 7) as i32,
                    (bend_amount >> 1) as i32,
                );
            },
            // System Event
            3 => {
                let event_id = self.get_byte();
                match event_id {
                    // All Sounds Off
                    10 => self.synth.process_midi_message(channel, 0xB0, 120, 0),
                    // All Notes Off
                    11 => self.synth.process_midi_message(channel, 0xB0, 123, 0),
                    // Mono (One note per channel)
                    12 => self.synth.process_midi_message(channel, 0xB0, 126, 0),
                    // Poly (Multiple notes per channel)
                    13 => self.synth.process_midi_message(channel, 0xB0, 127, 0),
                    // Reset all controllers
                    14 => self.synth.process_midi_message(channel, 0xB0, 121, 0),
                    _ => {},
                }
            },
            // Controller
            4 => {
                let controller_number = self.get_byte();
                let value = self.get_byte() as i32;

                match controller_number {
                    // Change Instrument
                    0 => self.synth.process_midi_message(channel, 0xC0, value, 0),
                    // Bank Select
                    1 => self.synth.process_midi_message(channel, 0xB0, 0, value),
                    // Modulation
                    2 => self.synth.process_midi_message(channel, 0xB0, 1, value),
                    // Volume
                    3 => self.synth.process_midi_message(channel, 0xB0, 7, value),
                    // Pan
                    4 => self.synth.process_midi_message(channel, 0xB0, 10, value),
                    // Expression
                    5 => self.synth.process_midi_message(channel, 0xB0, 11, value),
                    // Reverb Depth
                    6 => self.synth.process_midi_message(channel, 0xB0, 91, value),
                    // Chorus Depth
                    7 => self.synth.process_midi_message(channel, 0xB0, 93, value),
                    // Sustain Pedal
                    8 => self.synth.process_midi_message(channel, 0xB0, 64, value),
                    // Soft Pedal
                    9 => self.synth.process_midi_message(channel, 0xB0, 67, value),
                    _ => {},
                }
            },
            5 => {},
            6 => {},
            7 => {},
            other => panic!("Invalid MUS event type {}", other),
        };

        if delay_present {
            let mut total_delay: usize = 0;

            loop {
                let delay = self.get_byte();
                total_delay = total_delay * 128 + (delay & 0x7F) as usize;
                if delay & 0x80 == 0 {
                    break;
                }
            }

            Some(total_delay)
        } else {
            None
        }
    }

    /// Parses events until the next delay (or the file ends) and appends the samples to the sample buffer
    fn parse_mus_events(&mut self) {
        loop {
            if self.data_offset >= self.data.len() {
                break;
            }

            if let Some(delay) = self.parse_mus_event() {
                let mut left_samples_for_ticks = vec![0_f32; SAMPLES_PER_TICK * delay];
                let mut right_samples_for_ticks = vec![0_f32; SAMPLES_PER_TICK * delay];
                self.synth
                    .render(&mut left_samples_for_ticks, &mut right_samples_for_ticks);

                let interleaved_samples = left_samples_for_ticks
                    .iter()
                    .zip(right_samples_for_ticks.iter())
                    .flat_map(|(&left, &right)| [left, right]);

                self.samples.extend(interleaved_samples);
                break;
            }
        }
    }
}

impl Iterator for StreamedDMX {
    type Item = rodio::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data_offset >= self.data.len() {
            return None;
        }

        if self.samples.len() == 0 {
            self.parse_mus_events();
        }

        self.samples.pop_front()
    }
}

impl rodio::Source for StreamedDMX {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> rodio::ChannelCount {
        nz!(2)
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        nz!(44100)
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
