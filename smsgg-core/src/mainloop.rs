use crate::bus::Bus;
use crate::input::InputState;
use crate::memory::Memory;
use crate::psg::{Psg, PsgTickEffect};
use crate::vdp::{FrameBuffer, Vdp, VdpTickEffect, VdpVersion};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};
use minifb::{Key, Window, WindowOptions};
use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, process, thread};
use z80_emu::Z80;

// TODO generalize all this
/// # Panics
///
/// Panics if the file cannot be read
pub fn run(path: &str) {
    let file_name = Path::new(path).file_name().unwrap().to_str().unwrap();

    let mut window = Window::new(file_name, 3 * 256, 3 * 192, WindowOptions::default()).unwrap();
    window.limit_update_rate(Some(Duration::from_micros(16600)));

    let mut minifb_buffer = vec![0_u32; 256 * 192];

    let mut audio_buffer = Vec::new();
    let audio_queue = Arc::new(Mutex::new(VecDeque::<f32>::new()));
    let callback_queue = Arc::clone(&audio_queue);

    let audio_host = cpal::default_host();
    let audio_device = audio_host.default_output_device().unwrap();
    let audio_stream = audio_device
        .build_output_stream(
            &StreamConfig {
                channels: 1,
                sample_rate: SampleRate(48000),
                buffer_size: BufferSize::Fixed(1024),
            },
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut callback_queue = callback_queue.lock().unwrap();
                for output in data {
                    let Some(sample) = callback_queue.pop_front() else { break };
                    *output = sample;
                }
            },
            move |err| {
                log::error!("Audio error: {err}");
                process::exit(1);
            },
            None,
        )
        .unwrap();
    audio_stream.play().unwrap();

    let rom = fs::read(Path::new(path)).unwrap();
    let mut memory = Memory::new(rom);

    let mut z80 = Z80::new();
    z80.set_pc(0x0000);
    z80.set_sp(0xDFFF);

    let mut vdp = Vdp::new(VdpVersion::MasterSystem);
    let mut psg = Psg::new();
    let mut input = InputState::new();

    let mut sample_count = 0_u64;
    let downsampling_ratio = 53693175.0 / 15.0 / 16.0 / 48000.0;

    let mut leftover_vdp_cycles = 0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let t_cycles =
            z80.execute_instruction(&mut Bus::new(&mut memory, &mut vdp, &mut psg, &mut input))
                + leftover_vdp_cycles;

        for _ in 0..t_cycles {
            if psg.tick() == PsgTickEffect::Clocked {
                let sample = psg.sample();

                let prev_count = sample_count;
                sample_count += 1;

                if (prev_count as f64 / downsampling_ratio).round() as u64
                    != (sample_count as f64 / downsampling_ratio).round() as u64
                {
                    audio_buffer.push(sample as f32);
                    if audio_buffer.len() == 64 {
                        loop {
                            {
                                let mut audio_queue = audio_queue.lock().unwrap();
                                if audio_queue.len() < 1024 {
                                    audio_queue.extend(audio_buffer.drain(..));
                                    break;
                                }
                            }

                            thread::sleep(Duration::from_micros(250));
                        }
                    }
                }
            }
        }

        leftover_vdp_cycles = t_cycles % 2;

        let vdp_cycles = t_cycles / 2 * 3;
        for _ in 0..vdp_cycles {
            if vdp.tick() == VdpTickEffect::FrameComplete {
                let vdb_buffer = vdp.frame_buffer();

                vdp_buffer_to_minifb_buffer(vdb_buffer, &mut minifb_buffer);

                window.update_with_buffer(&minifb_buffer, 256, 192).unwrap();

                let p1_input = input.p1();
                p1_input.up = window.is_key_down(Key::Up);
                p1_input.left = window.is_key_down(Key::Left);
                p1_input.right = window.is_key_down(Key::Right);
                p1_input.down = window.is_key_down(Key::Down);
                p1_input.button_1 = window.is_key_down(Key::S);
                p1_input.button_2 = window.is_key_down(Key::A);
            }
        }
    }
}

fn vdp_buffer_to_minifb_buffer(vdp_buffer: &FrameBuffer, minifb_buffer: &mut [u32]) {
    for (i, row) in vdp_buffer[..192].iter().enumerate() {
        for (j, sms_color) in row.iter().copied().enumerate() {
            let r = convert_sms_color(sms_color & 0x03);
            let g = convert_sms_color((sms_color >> 2) & 0x03);
            let b = convert_sms_color((sms_color >> 4) & 0x03);

            minifb_buffer[i * 256 + j] = (r << 16) | (g << 8) | b;
        }
    }
}

fn convert_sms_color(color: u8) -> u32 {
    [0, 85, 170, 255][color as usize]
}
