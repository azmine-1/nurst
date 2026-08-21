use super::{Banks, Mapper};
use crate::rom::{Mirroring, Rom};

pub struct Axrom {
    banks: Banks,
    bank: u8,
    upper_nametable: bool,
}

impl Axrom {
    pub fn new(rom: Rom) -> Self {
        Self { banks: Banks::new(&rom), bank: 0, upper_nametable: false }
    }
}

impl Mapper for Axrom {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.banks.prg_ram_read(addr),
            0x8000..=0xFFFF => self.banks.prg(self.bank as isize, 0x8000, addr - 0x8000),
            _ => 0,
        }
    }

    fn cpu_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x6000..=0x7FFF => self.banks.prg_ram_write(addr, val),
            0x8000..=0xFFFF => {
                self.bank = val & 0x07;
                self.upper_nametable = val & 0x10 != 0;
            }
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
        if self.upper_nametable {
            Mirroring::SingleScreenUpper
        } else {
            Mirroring::SingleScreenLower
        }
    }
}
