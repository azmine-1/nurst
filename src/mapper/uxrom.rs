use super::{Banks, Mapper};
use crate::rom::{Mirroring, Rom};

/// Mapper 2: a switchable 16 KB PRG bank at $8000, the last bank fixed at $C000.
pub struct Uxrom {
    banks: Banks,
    mirroring: Mirroring,
    bank: u8,
}

impl Uxrom {
    pub fn new(rom: Rom) -> Self {
        let mirroring = rom.mirroring;
        Self { banks: Banks::new(&rom), mirroring, bank: 0 }
    }
}

impl Mapper for Uxrom {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.banks.prg_ram_read(addr),
            0x8000..=0xBFFF => self.banks.prg(self.bank as isize, 0x4000, addr - 0x8000),
            0xC000..=0xFFFF => self.banks.prg(-1, 0x4000, addr - 0xC000),
            _ => 0,
        }
    }

    fn cpu_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x6000..=0x7FFF => self.banks.prg_ram_write(addr, val),
            // UOROM boards carry up to 512 KB, so all the low bits matter;
            // Banks::prg folds the value into the range the cart actually has.
            0x8000..=0xFFFF => self.bank = val,
            _ => {}
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
