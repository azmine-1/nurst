pub struct Bus { 
    ram: [u8; 2048], 
    ppu_registers: [u8; 8],
    apu_io: [u8; 24],
    cartridge_rom: [u8; 32768],
}

impl Bus { 
    pub fn new() -> Self {
        Bus { 
            ram: [0; 2048],
            ppu_registers: [0; 8],
            apu_io: [0; 24].
            cartridge_rom: [0; 32768]
        }
    }
    
    pub fn read(&self, addr: u16) -> u8{
        match addr { 
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
            0x2000..=0x3FFF => self.ppu_registers[(addr & 0x0007) as usize],
            0x4000..=0x4017 => self.apu_io[(addr - 0x4000) as usize],
            _ = 0 
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        if addr < self.ram.len() as u16 {
            return self.ram[addr as usize];
        }
        0 
    }

}