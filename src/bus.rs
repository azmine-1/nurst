use crate::cpu::Mem;
const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

pub struct Bus {
    ram: [u8; 2048],
    ppu_registers: [u8; 8],
    apu_io: [u8; 24],
    cartridge_rom: [u8; 32768],
}

impl Bus {
    pub fn new() -> Self {
        Self {
            ram: [0; 2048],
            ppu_registers: [0; 8],
            apu_io: [0; 24],
            cartridge_rom: [0; 32768],
        }
    }

    pub fn load_rom(&mut self, rom: &[u8], start_addr: u16) {
        let start = start_addr as usize;
        for (i, &byte) in rom.iter().enumerate() {
            if start_addr >= 0x8000 {
                let rom_addr = (start + i) - 0x8000;
                if rom_addr < self.cartridge_rom.len() {
                    self.cartridge_rom[rom_addr] = byte;
                }
            }
        }
    }

    pub fn mem_read_u16_zp(&self, pos: u8) -> u16 {
        let lo = self.mem_read(pos as u16);
        let hi = self.mem_read(pos.wrapping_add(1) as u16);
        (hi as u16) << 8 | (lo as u16)
    }
}

impl Mem for Bus {
    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.ram[mirror_down_addr as usize]
            }

            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_addr_down = addr & 0b00100000_000001111;
                0
            }

            0x8000..=0xFFFF => {
                let rom_addr = ((addr - 0x8000) % 0x4000) as usize;
                self.cartridge_rom[rom_addr]
            }

            _ => {
                eprintln!("WARNING: Ignoring mem access at {:#06X}", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.ram[mirror_down_addr as usize] = data;
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {}

            _ => {
                eprintln!("WARNING: Ignoring mem write-access at {}", addr);
            }
        }
    }

    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos);
        let hi = self.mem_read(pos.wrapping_add(1));
        (hi as u16) << 8 | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let lo = (data & 0xFF) as u8;
        let hi = (data >> 8) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
}
