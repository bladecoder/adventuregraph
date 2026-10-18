//! Finite, monophonic buzzer playback shared by all platforms.
use alloc::format;
use bladeink::story_error::StoryError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuzzerCommand {
    Tone { frequency: u16, duration_ms: u16 },
    Pattern(BuzzerPattern),
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuzzerPattern {
    Phone,
    Error,
    Alarm,
}

// Frequency 0 represents a silent interval. Patterns never repeat indefinitely.
const PHONE: &[(u16, u16)] = &[
    (1800, 150),
    (0, 100),
    (1800, 150),
    (0, 600),
    (1800, 150),
    (0, 100),
    (1800, 150),
];
const ERROR: &[(u16, u16)] = &[(600, 150), (0, 80), (300, 250)];
const ALARM: &[(u16, u16)] = &[
    (900, 200),
    (1800, 200),
    (900, 200),
    (1800, 200),
    (900, 200),
    (1800, 200),
];

impl BuzzerCommand {
    pub(crate) fn parse(command: &str) -> Result<Self, StoryError> {
        let invalid =
            || StoryError::InvalidStoryState(format!("Invalid buzzer command: {command}"));
        let attributes: alloc::vec::Vec<_> = command.split_whitespace().skip(1).collect();
        if attributes.as_slice() == ["stop"] {
            return Ok(Self::Stop);
        }
        if let [attribute] = attributes.as_slice()
            && let Some(name) = attribute.strip_prefix("pattern=")
        {
            return Ok(Self::Pattern(match name {
                "phone" => BuzzerPattern::Phone,
                "error" => BuzzerPattern::Error,
                "alarm" => BuzzerPattern::Alarm,
                _ => return Err(invalid()),
            }));
        }
        let mut frequency = None;
        let mut duration = None;
        for attribute in attributes {
            let (key, value) = attribute.split_once('=').ok_or_else(invalid)?;
            if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
                return Err(invalid());
            }
            let value = value.parse::<u16>().map_err(|_| invalid())?;
            match key {
                "frequency" if frequency.is_none() && (100..=10000).contains(&value) => {
                    frequency = Some(value)
                }
                "duration" if duration.is_none() && (1..=10000).contains(&value) => {
                    duration = Some(value)
                }
                _ => return Err(invalid()),
            }
        }
        Ok(Self::Tone {
            frequency: frequency.ok_or_else(invalid)?,
            duration_ms: duration.ok_or_else(invalid)?,
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Buzzer {
    command: Option<BuzzerCommand>,
    step: usize,
    remaining_us: u64,
}

impl Buzzer {
    pub fn play(&mut self, command: BuzzerCommand) {
        *self = Self::default();
        if command != BuzzerCommand::Stop {
            self.command = Some(command);
            self.remaining_us = u64::from(self.segment().1) * 1000;
        }
    }

    fn segment(&self) -> (u16, u16) {
        match self.command {
            Some(BuzzerCommand::Tone {
                frequency,
                duration_ms,
            }) => (frequency, duration_ms),
            Some(BuzzerCommand::Pattern(pattern)) => {
                let sequence = match pattern {
                    BuzzerPattern::Phone => PHONE,
                    BuzzerPattern::Error => ERROR,
                    BuzzerPattern::Alarm => ALARM,
                };
                sequence.get(self.step).copied().unwrap_or_default()
            }
            _ => (0, 0),
        }
    }

    /// None during silence and after playback finishes.
    pub fn frequency(&self) -> Option<u16> {
        let frequency = self.segment().0;
        (frequency != 0).then_some(frequency)
    }

    /// Includes the silent intervals within a pattern.
    pub fn is_active(&self) -> bool {
        self.command.is_some()
    }

    pub fn tick_us(&mut self, mut elapsed_us: u64) {
        while self.is_active() {
            if elapsed_us < self.remaining_us {
                self.remaining_us -= elapsed_us;
                return;
            }
            elapsed_us -= self.remaining_us;
            self.step += 1;
            if matches!(self.command, Some(BuzzerCommand::Tone { .. })) {
                self.command = None;
            }
            self.remaining_us = u64::from(self.segment().1) * 1000;
            if self.remaining_us == 0 {
                *self = Self::default();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_commands_and_limits() {
        assert_eq!(
            BuzzerCommand::parse("buzzer duration=100 frequency=1200").unwrap(),
            BuzzerCommand::Tone {
                frequency: 1200,
                duration_ms: 100
            }
        );
        for valid in [
            "buzzer frequency=100 duration=1",
            "buzzer frequency=10000 duration=10000",
            "buzzer pattern=phone",
            "buzzer pattern=error",
            "buzzer pattern=alarm",
            "buzzer stop",
        ] {
            assert!(BuzzerCommand::parse(valid).is_ok(), "{valid}");
        }
        for invalid in [
            "buzzer",
            "buzzer stop extra",
            "buzzer pattern=unknown",
            "buzzer pattern=phone duration=1",
            "buzzer frequency=99 duration=1",
            "buzzer frequency=10001 duration=1",
            "buzzer frequency=1000 duration=0",
            "buzzer frequency=1000 duration=10001",
            "buzzer frequency=1000 duration=-1",
            "buzzer frequency=1000 duration=1.5",
            "buzzer frequency=+1000 duration=1",
            "buzzer frequency=1000 duration=65536",
            "buzzer frequency=1000 duration=1 duration=2",
            "buzzer frequency=1000 frequency=2000 duration=1",
            "buzzer frequency=1000",
            "buzzer duration=1",
            "buzzer volume=1",
        ] {
            assert!(BuzzerCommand::parse(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn finite_patterns_silences_interruption_and_stop() {
        let mut buzzer = Buzzer::default();
        buzzer.play(BuzzerCommand::Pattern(BuzzerPattern::Phone));
        assert_eq!(buzzer.frequency(), Some(1800));
        buzzer.tick_us(150_000);
        assert_eq!(buzzer.frequency(), None);
        assert!(buzzer.is_active());
        buzzer.tick_us(100_000);
        assert_eq!(buzzer.frequency(), Some(1800));
        buzzer.tick_us(1_150_000);
        assert!(!buzzer.is_active());
        for pattern in [
            BuzzerPattern::Phone,
            BuzzerPattern::Error,
            BuzzerPattern::Alarm,
        ] {
            buzzer.play(BuzzerCommand::Pattern(pattern));
            buzzer.tick_us(u64::MAX);
            assert!(!buzzer.is_active());
        }
        buzzer.play(BuzzerCommand::Pattern(BuzzerPattern::Alarm));
        buzzer.play(BuzzerCommand::Tone {
            frequency: 1200,
            duration_ms: 100,
        });
        buzzer.tick_us(99_999);
        assert_eq!(buzzer.frequency(), Some(1200));
        buzzer.tick_us(1);
        assert!(!buzzer.is_active());
        buzzer.play(BuzzerCommand::Pattern(BuzzerPattern::Alarm));
        buzzer.play(BuzzerCommand::Stop);
        assert_eq!(buzzer.frequency(), None);
        assert!(!buzzer.is_active());
    }
}
