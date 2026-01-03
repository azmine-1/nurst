use super::types::{AddressingMode, Flags, Instruction, Opcode};
use super::{Mem, CPU};

impl CPU {
    pub fn adc(&mut self, val: u8, acc: u8) -> u8 {
        let carry = if self.get_flag(Flags::C) { 1 } else { 0 };
        let sum = acc as u16 + val as u16 + carry as u16;
        self.set_carry(sum);
        let result = sum as u8;
        self.set_overflow(val, acc, result);
        self.set_zn(result);
        result
    }

    pub fn sbc(&mut self, acc: u8, mem: u8) -> u8 {
        let carry = if self.get_flag(Flags::C) { 0 } else { 1 };
        let sub = acc as i16 - mem as i16 - carry as i16;
        let overflow: i16 = (sub ^ acc as i16) & (sub ^ !(mem as i16)) & 0x80;
        self.set_flag(Flags::V, overflow != 0);
        self.set_flag(Flags::C, sub >= 0);
        let result = sub as u8;
        self.set_zn(result);
        result
    }

    pub fn execute(&mut self, instruction: Instruction) {
        let addr = self.resolve_addr(&instruction.addressing_mode);
        let opcode_copy = instruction.opcode;
        match instruction.opcode {
            Opcode::LDA => {
                self.accumulator = self.mem_read(addr);
                self.set_zn(self.accumulator);
            }
            Opcode::LDX => {
                self.register_x = self.mem_read(addr);
                self.set_zn(self.register_x);
            }
            Opcode::LDY => {
                self.register_y = self.mem_read(addr);
                self.set_zn(self.register_y);
            }
            Opcode::STA => self.mem_write(addr, self.accumulator),
            Opcode::STX => self.mem_write(addr, self.register_x),
            Opcode::STY => self.mem_write(addr, self.register_y),
            Opcode::TAX => {
                self.register_x = self.accumulator;
                self.set_zn(self.register_x);
            }
            Opcode::TAY => {
                self.register_y = self.accumulator;
                self.set_zn(self.register_y);
            }
            Opcode::TSX => {
                self.register_x = self.stack_pointer;
                self.set_zn(self.register_x);
            }
            Opcode::TXS => self.stack_pointer = self.register_x,
            Opcode::TXA => {
                self.accumulator = self.register_x;
                self.set_zn(self.accumulator);
            }
            Opcode::TYA => {
                self.accumulator = self.register_y;
                self.set_zn(self.accumulator);
            }
            Opcode::ADC => self.accumulator = self.adc(self.mem_read(addr), self.accumulator),
            Opcode::SBC => self.accumulator = self.sbc(self.accumulator, self.mem_read(addr)),
            Opcode::INC => {
                let value = self.mem_read(addr);
                let res = value.wrapping_add(1);
                self.mem_write(addr, res);
                self.set_zn(res);
            }
            Opcode::INX => {
                let res = self.register_x.wrapping_add(1);
                self.register_x = res;
                self.set_zn(res);
            }
            Opcode::INY => {
                let res = self.register_y.wrapping_add(1);
                self.register_y = res;
                self.set_zn(res);
            }
            Opcode::DEC => {
                let value = self.mem_read(addr);
                let res = value.wrapping_sub(1);
                self.mem_write(addr, res);
                self.set_zn(res);
            }
            Opcode::DEX => {
                let res = self.register_x.wrapping_sub(1);
                self.register_x = res;
                self.set_zn(res);
            }
            Opcode::DEY => {
                let res = self.register_y.wrapping_sub(1);
                self.register_y = res;
                self.set_zn(res);
            }
            Opcode::ASL => {
                if instruction.addressing_mode == AddressingMode::Accumulator {
                    self.set_flag(Flags::C, (self.accumulator & 0x80) != 0);
                    self.accumulator = self.accumulator << 1;
                    self.set_zn(self.accumulator);
                } else {
                    let val = self.mem_read(addr);
                    self.set_flag(Flags::C, (val & 0x80) != 0);
                    let result = val << 1;
                    self.mem_write(addr, result);
                    self.set_zn(result);
                }
            }
            Opcode::LSR => {
                if instruction.addressing_mode == AddressingMode::Accumulator {
                    self.set_flag(Flags::C, (self.accumulator & 0x01) != 0);
                    self.accumulator = self.accumulator >> 1;
                    self.set_zn(self.accumulator);
                } else {
                    let val = self.mem_read(addr);
                    self.set_flag(Flags::C, (val & 0x01) != 0);
                    let result = val >> 1;
                    self.mem_write(addr, result);
                    self.set_zn(result);
                }
            }
            Opcode::ROL => {
                if instruction.addressing_mode == AddressingMode::Accumulator {
                    let carry_flag = if self.get_flag(Flags::C) { 1 } else { 0 };
                    self.set_flag(Flags::C, (self.accumulator & 0x80) != 0);
                    self.accumulator = self.accumulator << 1 | carry_flag;
                    self.set_zn(self.accumulator);
                } else {
                    let carry_flag = if self.get_flag(Flags::C) { 1 } else { 0 };
                    let value = self.mem_read(addr);
                    self.set_flag(Flags::C, (value & 0x80) != 0);
                    let result = value << 1 | carry_flag;
                    self.mem_write(addr, result);
                    self.set_zn(result);
                }
            }
            Opcode::ROR => {
                if instruction.addressing_mode == AddressingMode::Accumulator {
                    let carry_flag = if self.get_flag(Flags::C) { 1 } else { 0 };
                    self.set_flag(Flags::C, (self.accumulator & 0x01) != 0);
                    self.accumulator = self.accumulator >> 1 | (carry_flag << 7);
                    self.set_zn(self.accumulator);
                } else {
                    let carry_flag = if self.get_flag(Flags::C) { 1 } else { 0 };
                    let value = self.mem_read(addr);
                    self.set_flag(Flags::C, (value & 0x01) != 0);
                    let result = value >> 1 | (carry_flag << 7);
                    self.mem_write(addr, result);
                    self.set_zn(result);
                }
            }
            Opcode::AND => {
                let val = self.mem_read(addr);
                self.accumulator = val & self.accumulator;
                self.set_zn(self.accumulator);
            }
            Opcode::ORA => {
                let val = self.mem_read(addr);
                self.accumulator = val | self.accumulator;
                self.set_zn(self.accumulator);
            }
            Opcode::EOR => {
                let val = self.mem_read(addr);
                self.accumulator = self.accumulator ^ val;
                self.set_zn(self.accumulator);
            }
            Opcode::BIT => {
                let val = self.mem_read(addr);
                self.set_flag(Flags::N, (val & 0x80) != 0);
                self.set_flag(Flags::V, (val & 0x40) != 0);
                self.set_flag(Flags::Z, (val & self.accumulator) == 0);
            }
            Opcode::CMP => {
                let val = self.mem_read(addr);
                let result = self.accumulator.wrapping_sub(val);
                self.set_flag(Flags::C, self.accumulator >= val);
                self.set_flag(Flags::Z, self.accumulator == val);
                self.set_flag(Flags::N, (result & 0x80) != 0);
            }
            Opcode::CPX => {
                let val = self.mem_read(addr);
                let result = self.register_x.wrapping_sub(val);
                self.set_flag(Flags::C, self.register_x >= val);
                self.set_flag(Flags::Z, self.register_x == val);
                self.set_flag(Flags::N, (result & 0x80) != 0);
            }
            Opcode::CPY => {
                let val = self.mem_read(addr);
                let result = self.register_y.wrapping_sub(val);
                self.set_flag(Flags::C, self.register_y >= val);
                self.set_flag(Flags::Z, self.register_y == val);
                self.set_flag(Flags::N, (result & 0x80) != 0);
            }
            Opcode::BCC => {
                if !(self.get_flag(Flags::C)) {
                    self.program_counter = addr;
                }
            }
            Opcode::BCS => {
                if self.get_flag(Flags::C) {
                    self.program_counter = addr;
                }
            }
            Opcode::BEQ => {
                if self.get_flag(Flags::Z) {
                    self.program_counter = addr;
                }
            }
            Opcode::BMI => {
                if self.get_flag(Flags::N) {
                    self.program_counter = addr;
                }
            }
            Opcode::BNE => {
                if !self.get_flag(Flags::Z) {
                    self.program_counter = addr;
                }
            }
            Opcode::BPL => {
                if !self.get_flag(Flags::N) {
                    self.program_counter = addr;
                }
            }
            Opcode::BRK => {
                self.program_counter += 1; // Skip padding byte
                let high = (self.program_counter >> 8) as u8;
                let low = (self.program_counter & 0xFF) as u8;
                self.push(high);
                self.push(low);
                self.push(self.status | 0x30); // Push status with B and U flags set
                self.set_flag(Flags::I, true);
                self.load_irq_pc();
            }
            Opcode::BVC => {
                if !self.get_flag(Flags::V) {
                    self.program_counter = addr;
                }
            }
            Opcode::BVS => {
                if self.get_flag(Flags::V) {
                    self.program_counter = addr;
                }
            }
            Opcode::CLC => {
                self.set_flag(Flags::C, false);
            }
            Opcode::CLD => {
                self.set_flag(Flags::D, false);
            }
            Opcode::CLI => {
                self.set_flag(Flags::I, false);
            }
            Opcode::CLV => {
                self.set_flag(Flags::V, false);
            }
            Opcode::JMP => {
                self.program_counter = addr;
            }
            Opcode::JSR => {
                let return_addr = self.program_counter - 1;
                let high = (return_addr >> 8) as u8;
                let low = (return_addr & 0xFF) as u8;
                self.push(high);
                self.push(low);
                self.program_counter = addr;
            }
            Opcode::SEC => {
                self.set_flag(Flags::C, true);
            }
            Opcode::SED => {
                self.set_flag(Flags::D, true);
            }
            Opcode::SEI => {
                self.set_flag(Flags::I, true);
            }
            Opcode::PHA => {
                self.push(self.accumulator);
            }
            Opcode::PHP => {
                self.push(self.status | 0x30); // Push with B and U flags set
            }
            Opcode::PLA => {
                self.accumulator = self.pop();
                self.set_zn(self.accumulator);
            }
            Opcode::PLP => {
                self.status = (self.pop() & 0xEF) | 0x20;
            }
            Opcode::RTS => {
                let low = self.pop();
                let high = self.pop();
                self.program_counter = ((high as u16) << 8) | (low as u16);
                self.program_counter += 1; // RTS increments the return address
            }
            Opcode::RTI => {
                self.status = (self.pop() & 0xEF) | 0x20;
                let low = self.pop();
                let high = self.pop();
                self.program_counter = ((high as u16) << 8) | (low as u16);
            }
            Opcode::NOP => {}
            _ => println!("{:#?} not yet supported", opcode_copy),
        }
    }
}
