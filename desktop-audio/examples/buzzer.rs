//! Run with `cargo run -p adventuregraph-desktop-audio --example buzzer`.
use adventuregraph_core::{Buzzer, BuzzerCommand, BuzzerPattern};
use adventuregraph_desktop_audio::Audio;
use std::{thread, time::Duration};

fn main() {
    let audio = Audio::new();
    for (name, command) in [
        (
            "Tone: 1200 Hz, 100 ms",
            BuzzerCommand::Tone {
                frequency: 1200,
                duration_ms: 100,
            },
        ),
        ("Phone", BuzzerCommand::Pattern(BuzzerPattern::Phone)),
        ("Error", BuzzerCommand::Pattern(BuzzerPattern::Error)),
        ("Alarm", BuzzerCommand::Pattern(BuzzerPattern::Alarm)),
    ] {
        println!("{name}");
        let mut buzzer = Buzzer::default();
        buzzer.play(command);
        audio.play(buzzer);
        thread::sleep(Duration::from_secs(2));
    }
}
