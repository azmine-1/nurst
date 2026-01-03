use super::types::AddressingMode;
use super::{Mem, CPU};

impl CPU {
    pub fn resolve_addr(&mut self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Relative => {
                let offset = self.fetch_byte() as i8;
                self.program_counter.wrapping_add(offset as u16)
            }
            AddressingMode::Implied => 0,
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
                self.mem_read_u16(ptr)
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
                println!("Addressmode not yet supported");
                0
            }
        }
    }
}
