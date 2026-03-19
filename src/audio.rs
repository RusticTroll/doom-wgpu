use rodio::{mixer::*, *};

use crate::wad::{Music, Sound};

pub struct AudioManager {
    sink: MixerDeviceSink,
    music_player: Player,
    sound_mixer: Option<Mixer>,
}

impl AudioManager {
    pub fn new() -> Self {
        let sink =
            rodio::DeviceSinkBuilder::open_default_sink().expect("Failed to open audio sink");
        let music_player = Player::connect_new(&sink.mixer());
        let (sound_mixer, mixer_source) = mixer(nz!(1), nz!(44100));
        sink.mixer().add(mixer_source);

        Self {
            sink,
            music_player,
            sound_mixer: Some(sound_mixer),
        }
    }

    pub fn play_music(&self, music: &Music) {
        self.music_player.append(music.sample_buffer.clone());
    }

    pub fn repeat_forever(&self, sound: &Sound) {
        match &self.sound_mixer {
            Some(mixer) => mixer.add(sound.sample_buffer.clone().repeat_infinite()),
            _ => {},
        }
    }

    pub fn stop_all_sounds(&mut self) {
        self.music_player.stop();

        // Drop and reinitialize sound mixer to stop all sounds
        drop(self.sound_mixer.take());
        let (sound_mixer, mixer_source) = mixer(nz!(1), nz!(44100));
        self.sink.mixer().add(mixer_source);
        self.sound_mixer = Some(sound_mixer);
    }
}
