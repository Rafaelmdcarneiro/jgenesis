use crate::ppu::registers::Registers;
use crate::ppu::{PpuFrameBuffer, Vram, SCREEN_WIDTH};
use bincode::{Decode, Encode};
use jgenesis_common::num::GetBit;
use std::collections::VecDeque;

const MAX_FIFO_X: u8 = SCREEN_WIDTH as u8 + 8;

#[derive(Debug, Clone, Copy, Encode, Decode)]
struct BgPixel {
    color: u8,
}

#[derive(Debug, Clone, Copy, Encode, Decode)]
struct SpritePixel {
    color: u8,
    palette: u8,
    low_priority: bool,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PixelFifo {
    bg: VecDeque<BgPixel>,
    sprites: VecDeque<SpritePixel>,
    x: u8,
    y: u8,
    fine_x_scroll: u8,
    delay: u8,
}

impl PixelFifo {
    pub fn new() -> Self {
        Self {
            bg: VecDeque::with_capacity(16),
            sprites: VecDeque::with_capacity(16),
            x: 0,
            y: 0,
            fine_x_scroll: 0,
            delay: 0,
        }
    }

    pub fn start_new_line(&mut self, scanline: u8, registers: &Registers) {
        self.bg.clear();
        self.sprites.clear();
        self.x = 0;
        self.y = scanline;
        self.fine_x_scroll = registers.bg_x_scroll & 0x7;
    }

    fn fetch_bg_tile_row(&mut self, vram: &Vram, registers: &Registers) {
        log::trace!("Fetching tile row at X={}, SCX={}", self.x, registers.bg_x_scroll);

        let bg_x: u16 = self
            .x
            .wrapping_add(self.fine_x_scroll)
            .wrapping_add(registers.bg_x_scroll & !0x7)
            .into();
        let bg_y: u16 = self.y.wrapping_add(registers.bg_y_scroll).into();

        let tile_map_x = bg_x / 8;
        let tile_map_y = bg_y / 8;

        let tile_map_addr = registers.bg_tile_map_addr | (tile_map_y << 5) | tile_map_x;
        let tile_number = vram[tile_map_addr as usize];

        let tile_data_addr = registers.bg_tile_data_area.tile_address(tile_number) + 2 * (bg_y % 8);
        let tile_data_low = vram[tile_data_addr as usize];
        let tile_data_high = vram[(tile_data_addr + 1) as usize];

        for bit in (0..8).rev() {
            let color = u8::from(tile_data_low.bit(bit)) | (u8::from(tile_data_high.bit(bit)) << 1);
            self.bg.push_back(BgPixel { color });
        }
    }

    pub fn tick(&mut self, vram: &Vram, registers: &Registers, frame_buffer: &mut PpuFrameBuffer) {
        if self.delay != 0 {
            self.delay -= 1;
            return;
        }

        if self.x == 0 {
            if self.bg.is_empty() {
                self.fetch_bg_tile_row(vram, registers);
                for _ in 0..self.fine_x_scroll {
                    self.bg.pop_front();
                }

                if self.fine_x_scroll >= 2 {
                    self.fetch_bg_tile_row(vram, registers);
                }

                self.delay = 6 + self.fine_x_scroll;

                return;
            }
        }

        let bg_pixel = self.bg.pop_front().expect("BG FIFO should never be empty past X=0");
        if self.x >= 8 {
            let color = registers.bg_palettes[bg_pixel.color as usize];
            frame_buffer.set(self.y, self.x - 8, color);
        }
        self.x += 1;

        if self.bg.len() == 6 {
            self.fetch_bg_tile_row(vram, registers);
        }
    }

    pub fn done_with_line(&self) -> bool {
        self.x >= MAX_FIFO_X
    }
}
