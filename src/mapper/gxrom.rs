use super::{Banks, Mapper};
use crate::rom::{Mirroring, Rom};

/// Mapper 66 (GxROM): one register switches a 32 KB PRG bank and an 8 KB CHR
/// bank at the same time.
pub struct Gxrom {
    banks: Banks,
    mirroring: Mirroring,
    prg_bank: u8,
    chr_bank: u8,
}

impl Gxrom {
    pub fn new(rom: Rom) -> Self {
        let mirroring = rom.mirroring;
        Self { banks: Banks::new(&rom), mirroring, prg_bank: 0, chr_bank: 0 }
    }
}

impl Mapper for Gxrom {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.banks.prg_ram_read(addr),
            0x8000..=0xFFFF => self.banks.prg(self.prg_bank as isize, 0x8000, addr - 0x8000),
            _ => 0,
        }
    }

    fn cpu_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x6000..=0x7FFF => self.banks.prg_ram_write(addr, val),
            0x8000..=0xFFFF => {
                self.prg_bank = (val >> 4) & 0x03;
                self.chr_bank = val & 0x03;
            }
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
