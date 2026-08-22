use super::CPU;
use super::types::AddressingMode;

impl CPU {
    pub fn resolve_addr(&mut self, mode: AddressingMode) -> (u16, bool) {
        match mode {
            AddressingMode::Implied | AddressingMode::Accumulator => (0, false),
            AddressingMode::Immediate => {
                let addr = self.program_counter;
                self.program_counter = self.program_counter.wrapping_add(1);
                (addr, false)
            }
            AddressingMode::Relative => {
                let offset = self.fetch_byte() as i8;
                let target = self.program_counter.wrapping_add(offset as u16);
                (target, page_crossed(self.program_counter, target))
            }
            AddressingMode::ZeroPage => (self.fetch_byte() as u16, false),
            AddressingMode::ZeroPageX => {
                (self.fetch_byte().wrapping_add(self.register_x) as u16, false)
            }
            AddressingMode::ZeroPageY => {
                (self.fetch_byte().wrapping_add(self.register_y) as u16, false)
            }
            AddressingMode::Absolute => (self.fetch_word(), false),
            AddressingMode::AbsoluteX => {
                let base = self.fetch_word();
                let addr = base.wrapping_add(self.register_x as u16);
                (addr, page_crossed(base, addr))
            }
            AddressingMode::AbsoluteY => {
                let base = self.fetch_word();
                let addr = base.wrapping_add(self.register_y as u16);
                (addr, page_crossed(base, addr))
            }
            AddressingMode::Indirect => {
                let ptr = self.fetch_word();
                (self.read_u16_bugged(ptr), false)
            }
            AddressingMode::IndirectX => {
                let base = self.fetch_byte();
                let ptr = base.wrapping_add(self.register_x);
                (self.bus.read_u16_zeropage(ptr), false)
            }
            AddressingMode::IndirectY => {
                let base = self.fetch_byte();
                let ptr = self.bus.read_u16_zeropage(base);
                let addr = ptr.wrapping_add(self.register_y as u16);
                (addr, page_crossed(ptr, addr))
            }
        }
    }

    pub fn read_u16_bugged(&mut self, ptr: u16) -> u16 {
        if ptr & 0x00FF == 0x00FF {
            let lo = self.bus.read(ptr) as u16;
            let hi = self.bus.read(ptr & 0xFF00) as u16;
            (hi << 8) | lo
        } else {
            self.bus.read_u16(ptr)
        }
    }
}

fn page_crossed(from: u16, to: u16) -> bool {
    from & 0xFF00 != to & 0xFF00
}
