use crate::memory::Memory;

pub struct Bus {
    ram: [u8; 0x0800],
    rom: Vec<u8>,
}

impl Bus {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            ram: [0; 0x0800],
            rom,
        }
    }
}

impl Memory for Bus {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
            0x8000..=0xFFFF => {
                let index = (addr - 0x8000) as usize;
                if index < self.rom.len() {
                    self.rom[index]
                } else {
                    0
                }
            }
            _ => 0, // unmapped
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        if addr <= 0x1FFF {
            self.ram[(addr & 0x07FF) as usize] = value;
        }
        // ignore writes to ROM
    }
}
