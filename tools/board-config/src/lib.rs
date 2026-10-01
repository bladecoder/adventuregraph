//! Build-time board profiles. This crate never runs on the microcontroller.
use serde::Deserialize;
use std::collections::BTreeMap;

fn yes() -> bool {
    true
}
fn spi_mhz() -> u32 {
    40
}
fn rotation() -> u16 {
    180
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Pull {
    #[default]
    Up,
    Down,
    None,
}
impl Pull {
    fn rust(&self) -> &'static str {
        match self {
            Self::Up => "Up",
            Self::Down => "Down",
            Self::None => "None",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Display {
    cs: u8,
    reset: u8,
    dc: u8,
    mosi: u8,
    sck: u8,
    backlight: u8,
    #[serde(default = "yes")]
    backlight_active_high: bool,
    #[serde(default = "rotation")]
    rotation: u16,
    #[serde(default = "spi_mhz")]
    spi_mhz: u32,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Encoder {
    a: u8,
    b: u8,
    button: u8,
    #[serde(default)]
    reverse: bool,
    #[serde(default)]
    pull: Pull,
    #[serde(default = "yes")]
    button_active_low: bool,
    #[serde(default)]
    button_pull: Pull,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Button {
    pin: u8,
    #[serde(default = "yes")]
    active_low: bool,
    #[serde(default)]
    pull: Pull,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Buzzer {
    pin: u8,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Board {
    /// Explicit opt-in for boot strapping, USB, and module memory pins.
    #[serde(default)]
    allow_reserved_pins: bool,
    display: Display,
    encoder: Encoder,
    button: Option<Button>,
    buzzer: Option<Buzzer>,
}

impl Board {
    pub fn parse(source: &str) -> Result<Self, String> {
        let board: Self = toml_edit::de::from_str(source).map_err(|e| e.to_string())?;
        board.validate()?;
        Ok(board)
    }

    fn validate(&self) -> Result<(), String> {
        let d = &self.display;
        let e = &self.encoder;
        if ![0, 90, 180, 270].contains(&d.rotation) {
            return Err("display.rotation must be 0, 90, 180, or 270".into());
        }
        if !(1..=80).contains(&d.spi_mhz) {
            return Err("display.spi_mhz must be between 1 and 80".into());
        }
        let mut pins = BTreeMap::new();
        let mut check = |name: &str, pin: u8, output: bool| -> Result<(), String> {
            if !(pin <= 21 || (26..=46).contains(&pin)) {
                return Err(format!("{name}: GPIO{pin} does not exist on ESP32-S2"));
            }
            if output && pin == 46 {
                return Err(format!("{name}: GPIO46 is input-only"));
            }
            if !self.allow_reserved_pins && matches!(pin, 0 | 19 | 20 | 26..=32 | 45 | 46) {
                return Err(format!(
                    "{name}: GPIO{pin} is reserved for boot, USB, or module memory; verify the wiring and set allow_reserved_pins = true to opt in"
                ));
            }
            if let Some(previous) = pins.insert(pin, name.to_owned()) {
                return Err(format!(
                    "GPIO{pin} is assigned to both {previous} and {name}"
                ));
            }
            Ok(())
        };
        for (name, pin) in [
            ("display.cs", d.cs),
            ("display.reset", d.reset),
            ("display.dc", d.dc),
            ("display.mosi", d.mosi),
            ("display.sck", d.sck),
            ("display.backlight", d.backlight),
        ] {
            check(name, pin, true)?;
        }
        for (name, pin) in [
            ("encoder.a", e.a),
            ("encoder.b", e.b),
            ("encoder.button", e.button),
        ] {
            check(name, pin, false)?;
        }
        if let Some(b) = &self.button {
            check("button.pin", b.pin, false)?;
        }
        if let Some(b) = &self.buzzer {
            check("buzzer.pin", b.pin, true)?;
        }
        Ok(())
    }

    /// Generate typed peripheral moves; esp-hal still checks pin capabilities.
    pub fn generate(&self) -> String {
        let d = &self.display;
        let e = &self.encoder;
        let button = self
            .button
            .as_ref()
            .map(|b| {
                format!(
                    "Some(Input::new($p.GPIO{}, InputConfig::default().with_pull(Pull::{})))",
                    b.pin,
                    b.pull.rust()
                )
            })
            .unwrap_or_else(|| "None::<Input<'static>>".into());
        let buzzer = self
            .buzzer
            .as_ref()
            .map(|b| {
                format!(
                    "Some(Output::new($p.GPIO{}, Level::Low, OutputConfig::default()))",
                    b.pin
                )
            })
            .unwrap_or_else(|| "None::<Output<'static>>".into());
        let active_low = self.button.as_ref().is_none_or(|b| b.active_low);
        format!(
            r#"
pub const SPI_MHZ: u32 = {spi};
pub const ROTATION: Rotation = Rotation::Deg{rotation};
pub const BACKLIGHT_ON: Level = Level::{level};
pub const ENCODER_REVERSE: bool = {reverse};
pub const ENCODER_BUTTON_ACTIVE_LOW: bool = {enc_low};
pub const USER_BUTTON_ACTIVE_LOW: bool = {user_low};
macro_rules! board_pins {{
    ($p:ident) => {{ {{
        ($p.GPIO{bl}, $p.GPIO{dc}, $p.GPIO{reset}, $p.GPIO{cs},
         {buzzer}, $p.GPIO{sck}, $p.GPIO{mosi},
         Input::new($p.GPIO{a}, InputConfig::default().with_pull(Pull::{pull})),
         Input::new($p.GPIO{b}, InputConfig::default().with_pull(Pull::{pull})),
         Input::new($p.GPIO{enc}, InputConfig::default().with_pull(Pull::{enc_pull})),
         {button})
    }} }};
}}
"#,
            spi = d.spi_mhz,
            rotation = d.rotation,
            level = if d.backlight_active_high {
                "High"
            } else {
                "Low"
            },
            reverse = e.reverse,
            enc_low = e.button_active_low,
            user_low = active_low,
            bl = d.backlight,
            dc = d.dc,
            reset = d.reset,
            cs = d.cs,
            sck = d.sck,
            mosi = d.mosi,
            a = e.a,
            b = e.b,
            pull = e.pull.rust(),
            enc = e.button,
            enc_pull = e.button_pull.rust()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const V10: &str = include_str!("../../../esp/boards/v1.0.toml");
    const V11: &str = include_str!("../../../esp/boards/v1.1.toml");
    #[test]
    fn pcb_profiles_match_historical_wiring() {
        let old = Board::parse(V10).unwrap();
        let new = Board::parse(V11).unwrap();
        assert!(old.button.is_none());
        assert_eq!(old.buzzer.unwrap().pin, 18);
        assert_eq!(new.button.as_ref().unwrap().pin, 37);
        assert_eq!(new.buzzer.as_ref().unwrap().pin, 1);
        assert!(new.generate().contains("$p.GPIO37"));
    }
    #[test]
    fn minimal_profile_uses_documented_defaults() {
        let optional = [
            "backlight_active_high",
            "rotation",
            "spi_mhz",
            "reverse",
            "pull",
            "button_active_low",
            "button_pull",
            "active_low",
        ];
        let minimal = V11
            .lines()
            .filter(|line| {
                !optional
                    .iter()
                    .any(|key| line.starts_with(&format!("{key} =")))
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            Board::parse(&minimal).unwrap().generate(),
            Board::parse(V11).unwrap().generate()
        );
        assert!(Board::parse(&minimal.replace("cs = 3", "")).is_err());
    }
    #[test]
    fn rejects_duplicate_nonexistent_and_input_only_pins() {
        for (replacement, message) in [
            ("3", "both"),
            ("22", "does not exist"),
            ("46", "input-only"),
        ] {
            let err = Board::parse(&V11.replace("pin = 1\n", &format!("pin = {replacement}\n")))
                .unwrap_err();
            assert!(err.contains(message), "{err}");
        }
    }
    #[test]
    fn reserved_pins_require_explicit_opt_in() {
        let profile = V11.replace("pin = 1\n", "pin = 19\n");
        assert!(Board::parse(&profile).unwrap_err().contains("reserved"));
        Board::parse(&format!("allow_reserved_pins = true\n{profile}")).unwrap();
    }
    #[test]
    fn rejects_typos_invalid_rotation_speed_and_pull() {
        for profile in [
            V11.replace("spi_mhz", "spi_mzh"),
            V11.replace("rotation = 180", "rotation = 45"),
            V11.replace("spi_mhz = 40", "spi_mhz = 0"),
            V11.replace("spi_mhz = 40", "spi_mhz = 81"),
            V11.replace("\"up\"", "\"sideways\""),
        ] {
            assert!(Board::parse(&profile).is_err());
        }
    }
    #[test]
    fn generates_optional_devices_and_custom_settings() {
        let profile = V11
            .replace("[buzzer]\npin = 1\n", "")
            .replace("active_low = true", "active_low = false")
            .replace("reverse = false", "reverse = true")
            .replace("rotation = 180", "rotation = 90")
            .replace(
                "backlight_active_high = true",
                "backlight_active_high = false",
            )
            .replace("\"up\"", "\"down\"");
        let code = Board::parse(&profile).unwrap().generate();
        for value in [
            "None::<Output<'static>>",
            "Deg90",
            "BACKLIGHT_ON: Level = Level::Low",
            "ENCODER_REVERSE: bool = true",
            "USER_BUTTON_ACTIVE_LOW: bool = false",
            "Pull::Down",
        ] {
            assert!(code.contains(value), "{value}");
        }
    }
}
