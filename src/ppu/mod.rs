//! The 2C02 picture processing unit, rendered one dot at a time.
//!
//! Timing follows the standard NTSC frame: 262 scanlines of 341 dots each.
//! Lines 0-239 are visible, 240 is idle, 241-260 are vertical blank (the NMI
//! fires on line 241 dot 1) and 261 is the pre-render line.

mod palette;

use crate::mapper::Mapper;
use crate::rom::Mirroring;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

const DOTS_PER_SCANLINE: u16 = 341;
const SCANLINES_PER_FRAME: u16 = 262;
const PRERENDER_LINE: u16 = 261;

/// One entry of secondary OAM, already paired with its fetched pattern data.
#[derive(Clone, Copy, Default)]
struct Sprite {
    x: u8,
    attributes: u8,
    pattern_lo: u8,
    pattern_hi: u8,
    /// True when this slot holds sprite 0 from primary OAM.
    is_sprite_zero: bool,
}

pub struct PPU {
    // Memory-mapped registers.
    ctrl: u8,
    mask: u8,
    status: u8,
    oam_addr: u8,

    // Internal "loopy" registers: current address, temporary address, fine X
    // scroll and the shared write latch used by $2005/$2006.
    v: u16,
    t: u16,
    x: u8,
    w: bool,
    read_buffer: u8,

    // Memory.
    vram: [u8; 4096],
    palette_mem: [u8; 32],
    oam: [u8; 256],

    // Background fetch pipeline.
    nametable_byte: u8,
    attribute_byte: u8,
    pattern_lo: u8,
    pattern_hi: u8,
    shift_pattern_lo: u16,
    shift_pattern_hi: u16,
    shift_attr_lo: u16,
    shift_attr_hi: u16,

    // Sprites for the scanline being drawn.
    sprites: [Sprite; 8],
    sprite_count: usize,

    // Timing.
    scanline: u16,
    dot: u16,
    odd_frame: bool,
    pub cycles: u64,

    /// Set for one CPU poll when the PPU raises /NMI.
    nmi_pending: bool,
    /// Tracks the NMI edge so setting PPUCTRL bit 7 mid-vblank re-triggers it.
    nmi_previous: bool,

    /// Last value on the PPU bus, returned by the write-only register reads.
    open_bus: u8,

    pub frame: Vec<u32>,
    pub frame_complete: bool,
    pub frame_count: u64,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            v: 0,
            t: 0,
            x: 0,
            w: false,
            read_buffer: 0,
            vram: [0; 4096],
            palette_mem: [
                0x09, 0x01, 0x00, 0x01, 0x00, 0x02, 0x02, 0x0D, 0x08, 0x10, 0x08, 0x24, 0x00,
                0x00, 0x04, 0x2C, 0x09, 0x01, 0x34, 0x03, 0x00, 0x04, 0x00, 0x14, 0x08, 0x3A,
                0x00, 0x02, 0x00, 0x20, 0x2C, 0x08,
            ],
            oam: [0; 256],
            nametable_byte: 0,
            attribute_byte: 0,
            pattern_lo: 0,
            pattern_hi: 0,
            shift_pattern_lo: 0,
            shift_pattern_hi: 0,
            shift_attr_lo: 0,
            shift_attr_hi: 0,
            sprites: [Sprite::default(); 8],
            sprite_count: 0,
            scanline: 0,
            dot: 0,
            odd_frame: false,
            cycles: 0,
            nmi_pending: false,
            nmi_previous: false,
            open_bus: 0,
            frame: vec![0; WIDTH * HEIGHT],
            frame_complete: false,
            frame_count: 0,
        }
    }

    pub fn reset(&mut self) {
        self.ctrl = 0;
        self.mask = 0;
        self.w = false;
        self.read_buffer = 0;
        self.scanline = 0;
        self.dot = 0;
        self.odd_frame = false;
    }

    /// Scanline and dot, for trace output that mirrors nestest's PPU column.
    pub fn position(&self) -> (u16, u16) {
        (self.scanline, self.dot)
    }

    pub fn take_nmi(&mut self) -> bool {
        std::mem::take(&mut self.nmi_pending)
    }

    fn rendering_enabled(&self) -> bool {
        self.mask & 0x18 != 0
    }

    fn sprite_height(&self) -> u16 {
        if self.ctrl & 0x20 != 0 { 16 } else { 8 }
    }

    fn vram_increment(&self) -> u16 {
        if self.ctrl & 0x04 != 0 { 32 } else { 1 }
    }

    // ---------------------------------------------------------------- CPU bus

    pub fn cpu_read(&mut self, addr: u16, mapper: &mut dyn Mapper) -> u8 {
        let value = match addr & 0x07 {
            2 => {
                // Only the top three bits are real; the rest is open bus.
                let result = (self.status & 0xE0) | (self.open_bus & 0x1F);
                self.status &= 0x7F;
                self.w = false;
                self.nmi_previous = false;
                result
            }
            4 => self.oam[self.oam_addr as usize],
            7 => {
                let addr = self.v & 0x3FFF;
                mapper.a12_clock(addr, self.cycles);
                let result = if addr < 0x3F00 {
                    // Reads below the palettes are delayed by one fetch.
                    let fetched = self.bus_read(addr, mapper);
                    std::mem::replace(&mut self.read_buffer, fetched)
                } else {
                    // Palette reads are immediate, but still refill the buffer
                    // from the nametable underneath.
                    self.read_buffer = self.bus_read(addr - 0x1000, mapper);
                    self.read_palette(addr)
                };
                self.v = self.v.wrapping_add(self.vram_increment()) & 0x7FFF;
                result
            }
            _ => self.open_bus,
        };
        self.open_bus = value;
        value
    }

    /// Read a nametable byte without touching the PPU's state, for tests that
    /// need to see what a ROM printed on screen.
    pub fn peek_vram(&self, addr: u16, mirroring: Mirroring) -> u8 {
        self.vram[self.mirrored_vram(addr & 0x2FFF, mirroring)]
    }

    /// Register read with no side effects, for the debugger and trace output.
    pub fn peek(&self, addr: u16) -> u8 {
        match addr & 0x07 {
            2 => (self.status & 0xE0) | (self.open_bus & 0x1F),
            4 => self.oam[self.oam_addr as usize],
            _ => self.open_bus,
        }
    }

    pub fn cpu_write(&mut self, addr: u16, val: u8, mapper: &mut dyn Mapper) {
        self.open_bus = val;
        match addr & 0x07 {
            0 => {
                self.ctrl = val;
                self.t = (self.t & 0xF3FF) | ((val as u16 & 0x03) << 10);
                self.update_nmi();
            }
            1 => self.mask = val,
            3 => self.oam_addr = val,
            4 => {
                self.oam[self.oam_addr as usize] = val;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            5 => {
                if !self.w {
                    self.t = (self.t & 0xFFE0) | (val as u16 >> 3);
                    self.x = val & 0x07;
                } else {
                    self.t = (self.t & 0x8FFF) | ((val as u16 & 0x07) << 12);
                    self.t = (self.t & 0xFC1F) | ((val as u16 & 0xF8) << 2);
                }
                self.w = !self.w;
            }
            6 => {
                if !self.w {
                    self.t = (self.t & 0x00FF) | ((val as u16 & 0x3F) << 8);
                } else {
                    self.t = (self.t & 0xFF00) | val as u16;
                    self.v = self.t;
                    // Pointing the address bus at $1xxx toggles A12, which is
                    // how MMC3's scanline counter is clocked outside rendering.
                    mapper.a12_clock(self.v & 0x3FFF, self.cycles);
                }
                self.w = !self.w;
            }
            7 => {
                let addr = self.v & 0x3FFF;
                mapper.a12_clock(addr, self.cycles);
                self.bus_write(addr, val, mapper);
                self.v = self.v.wrapping_add(self.vram_increment()) & 0x7FFF;
            }
            _ => {}
        }
    }

    /// One byte of an OAM DMA transfer.
    pub fn write_oam_dma(&mut self, val: u8) {
        self.oam[self.oam_addr as usize] = val;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    fn update_nmi(&mut self) {
        let nmi = self.ctrl & 0x80 != 0 && self.status & 0x80 != 0;
        if nmi && !self.nmi_previous {
            self.nmi_pending = true;
        }
        self.nmi_previous = nmi;
    }

    // ---------------------------------------------------------------- PPU bus

    fn mirrored_vram(&self, addr: u16, mirroring: Mirroring) -> usize {
        let addr = (addr - 0x2000) & 0x0FFF;
        let table = (addr / 0x0400) as usize;
        let offset = (addr % 0x0400) as usize;
        let physical = match mirroring {
            Mirroring::Horizontal => [0, 0, 1, 1][table],
            Mirroring::Vertical => [0, 1, 0, 1][table],
            Mirroring::SingleScreenLower => 0,
            Mirroring::SingleScreenUpper => 1,
            Mirroring::FourScreen => table,
        };
        physical * 0x0400 + offset
    }

    fn bus_read(&mut self, addr: u16, mapper: &mut dyn Mapper) -> u8 {
        let addr = addr & 0x3FFF;
        match addr {
            0x0000..=0x1FFF => {
                mapper.a12_clock(addr, self.cycles);
                mapper.ppu_read(addr)
            }
            0x2000..=0x3EFF => self.vram[self.mirrored_vram(addr & 0x2FFF, mapper.mirroring())],
            _ => self.read_palette(addr),
        }
    }

    fn bus_write(&mut self, addr: u16, val: u8, mapper: &mut dyn Mapper) {
        let addr = addr & 0x3FFF;
        match addr {
            0x0000..=0x1FFF => mapper.ppu_write(addr, val),
            0x2000..=0x3EFF => {
                let index = self.mirrored_vram(addr & 0x2FFF, mapper.mirroring());
                self.vram[index] = val;
            }
            _ => {
                let index = palette_index(addr);
                self.palette_mem[index] = val;
            }
        }
    }

    fn read_palette(&self, addr: u16) -> u8 {
        self.palette_mem[palette_index(addr)]
    }

    // ------------------------------------------------------------ scroll regs

    fn increment_coarse_x(&mut self) {
        if self.v & 0x001F == 31 {
            self.v &= !0x001F;
            self.v ^= 0x0400; // step to the horizontally adjacent nametable
        } else {
            self.v += 1;
        }
    }

    fn increment_y(&mut self) {
        if self.v & 0x7000 != 0x7000 {
            self.v += 0x1000; // fine Y
        } else {
            self.v &= !0x7000;
            let mut coarse_y = (self.v & 0x03E0) >> 5;
            if coarse_y == 29 {
                coarse_y = 0;
                self.v ^= 0x0800; // step to the vertically adjacent nametable
            } else if coarse_y == 31 {
                coarse_y = 0; // reading out of the attribute table: no wrap
            } else {
                coarse_y += 1;
            }
            self.v = (self.v & !0x03E0) | (coarse_y << 5);
        }
    }

    fn copy_horizontal(&mut self) {
        self.v = (self.v & !0x041F) | (self.t & 0x041F);
    }

    fn copy_vertical(&mut self) {
        self.v = (self.v & !0x7BE0) | (self.t & 0x7BE0);
    }

    // ------------------------------------------------------------- background

    fn load_shift_registers(&mut self) {
        self.shift_pattern_lo = (self.shift_pattern_lo & 0xFF00) | self.pattern_lo as u16;
        self.shift_pattern_hi = (self.shift_pattern_hi & 0xFF00) | self.pattern_hi as u16;
        // The two attribute bits are smeared across all eight pixels.
        let lo = if self.attribute_byte & 0x01 != 0 { 0xFF } else { 0x00 };
        let hi = if self.attribute_byte & 0x02 != 0 { 0xFF } else { 0x00 };
        self.shift_attr_lo = (self.shift_attr_lo & 0xFF00) | lo;
        self.shift_attr_hi = (self.shift_attr_hi & 0xFF00) | hi;
    }

    fn shift_background(&mut self) {
        self.shift_pattern_lo <<= 1;
        self.shift_pattern_hi <<= 1;
        self.shift_attr_lo <<= 1;
        self.shift_attr_hi <<= 1;
    }

    fn fetch_background(&mut self, mapper: &mut dyn Mapper) {
        match self.dot % 8 {
            1 => {
                self.load_shift_registers();
                let addr = 0x2000 | (self.v & 0x0FFF);
                self.nametable_byte = self.bus_read(addr, mapper);
            }
            3 => {
                let addr = 0x23C0
                    | (self.v & 0x0C00)
                    | ((self.v >> 4) & 0x38)
                    | ((self.v >> 2) & 0x07);
                let byte = self.bus_read(addr, mapper);
                // Pick the 2-bit quadrant this tile falls in.
                let shift = ((self.v >> 4) & 0x04) | (self.v & 0x02);
                self.attribute_byte = (byte >> shift) & 0x03;
            }
            5 => {
                let addr = self.background_pattern_addr();
                self.pattern_lo = self.bus_read(addr, mapper);
            }
            7 => {
                let addr = self.background_pattern_addr() + 8;
                self.pattern_hi = self.bus_read(addr, mapper);
            }
            0 => self.increment_coarse_x(),
            _ => {}
        }
    }

    fn background_pattern_addr(&self) -> u16 {
        let table = if self.ctrl & 0x10 != 0 { 0x1000 } else { 0x0000 };
        let fine_y = (self.v >> 12) & 0x07;
        table + (self.nametable_byte as u16) * 16 + fine_y
    }

    /// Background pixel as (2-bit colour, 2-bit palette).
    fn background_pixel(&self) -> (u8, u8) {
        if self.mask & 0x08 == 0 {
            return (0, 0);
        }
        let bit = 0x8000 >> self.x;
        let lo = (self.shift_pattern_lo & bit != 0) as u8;
        let hi = (self.shift_pattern_hi & bit != 0) as u8;
        let color = (hi << 1) | lo;
        if color == 0 {
            return (0, 0);
        }
        let attr_lo = (self.shift_attr_lo & bit != 0) as u8;
        let attr_hi = (self.shift_attr_hi & bit != 0) as u8;
        (color, (attr_hi << 1) | attr_lo)
    }

    // ---------------------------------------------------------------- sprites

    /// Scan primary OAM for the sprites visible on `scanline` and fetch their
    /// pattern data. Hardware spreads this across dots 65-320; doing it in one
    /// go at dot 257 is indistinguishable to software that is not counting
    /// cycles inside a scanline.
    fn evaluate_sprites(&mut self, scanline: u16, mapper: &mut dyn Mapper) {
        self.sprite_count = 0;
        self.sprites = [Sprite::default(); 8];
        let height = self.sprite_height();

        for index in 0..64 {
            let entry = index * 4;
            let y = self.oam[entry] as u16;
            if scanline < y || scanline >= y + height {
                continue;
            }
            if self.sprite_count == 8 {
                self.status |= 0x20; // sprite overflow
                break;
            }

            let tile = self.oam[entry + 1];
            let attributes = self.oam[entry + 2];
            let flip_vertical = attributes & 0x80 != 0;
            let mut row = scanline - y;
            if flip_vertical {
                row = height - 1 - row;
            }

            let addr = if height == 16 {
                // 8x16 sprites take their pattern table from the tile's low bit
                // and use the next tile for the bottom half.
                let table = (tile as u16 & 0x01) * 0x1000;
                let tile = (tile & 0xFE) as u16 + if row >= 8 { 1 } else { 0 };
                table + tile * 16 + (row & 0x07)
            } else {
                let table = if self.ctrl & 0x08 != 0 { 0x1000 } else { 0x0000 };
                table + tile as u16 * 16 + row
            };

            let mut pattern_lo = self.bus_read(addr, mapper);
            let mut pattern_hi = self.bus_read(addr + 8, mapper);
            if attributes & 0x40 != 0 {
                pattern_lo = pattern_lo.reverse_bits();
                pattern_hi = pattern_hi.reverse_bits();
            }

            self.sprites[self.sprite_count] = Sprite {
                x: self.oam[entry + 3],
                attributes,
                pattern_lo,
                pattern_hi,
                is_sprite_zero: index == 0,
            };
            self.sprite_count += 1;
        }
    }

    /// Sprite pixel at screen X as (colour, palette, behind_background, is_zero).
    fn sprite_pixel(&self, screen_x: u8) -> Option<(u8, u8, bool, bool)> {
        if self.mask & 0x10 == 0 {
            return None;
        }
        for sprite in &self.sprites[..self.sprite_count] {
            let offset = screen_x.wrapping_sub(sprite.x);
            if sprite.x > screen_x || offset >= 8 {
                continue;
            }
            let bit = 0x80 >> offset;
            let lo = (sprite.pattern_lo & bit != 0) as u8;
            let hi = (sprite.pattern_hi & bit != 0) as u8;
            let color = (hi << 1) | lo;
            if color == 0 {
                continue; // transparent: the next sprite in priority order wins
            }
            return Some((
                color,
                sprite.attributes & 0x03,
                sprite.attributes & 0x20 != 0,
                sprite.is_sprite_zero,
            ));
        }
        None
    }

    // ----------------------------------------------------------------- render

    fn render_pixel(&mut self) {
        let screen_x = (self.dot - 1) as usize;
        let screen_y = self.scanline as usize;

        let (mut bg_color, mut bg_palette) = self.background_pixel();
        if screen_x < 8 && self.mask & 0x02 == 0 {
            bg_color = 0;
            bg_palette = 0;
        }

        let sprite = match self.sprite_pixel(screen_x as u8) {
            Some(_) if screen_x < 8 && self.mask & 0x04 == 0 => None,
            other => other,
        };

        let mut palette_addr = 0x3F00u16;
        if let Some((sp_color, sp_palette, behind, is_zero)) = sprite {
            if is_zero && bg_color != 0 && screen_x != 255 && self.mask & 0x18 == 0x18 {
                self.status |= 0x40; // sprite 0 hit
            }
            if bg_color == 0 || !behind {
                palette_addr = 0x3F10 + (sp_palette as u16) * 4 + sp_color as u16;
            } else {
                palette_addr = 0x3F00 + (bg_palette as u16) * 4 + bg_color as u16;
            }
        } else if bg_color != 0 {
            palette_addr = 0x3F00 + (bg_palette as u16) * 4 + bg_color as u16;
        }

        let color_index = self.read_palette(palette_addr);
        self.frame[screen_y * WIDTH + screen_x] = palette::apply_mask(color_index, self.mask);
    }

    /// Advance the PPU by one dot.
    pub fn tick(&mut self, mapper: &mut dyn Mapper) {
        self.cycles += 1;

        let visible = self.scanline < 240;
        let prerender = self.scanline == PRERENDER_LINE;
        let rendering = self.rendering_enabled();

        if (visible || prerender) && rendering {
            // Dots 1-256 draw and fetch; 321-336 prefetch the next line.
            if (self.dot >= 1 && self.dot <= 256) || (self.dot >= 321 && self.dot <= 336) {
                if visible && self.dot <= 256 {
                    self.render_pixel();
                }
                self.shift_background();
                self.fetch_background(mapper);
            }

            match self.dot {
                256 => self.increment_y(),
                257 => {
                    self.copy_horizontal();
                    let line = if prerender { 0 } else { self.scanline + 1 };
                    if line < 240 {
                        self.evaluate_sprites(line, mapper);
                    } else {
                        self.sprite_count = 0;
                    }
                }
                280..=304 if prerender => self.copy_vertical(),
                // Two unused nametable fetches; MMC3 needs the A12 activity.
                338 | 340 => {
                    let addr = 0x2000 | (self.v & 0x0FFF);
                    self.bus_read(addr, mapper);
                }
                _ => {}
            }
        } else if visible && self.dot >= 1 && self.dot <= 256 {
            // Rendering off: the screen shows the current palette entry, which
            // is $3F00 unless the VRAM address happens to point into palette
            // space ("background colour hack").
            let addr = if self.v & 0x3F00 == 0x3F00 { self.v } else { 0x3F00 };
            let color_index = self.read_palette(addr);
            let index = self.scanline as usize * WIDTH + (self.dot - 1) as usize;
            self.frame[index] = palette::apply_mask(color_index, self.mask);
        }

        if self.scanline == 241 && self.dot == 1 {
            self.status |= 0x80;
            self.update_nmi();
            self.frame_complete = true;
            self.frame_count += 1;
        }

        if prerender && self.dot == 1 {
            self.status &= !0xE0; // clear vblank, sprite 0 hit and overflow
            self.nmi_previous = false;
        }

        self.dot += 1;
        if self.dot >= DOTS_PER_SCANLINE {
            self.dot = 0;
            self.scanline += 1;
            if self.scanline >= SCANLINES_PER_FRAME {
                self.scanline = 0;
                self.odd_frame = !self.odd_frame;
            }
        }

        // On odd frames with rendering on, the pre-render line is one dot short.
        if prerender && self.dot == 340 && self.odd_frame && rendering {
            self.dot = 0;
            self.scanline = 0;
            self.odd_frame = !self.odd_frame;
        }
    }
}

/// Map a palette address to its storage slot, folding the sprite-background
/// mirrors at $3F10/$3F14/$3F18/$3F1C onto the universal background colours.
fn palette_index(addr: u16) -> usize {
    let index = (addr & 0x1F) as usize;
    if index >= 16 && index % 4 == 0 { index - 16 } else { index }
}
