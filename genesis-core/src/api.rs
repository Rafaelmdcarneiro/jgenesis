use crate::audio;
use crate::audio::AudioDownsampler;
use crate::input::{GenesisInputs, InputState};
use crate::memory::{Cartridge, CartridgeLoadError, MainBus, Memory};
use crate::vdp::{Vdp, VdpTickEffect};
use crate::ym2612::{Ym2612, YmTickEffect};
use jgenesis_traits::frontend::{AudioOutput, FrameSize, PixelAspectRatio, Renderer};
use m68000_emu::M68000;
use smsgg_core::psg::{Psg, PsgVersion};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::num::NonZeroU32;
use z80_emu::Z80;

const M68K_MCLK_DIVIDER: u64 = 7;
const Z80_MCLK_DIVIDER: u64 = 15;

#[derive(Debug)]
pub enum GenesisError<RErr, AErr> {
    Render(RErr),
    Audio(AErr),
}

impl<RErr, AErr> Display for GenesisError<RErr, AErr>
where
    RErr: Display,
    AErr: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Render(err) => write!(f, "Rendering error: {err}"),
            Self::Audio(err) => write!(f, "Audio error: {err}"),
        }
    }
}

impl<RErr, AErr> Error for GenesisError<RErr, AErr>
where
    RErr: Debug + Display + AsRef<dyn Error + 'static>,
    AErr: Debug + Display + AsRef<dyn Error + 'static>,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Render(err) => Some(err.as_ref()),
            Self::Audio(err) => Some(err.as_ref()),
        }
    }
}

pub type GenesisResult<RErr, AErr> = Result<GenesisTickEffect, GenesisError<RErr, AErr>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenesisTickEffect {
    None,
    FrameRendered,
}

#[derive(Debug, Clone)]
pub struct GenesisEmulator {
    memory: Memory,
    m68k: M68000,
    z80: Z80,
    vdp: Vdp,
    psg: Psg,
    ym2612: Ym2612,
    input: InputState,
    audio_downsampler: AudioDownsampler,
    master_clock_cycles: u64,
}

impl GenesisEmulator {
    /// Initialize the emulator from the given ROM.
    ///
    /// # Errors
    ///
    /// Returns an error if unable to parse the ROM header.
    pub fn from_rom(rom: Vec<u8>) -> Result<Self, CartridgeLoadError> {
        let cartridge = Cartridge::from_rom(rom)?;
        let memory = Memory::new(cartridge);

        // Genesis cartridges store the initial stack pointer in the first 4 bytes and the entry point
        // in the next 4 bytes
        let mut m68k = M68000::new();
        m68k.set_supervisor_stack_pointer(memory.read_rom_u32(0));
        m68k.set_pc(memory.read_rom_u32(4));

        let z80 = Z80::new();
        let vdp = Vdp::new();
        let psg = Psg::new(PsgVersion::Standard);
        let ym2612 = Ym2612::new();
        let input = InputState::new();

        Ok(Self {
            memory,
            m68k,
            z80,
            vdp,
            psg,
            ym2612,
            input,
            audio_downsampler: AudioDownsampler::new(),
            master_clock_cycles: 0,
        })
    }

    #[must_use]
    pub fn cartridge_title(&self) -> String {
        self.memory.cartridge_title()
    }

    /// Execute one 68000 CPU instruction and run the rest of the components for the appropriate
    /// number of cycles.
    ///
    /// # Errors
    ///
    /// This method will propagate any errors encountered while rendering frames or pushing audio
    /// samples.
    #[inline]
    #[allow(clippy::missing_panics_doc)]
    pub fn tick<R, A>(
        &mut self,
        renderer: &mut R,
        audio_output: &mut A,
        inputs: &GenesisInputs,
    ) -> GenesisResult<R::Err, A::Err>
    where
        R: Renderer,
        A: AudioOutput,
    {
        let mut bus = MainBus::new(
            &mut self.memory,
            &mut self.vdp,
            &mut self.psg,
            &mut self.ym2612,
            &mut self.input,
            self.z80.stalled(),
        );
        let m68k_cycles = self.m68k.execute_instruction(&mut bus);

        let elapsed_mclk_cycles = u64::from(m68k_cycles) * M68K_MCLK_DIVIDER;
        let z80_cycles = ((self.master_clock_cycles + elapsed_mclk_cycles) / Z80_MCLK_DIVIDER)
            - self.master_clock_cycles / Z80_MCLK_DIVIDER;
        self.master_clock_cycles += elapsed_mclk_cycles;

        for _ in 0..z80_cycles {
            self.z80.tick(&mut bus);
        }

        // The PSG uses the same master clock divider as the Z80, but it needs to be ticked in a
        // separate loop because MainBus holds a mutable reference to the PSG
        for _ in 0..z80_cycles {
            self.psg.tick();
        }

        // The YM2612 uses the same master clock divider as the 68000
        for _ in 0..m68k_cycles {
            if self.ym2612.tick() == YmTickEffect::OutputSample {
                let (ym_sample_l, ym_sample_r) = self.ym2612.sample();
                let (psg_sample_l, psg_sample_r) = self.psg.sample();

                // TODO more intelligent PSG mixing
                let sample_l =
                    (ym_sample_l + audio::PSG_COEFFICIENT * psg_sample_l).clamp(-1.0, 1.0);
                let sample_r =
                    (ym_sample_r + audio::PSG_COEFFICIENT * psg_sample_r).clamp(-1.0, 1.0);
                self.audio_downsampler
                    .collect_sample(sample_l, sample_r, audio_output)
                    .map_err(GenesisError::Audio)?;
            }
        }

        if self.vdp.tick(elapsed_mclk_cycles, &mut self.memory) == VdpTickEffect::FrameComplete {
            let frame_width = self.vdp.screen_width();
            let frame_height = self.vdp.screen_height();

            let frame_size = FrameSize {
                width: frame_width,
                height: frame_height,
            };
            // TODO configurable
            let pixel_aspect_ratio = match (frame_width, frame_height) {
                (256, 224) => PixelAspectRatio::from_width_and_height(
                    NonZeroU32::new(8).unwrap(),
                    NonZeroU32::new(7).unwrap(),
                ),
                (320, 224) => PixelAspectRatio::from_width_and_height(
                    NonZeroU32::new(32).unwrap(),
                    NonZeroU32::new(35).unwrap(),
                ),
                (256, 448) => PixelAspectRatio::from_width_and_height(
                    NonZeroU32::new(16).unwrap(),
                    NonZeroU32::new(7).unwrap(),
                ),
                (320, 448) => PixelAspectRatio::from_width_and_height(
                    NonZeroU32::new(64).unwrap(),
                    NonZeroU32::new(35).unwrap(),
                ),
                _ => panic!("Unexpected Genesis frame size: {frame_width}x{frame_height}"),
            };

            renderer
                .render_frame(
                    self.vdp.frame_buffer(),
                    frame_size,
                    Some(pixel_aspect_ratio),
                )
                .map_err(GenesisError::Render)?;

            self.input.set_inputs(inputs);

            return Ok(GenesisTickEffect::FrameRendered);
        }

        Ok(GenesisTickEffect::None)
    }
}
