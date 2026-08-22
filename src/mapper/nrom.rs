use super::{Banks, Mapper};
use crate::rom::{Mirroring, Rom};

/// Mapper 0: 16 or 32 KB of fixed PRG, 8 KB of fixed CHR.
pub struct Nrom {
    banks: Banks,
    mirroring: Mirroring,
}

impl Nrom {
    pub fn new(rom: Rom) -> Self {
        let mirroring = rom.mirroring;
        Self { banks: Banks::new(&rom), mirroring }
    }
}

impl Mapper for Nrom {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.banks.prg_ram_read(addr),
            0x8000..=0xFFFF => {
                // A 16 KB cart mirrors into both halves of the window.
                let offset = (addr - 0x8000) as usize % self.banks.prg.len().max(1);
                self.banks.prg.get(offset).copied().unwrap_or(0)
            }
            _ => 0,
        }
    }

    fn cpu_write(&mut self, addr: u16, val: u8) {
        if let 0x6000..=0x7FFF = addr {
            self.banks.prg_ram_write(addr, val);
        }
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        self.banks.chr(0, 0x2000, addr & 0x1FFF)
    }

    fn ppu_write(&mut self, addr: u16, val: u8) {
        self.banks.chr_write(0, 0x2000, addr & 0x1FFF, val);
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
