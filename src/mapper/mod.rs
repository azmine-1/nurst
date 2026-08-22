//! Cartridge mappers.
//!
//! A mapper owns everything the cartridge board provides: PRG ROM/RAM visible to
//! the CPU at $4020-$FFFF, CHR ROM/RAM visible to the PPU at $0000-$1FFF, the
//! nametable mirroring the board wires up, and (for MMC3) a scanline IRQ counter.

mod axrom;
mod cnrom;
mod gxrom;
mod mmc1;
mod mmc3;
mod nrom;
mod uxrom;

use crate::rom::{Mirroring, Rom};

pub trait Mapper {
    fn cpu_read(&mut self, addr: u16) -> u8;
    fn cpu_write(&mut self, addr: u16, val: u8);
    fn ppu_read(&mut self, addr: u16) -> u8;
    fn ppu_write(&mut self, addr: u16, val: u8);
    fn mirroring(&self) -> Mirroring;

    /// True while the board is asserting /IRQ.
    fn irq_pending(&self) -> bool {
        false
    }

    /// Called with every PPU pattern-table fetch address so boards that watch
    /// the A12 line (MMC3) can drive their scanline counter.
    fn a12_clock(&mut self, _addr: u16, _ppu_cycle: u64) {}
}

pub fn from_rom(rom: Rom) -> Result<Box<dyn Mapper>, String> {
    Ok(match rom.mapper {
        0 => Box::new(nrom::Nrom::new(rom)),
        1 => Box::new(mmc1::Mmc1::new(rom)),
        2 => Box::new(uxrom::Uxrom::new(rom)),
        3 => Box::new(cnrom::Cnrom::new(rom)),
        4 => Box::new(mmc3::Mmc3::new(rom)),
        7 => Box::new(axrom::Axrom::new(rom)),
        66 => Box::new(gxrom::Gxrom::new(rom)),
        n => return Err(format!("mapper {} is not supported", n)),
    })
}

/// PRG/CHR storage plus the bank-window arithmetic every mapper needs.
pub struct Banks {
    pub prg: Vec<u8>,
    pub chr: Vec<u8>,
    pub prg_ram: Vec<u8>,
    pub chr_writable: bool,
}

impl Banks {
    fn new(rom: &Rom) -> Self {
        Self {
            prg: rom.prg_rom.clone(),
            chr: rom.chr_rom.clone(),
            prg_ram: vec![0; 8192],
            chr_writable: rom.chr_ram,
        }
    }

    /// Read `offset` bytes into the `size`-byte PRG bank numbered `bank`.
    /// Negative bank numbers count back from the end of the ROM.
    fn prg(&self, bank: isize, size: usize, offset: u16) -> u8 {
        let count = (self.prg.len() / size).max(1) as isize;
        let bank = bank.rem_euclid(count) as usize;
        let idx = bank * size + offset as usize;
        self.prg.get(idx % self.prg.len().max(1)).copied().unwrap_or(0)
    }

    fn chr(&self, bank: usize, size: usize, offset: u16) -> u8 {
        if self.chr.is_empty() {
            return 0;
        }
        let count = (self.chr.len() / size).max(1);
        let idx = (bank % count) * size + offset as usize;
        self.chr[idx % self.chr.len()]
    }

    fn chr_write(&mut self, bank: usize, size: usize, offset: u16, val: u8) {
        if !self.chr_writable || self.chr.is_empty() {
            return;
        }
        let count = (self.chr.len() / size).max(1);
        let idx = (bank % count) * size + offset as usize;
        let len = self.chr.len();
        self.chr[idx % len] = val;
    }

    fn prg_ram_read(&self, addr: u16) -> u8 {
        self.prg_ram[(addr as usize - 0x6000) % self.prg_ram.len()]
    }

    fn prg_ram_write(&mut self, addr: u16, val: u8) {
        let len = self.prg_ram.len();
        self.prg_ram[(addr as usize - 0x6000) % len] = val;
    }
}
