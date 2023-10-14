/// Struct that represents an Sin wave
pub struct Wave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl sdl2::audio::AudioCallback for Wave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.iter_mut() {
            *x = self.volume * self.phase.sin();
            self.phase += self.phase_inc;
        }
    }
}

enum State {
    Playing,
    Stopped,
}

/// Struct that represents the chip-8 speaker
pub struct Speaker {
    state: State,
    device: sdl2::audio::AudioDevice<Wave>,
}

impl Speaker {
    /// Creates a new `Speaker`.
    ///
    /// # Arguments
    ///
    /// * `audio_subsystem` - The `AudioSubsystem` from the `sdl2` crate.
    ///
    /// # Returns
    ///
    /// * `Result<Speaker, String>` - Returns Ok if the speaker was created successfully, otherwise returns an error.
    pub fn new(audio_subsystem: &sdl2::AudioSubsystem) -> Result<Self, String> {
        let spec = sdl2::audio::AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };

        let device = audio_subsystem.open_playback(None, &spec, |spec| Wave {
            phase_inc: 440.0 * 2.0 * std::f32::consts::PI / spec.freq as f32,
            phase: 0.0,
            volume: 0.25,
        })?;

        Ok(Self {
            state: State::Stopped,
            device,
        })
    }

    /// Plays the speaker.
    ///
    /// If the speaker is already playing, this function does nothing.
    pub fn play(&mut self) {
        if let State::Playing = self.state {
            return;
        }
        self.device.resume();
        self.state = State::Playing;
    }

    /// Stops the speaker.
    ///
    /// If the speaker is already stopped, this function does nothing.
    pub fn stop(&mut self) {
        if let State::Stopped = self.state {
            return;
        }
        self.device.pause();
        self.state = State::Stopped;
    }
}
