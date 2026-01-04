use super::types::AddressingMode;
use super::{CPU, Mem};

impl CPU {
    pub fn resolve_addr(&mut self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Relative => {
                let offset = self.fetch_byte() as i8;
                self.program_counter.wrapping_add(offset as u16)
            }
            AddressingMode::Implied | AddressingMode::Accumulator => 0,
            AddressingMode::Immediate => {
                let addr = self.program_counter;
                self.program_counter += 1;
                addr
            }
            AddressingMode::ZeroPage => self.fetch_byte() as u16,
            AddressingMode::ZeroPageX => self.fetch_byte().wrapping_add(self.register_x) as u16,
            AddressingMode::ZeroPageY => self.fetch_byte().wrapping_add(self.register_y) as u16,
            AddressingMode::Absolute => self.fetch_word(),
            AddressingMode::Indirect => {
                let ptr = self.fetch_word();
                if ptr & 0x00FF == 0x00FF {
                    let lo = self.mem_read(ptr) as u16;
                    let hi = self.mem_read(ptr & 0xFF00) as u16;
                    (hi << 8) | lo
                } else {
                    self.mem_read_u16(ptr)
                }
            }
            AddressingMode::AbsoluteX => self.fetch_word().wrapping_add(self.register_x as u16),
            AddressingMode::AbsoluteY => self.fetch_word().wrapping_add(self.register_y as u16),
            AddressingMode::IndirectX => {
                let base = self.fetch_byte();
                let ptr = base.wrapping_add(self.register_x);
                self.bus.mem_read_u16_zp(ptr)
            }
            AddressingMode::IndirectY => {
                let base = self.fetch_byte();
                let ptr = self.bus.mem_read_u16_zp(base);
                ptr.wrapping_add(self.register_y as u16)
            }
            _ => {
                eprintln!("WARNING: Addressmode not yet supported");
                0
            }
        }
    }
}
