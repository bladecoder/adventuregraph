#![no_std]

extern crate alloc;

mod app;
mod ui;

pub use app::{App, AppEvent, StoryLine};

/// The compiled story is converted to a read-only Blade Ink image by build.rs.
/// Every platform opens these same bytes; the ESP linker places the static in flash.
pub static STORY_IMAGE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/story.inkb"));
