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
                todo!("PPU is not supported yet")
            }

            _ => {
                println!("Ignoring mem access at {}", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b11111111111;
                self.ram[mirror_down_addr as usize] = data;
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_addr_down = addr & 0b00100000_00000111;
                todo!("PPU is not yet supported");
            }

            _ => {
                println!("Ignoring mem write-access at {}", addr);
            }
        }
    }

    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos);
        let hi = self.mem_read(pos + 1);
        (hi as u16) << 8 | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let lo = (data & 0xFF) as u8;
        let hi = (data >> 8) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
}
