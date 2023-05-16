#![forbid(unsafe_code)]

mod api;
mod apu;
pub mod audio;
mod bus;
mod cpu;
mod input;
mod num;
mod ppu;
mod serialize;

pub use api::{
    AudioPlayer, ColorEmphasis, EmulationError, EmulationResult, Emulator, EmulatorConfig,
    InputPoller, Renderer, SaveWriter, TickEffect,
};
pub use input::JoypadState;
pub use ppu::{FrameBuffer, SCREEN_HEIGHT, SCREEN_WIDTH, VISIBLE_SCREEN_HEIGHT};
