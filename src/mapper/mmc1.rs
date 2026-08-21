use super::{Banks, Mapper};
use crate::rom::{Mirroring, Rom};

/// Mapper 1 (MMC1). Registers are loaded one bit at a time through a serial
/// shift register: five writes with bit 7 clear commit a value, and any write
/// with bit 7 set resets the shifter and forces PRG mode 3.
pub struct Mmc1 {
    banks: Banks,
    shift: u8,
    shift_count: u8,
    control: u8,
    chr_bank0: u8,
    chr_bank1: u8,
    prg_bank: u8,
}

impl Mmc1 {
    pub fn new(rom: Rom) -> Self {
        Self {
            banks: Banks::new(&rom),
            shift: 0,
            shift_count: 0,
            control: 0x0C, // PRG mode 3: last bank fixed at $C000
            chr_bank0: 0,
            chr_bank1: 0,
            prg_bank: 0,
        }
    }

    fn write_register(&mut self, addr: u16, val: u8) {
        match addr {
            0x8000..=0x9FFF => self.control = val & 0x1F,
            0xA000..=0xBFFF => self.chr_bank0 = val & 0x1F,
            0xC000..=0xDFFF => self.chr_bank1 = val & 0x1F,
            _ => self.prg_bank = val & 0x0F,
        }
    }

    fn prg_mode(&self) -> u8 {
        (self.control >> 2) & 0x03
    }

    fn chr_4k_mode(&self) -> bool {
        self.control & 0x10 != 0
    }
}

impl Mapper for Mmc1 {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.banks.prg_ram_read(addr),
            0x8000..=0xFFFF => {
                let bank = self.prg_bank as isize;
                match (self.prg_mode(), addr) {
                    // Modes 0 and 1 switch 32 KB at a time.
                    (0 | 1, _) => self.banks.prg(bank >> 1, 0x8000, addr - 0x8000),
                    // Mode 2 fixes the first bank at $8000.
                    (2, 0x8000..=0xBFFF) => self.banks.prg(0, 0x4000, addr - 0x8000),
                    (2, _) => self.banks.prg(bank, 0x4000, addr - 0xC000),
                    // Mode 3 fixes the last bank at $C000.
                    (_, 0x8000..=0xBFFF) => self.banks.prg(bank, 0x4000, addr - 0x8000),
                    (_, _) => self.banks.prg(-1, 0x4000, addr - 0xC000),
                }
            }
            _ => 0,
        }
    }

    fn cpu_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x6000..=0x7FFF => self.banks.prg_ram_write(addr, val),
            0x8000..=0xFFFF => {
                if val & 0x80 != 0 {
                    self.shift = 0;
                    self.shift_count = 0;
                    self.control |= 0x0C;
                    return;
                }
                self.shift = (self.shift >> 1) | ((val & 1) << 4);
                self.shift_count += 1;
                if self.shift_count == 5 {
                    let value = self.shift;
                    self.shift = 0;
                    self.shift_count = 0;
                    self.write_register(addr, value);
                }
            }
            _ => {}
        }
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        let addr = addr & 0x1FFF;
        if self.chr_4k_mode() {
            let bank = if addr < 0x1000 { self.chr_bank0 } else { self.chr_bank1 };
            self.banks.chr(bank as usize, 0x1000, addr & 0x0FFF)
        } else {
            self.banks.chr((self.chr_bank0 >> 1) as usize, 0x2000, addr)
        }
    }

    fn ppu_write(&mut self, addr: u16, val: u8) {
        let addr = addr & 0x1FFF;
        if self.chr_4k_mode() {
            let bank = if addr < 0x1000 { self.chr_bank0 } else { self.chr_bank1 };
            self.banks.chr_write(bank as usize, 0x1000, addr & 0x0FFF, val);
        } else {
            self.banks.chr_write((self.chr_bank0 >> 1) as usize, 0x2000, addr, val);
        }
    }

    fn mirroring(&self) -> Mirroring {
        match self.control & 0x03 {
            0 => Mirroring::SingleScreenLower,
            1 => Mirroring::SingleScreenUpper,
            2 => Mirroring::Vertical,
            _ => Mirroring::Horizontal,
        }
    }
}
