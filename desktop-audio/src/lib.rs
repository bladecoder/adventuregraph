//! SDL audio output for both desktop frontends. Audio failure is nonfatal.
use adventuregraph_core::{App, Buzzer};
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::sync::{Arc, Mutex};

struct Oscillator {
    playback: Arc<Mutex<Buzzer>>,
    sample_rate: u32,
    phase: f32,
    remainder: u64,
}

impl AudioCallback for Oscillator {
    type Channel = f32;
    fn callback(&mut self, samples: &mut [f32]) {
        let Ok(mut playback) = self.playback.lock() else {
            samples.fill(0.0);
            return;
        };
        for sample in samples {
            *sample = if let Some(frequency) = playback.frequency() {
                let value = if self.phase < 0.5 { 0.12 } else { -0.12 };
                self.phase = (self.phase + f32::from(frequency) / self.sample_rate as f32).fract();
                value
            } else {
                self.phase = 0.0;
                0.0
            };
            self.remainder += 1_000_000;
            playback.tick_us(self.remainder / u64::from(self.sample_rate));
            self.remainder %= u64::from(self.sample_rate);
        }
    }
}

pub struct Audio {
    device: Option<AudioDevice<Oscillator>>,
    playback: Arc<Mutex<Buzzer>>,
    // Keep SDL alive until the audio device has closed.
    _subsystem: Option<sdl2::AudioSubsystem>,
    _sdl: Option<sdl2::Sdl>,
}

impl Default for Audio {
    fn default() -> Self {
        Self::new()
    }
}

impl Audio {
    pub fn new() -> Self {
        let playback = Arc::new(Mutex::new(Buzzer::default()));
        let open = || -> Result<_, String> {
            let sdl = sdl2::init()?;
            let subsystem = sdl.audio()?;
            let desired = AudioSpecDesired {
                freq: Some(48000),
                channels: Some(1),
                samples: Some(512),
            };
            let device = subsystem.open_playback(None, &desired, |spec| Oscillator {
                playback: Arc::clone(&playback),
                sample_rate: spec.freq as u32,
                phase: 0.0,
                remainder: 0,
            })?;
            device.resume();
            Ok((sdl, subsystem, device))
        };
        match open() {
            Ok((sdl, subsystem, device)) => Self {
                device: Some(device),
                playback,
                _subsystem: Some(subsystem),
                _sdl: Some(sdl),
            },
            Err(error) => {
                eprintln!("Audio unavailable; continuing silently: {error}");
                Self {
                    device: None,
                    playback,
                    _subsystem: None,
                    _sdl: None,
                }
            }
        }
    }

    pub fn play(&self, update: Buzzer) {
        if self.device.is_some()
            && let Ok(mut playback) = self.playback.lock()
        {
            *playback = update;
        }
    }

    pub fn sync(&self, app: &mut App) {
        if let Some(update) = app.take_buzzer_update() {
            self.play(update);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuregraph_core::BuzzerCommand;

    #[test]
    fn square_wave_has_requested_pitch_and_duration_then_silence() {
        let mut buzzer = Buzzer::default();
        buzzer.play(BuzzerCommand::Tone {
            frequency: 1000,
            duration_ms: 100,
        });
        let playback = Arc::new(Mutex::new(buzzer));
        let mut oscillator = Oscillator {
            playback,
            sample_rate: 48000,
            phase: 0.0,
            remainder: 0,
        };
        let mut samples = vec![0.0; 9600];
        oscillator.callback(&mut samples);
        assert!(samples[..4800].iter().all(|sample| sample.abs() == 0.12));
        let edges = samples[..4800]
            .windows(2)
            .filter(|pair| pair[0] != pair[1])
            .count();
        assert!((199..=200).contains(&edges), "{edges}");
        assert!(samples[4800..].iter().all(|sample| *sample == 0.0));
    }

    #[test]
    fn sample_clock_carries_fractional_microseconds() {
        let mut buzzer = Buzzer::default();
        buzzer.play(BuzzerCommand::Tone {
            frequency: 1000,
            duration_ms: 1000,
        });
        let playback = Arc::new(Mutex::new(buzzer));
        let mut oscillator = Oscillator {
            playback: Arc::clone(&playback),
            sample_rate: 44100,
            phase: 0.0,
            remainder: 0,
        };
        oscillator.callback(&mut vec![0.0; 44099]);
        assert!(playback.lock().unwrap().is_active());
        oscillator.callback(&mut [0.0]);
        assert!(!playback.lock().unwrap().is_active());
    }
}
