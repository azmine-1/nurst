use super::{Banks, Mapper};
use crate::rom::{Mirroring, Rom};

const A12_FILTER: u64 = 10;

pub struct Mmc3 {
    banks: Banks,
    bank_select: u8,
    bank_regs: [u8; 8],
    mirroring: Mirroring,
    four_screen: bool,

    irq_latch: u8,
    irq_counter: u8,
    irq_enabled: bool,
    irq_reload: bool,
    irq_pending: bool,

    a12_high: bool,
    a12_low_since: u64,
}

impl Mmc3 {
    pub fn new(rom: Rom) -> Self {
        Self {
            four_screen: rom.mirroring == Mirroring::FourScreen,
            mirroring: rom.mirroring,
            banks: Banks::new(&rom),
            bank_select: 0,
            bank_regs: [0, 2, 4, 5, 6, 7, 0, 1],
            irq_latch: 0,
            irq_counter: 0,
            irq_enabled: false,
            irq_reload: false,
            irq_pending: false,
            a12_high: false,
            a12_low_since: 0,
        }
    }

    fn prg_mode_swapped(&self) -> bool {
        self.bank_select & 0x40 != 0
    }

    fn chr_mode_swapped(&self) -> bool {
        self.bank_select & 0x80 != 0
    }

    fn chr_bank_for(&self, addr: u16) -> usize {
        let slot = (addr >> 10) & 0x07;
        let slot = if self.chr_mode_swapped() { slot ^ 0x04 } else { slot };
        let bank = match slot {
            0 | 1 => (self.bank_regs[0] & 0xFE) as usize + slot as usize,
            2 | 3 => (self.bank_regs[1] & 0xFE) as usize + (slot as usize - 2),
            n => self.bank_regs[(n - 2) as usize] as usize,
        };
        bank
    }

    fn clock_irq(&mut self) {
        if self.irq_counter == 0 || self.irq_reload {
            self.irq_counter = self.irq_latch;
            self.irq_reload = false;
        } else {
            self.irq_counter -= 1;
        }
        if self.irq_counter == 0 && self.irq_enabled {
            self.irq_pending = true;
        }
    }
}

impl Mapper for Mmc3 {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.banks.prg_ram_read(addr),
            0x8000..=0x9FFF => {
                let bank =
                    if self.prg_mode_swapped() { -2 } else { self.bank_regs[6] as isize };
                self.banks.prg(bank, 0x2000, addr - 0x8000)
            }
            0xA000..=0xBFFF => self.banks.prg(self.bank_regs[7] as isize, 0x2000, addr - 0xA000),
            0xC000..=0xDFFF => {
                let bank =
                    if self.prg_mode_swapped() { self.bank_regs[6] as isize } else { -2 };
                self.banks.prg(bank, 0x2000, addr - 0xC000)
            }
            0xE000..=0xFFFF => self.banks.prg(-1, 0x2000, addr - 0xE000),
            _ => 0,
        }
    }

    fn cpu_write(&mut self, addr: u16, val: u8) {
        let odd = addr & 1 != 0;
        match addr {
            0x6000..=0x7FFF => self.banks.prg_ram_write(addr, val),
            0x8000..=0x9FFF if !odd => self.bank_select = val,
            0x8000..=0x9FFF => {
                let idx = (self.bank_select & 0x07) as usize;
                self.bank_regs[idx] = val;
            }
            0xA000..=0xBFFF if !odd => {
                if !self.four_screen {
                    self.mirroring = if val & 1 == 0 {
                        Mirroring::Vertical
                    } else {
                        Mirroring::Horizontal
                    };
                }
            }
            0xA000..=0xBFFF => {} // PRG RAM protect: not emulated
            0xC000..=0xDFFF if !odd => self.irq_latch = val,
            0xC000..=0xDFFF => {
                self.irq_counter = 0;
                self.irq_reload = true;
            }
            0xE000..=0xFFFF if !odd => {
                self.irq_enabled = false;
                self.irq_pending = false;
            }
            0xE000..=0xFFFF => self.irq_enabled = true,
            _ => {}
        }
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        let bank = self.chr_bank_for(addr);
        self.banks.chr(bank, 0x0400, addr & 0x03FF)
    }

    fn ppu_write(&mut self, addr: u16, val: u8) {
        let bank = self.chr_bank_for(addr);
        self.banks.chr_write(bank, 0x0400, addr & 0x03FF, val);
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn a12_clock(&mut self, addr: u16, ppu_cycle: u64) {
        let high = addr & 0x1000 != 0;
        if high && !self.a12_high && ppu_cycle.saturating_sub(self.a12_low_since) >= A12_FILTER {
            self.clock_irq();
        }
        if !high && self.a12_high {
            self.a12_low_since = ppu_cycle;
        }
        self.a12_high = high;
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }
}
