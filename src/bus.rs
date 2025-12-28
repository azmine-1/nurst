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


impl Mem for Bus{ 
    fn mem_read(&self, addr: u16) -> u8{ 
        match addr {
            RAM ..= RAM_MIRRORS_END => { 
                let mirror_down_addr = addr & 0b00000111_11111111; 
                self.cpu_vram[mirror_down_addr as usize]
            }

            PPU_REGISTERS ..= PPU_REGISTERS_MIRRORS_END => { 
                let _mirror_addr_down = addr & 0b00100000_000001111; 
                todo!("PPU is not supported yet")
            }

            _ => {
                println!("Ignoring mem access at {}", addr);
                0
            }
        }
    }
}
