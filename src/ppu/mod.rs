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
            vram: [0; 2000],
            palette_mem: [0; 32],
            v: 0,
            x: 0,
            t: 0,
            w: 0,
            cycles: 0,
        }
    }
    pub fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr & 0x007 {
            0 | 1 | 3 | 5 | 6 => 0,
            2 => {
                let result = (self.status & 0xE0) | (self.data & 0x1F);
                self.status &= 0x7F;
                self.w = false;
                result
            }
            4 => self.oam[self.oam_addr as usize],
            7 => {
                let addr = self.v;
                let result = if addr < 0x3F00 {
                    let buffered = self.data;
                    self.data = self.read(addr);
                    buffered
                } else {
                    self.read(addr)
                };
                self.v = self.v.wrapping_add(self.v_increment());
                result
            }
            _ => 0,
        }
    }

    pub fn cpu_write(&mut self, addr: u16) {
        match addr & 0x007 {
            0 => {}
        }
    }
}
