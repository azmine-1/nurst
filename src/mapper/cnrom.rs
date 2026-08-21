use super::{Banks, Mapper};
use crate::rom::{Mirroring, Rom};

pub struct Cnrom {
    banks: Banks,
    mirroring: Mirroring,
    chr_bank: u8,
}

impl Cnrom {
    pub fn new(rom: Rom) -> Self {
        let mirroring = rom.mirroring;
        Self { banks: Banks::new(&rom), mirroring, chr_bank: 0 }
    }
}

impl Mapper for Cnrom {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.banks.prg_ram_read(addr),
            0x8000..=0xFFFF => {
                let offset = (addr - 0x8000) as usize % self.banks.prg.len().max(1);
                self.banks.prg.get(offset).copied().unwrap_or(0)
            }
            _ => 0,
        }
    }

    fn cpu_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x6000..=0x7FFF => self.banks.prg_ram_write(addr, val),
            0x8000..=0xFFFF => self.chr_bank = val & 0x03,
            _ => {}
        }
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        self.banks.chr(self.chr_bank as usize, 0x2000, addr & 0x1FFF)
    }

    fn ppu_write(&mut self, addr: u16, val: u8) {
        self.banks.chr_write(self.chr_bank as usize, 0x2000, addr & 0x1FFF, val);
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
