pub mod apu;
pub mod bus;
pub mod cpu;
pub mod input;
pub mod mapper;
pub mod png;
pub mod ppu;
pub mod rom;

use cpu::CPU;
use input::Button;
use ppu::{HEIGHT, WIDTH};
use rom::Rom;

pub struct Nes {
    pub cpu: CPU,
}

impl Nes {
    pub fn new(rom_data: &[u8], sample_rate: u32) -> Result<Nes, String> {
        let rom = Rom::new(rom_data)?;
        let mut cpu = CPU::new(mapper::from_rom(rom)?, sample_rate);
        cpu.reset();
        Ok(Nes { cpu })
    }

    pub fn step_frame(&mut self) {
        while !self.cpu.bus.ppu.frame_complete {
            self.cpu.step();
        }
        self.cpu.bus.ppu.frame_complete = false;
    }

    pub fn frame(&self) -> &[u32] {
        &self.cpu.bus.ppu.frame
    }

    pub fn pixel(&self, x: usize, y: usize) -> u32 {
        self.cpu.bus.ppu.frame[y * WIDTH + x]
    }

    pub fn set_button(&mut self, player: usize, button: Button, pressed: bool) {
        self.cpu.bus.controllers[player].set_button(button, pressed);
    }

    pub fn drain_audio(&mut self) -> Vec<f32> {
        self.cpu.bus.apu.drain_samples()
    }

    pub fn screen_text(&self) -> String {
        let mirroring = self.cpu.bus.mapper.mirroring();
        let mut text = String::new();
        for row in 0..30u16 {
            for column in 0..32u16 {
                let tile = self.cpu.bus.ppu.peek_vram(0x2000 + row * 32 + column, mirroring);
                text.push(match tile {
                    0x20..=0x7E => tile as char,
                    _ => ' ',
                });
            }
            text.push('\n');
        }
        text
    }

    pub fn screen_size(&self) -> (usize, usize) {
        (WIDTH, HEIGHT)
    }
}
