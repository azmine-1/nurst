use crate::bus::Bus;

pub struct PPU {
    ctrl: u8,
    mask: u8,
    status: u8,
    oam_addr: u8,
    oam_data: u8,
    scroll: u8,
    addr: u8,
    data: u8,
    oam_dma: u8,
    oam: [u8; 256],
    vram: [u8; 2000],
    palette_mem: [u8; 32],
    v: u16,
    x: u8,
    t: u16,
    w: bool,
    cycles: u8,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            oam_data: 0,
            scroll: 0,
            addr: 0,
            data: 0,
            oam_dma: u8,
            io_db: Bus,
            vram: [u8; 2000],
            palette_mem: [u8; 32],
            v: u8,
            x: u8,
            t: u8,
            w: u8,
            cycles: u8,
        }
    }
    pub fn cpu_read(&self) -> u16 {
        if self.ctrl & 0x04 != 0 { 32 } else { 1 }
    }
    pub fn cpu_read(&self, addr: u16) -> u8 {
        match addr {
            0 | 1 | 3 | 5 | 6 => 0,
            2 => {
                let result = (self.status & 0xE0) | (self.data);
                self.status &= !0x80;
                self.w = false;
                result
            }
            4 => self.oam[self.oam_addr as usize],

            7 => {
                let addr = self.v;
                let result = if addr < 0x3F00 {
                    let buffered = self.data_buffer;
                    self.data = self.ppu_read(addr);
                    buffered
                } else {
                    self.data = self.ppu_read(addr - 0x1000);
                    self.ppu_read(addr)
                };
                self.v = self.v.wrapping_add(self.vram_increment());
            }
        }
    }
    pub fn cpu_write(&self, addr: u16, val: u8) {
        match addr {
            0 => {}
        }
    }
}
