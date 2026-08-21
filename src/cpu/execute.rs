use super::CPU;
use super::types::{AddressingMode, Flags, Instruction, Opcode};

impl CPU {
    pub fn adc(&mut self, val: u8, acc: u8) -> u8 {
        let carry = self.get_flag(Flags::C) as u16;
        let sum = acc as u16 + val as u16 + carry;
        self.set_carry(sum);
        let result = sum as u8;
        self.set_overflow(val, acc, result);
        self.set_zn(result);
        result
    }

    pub fn sbc(&mut self, acc: u8, mem: u8) -> u8 {
        self.adc(!mem, acc)
    }

    fn compare(&mut self, register: u8, val: u8) {
        let result = register.wrapping_sub(val);
        self.set_flag(Flags::C, register >= val);
        self.set_zn(result);
    }

    fn operand(&mut self, mode: AddressingMode, addr: u16) -> u8 {
        if mode == AddressingMode::Accumulator {
            self.accumulator
        } else {
            self.bus.read(addr)
        }
    }

    fn store_result(&mut self, mode: AddressingMode, addr: u16, val: u8) {
        if mode == AddressingMode::Accumulator {
            self.accumulator = val;
        } else {
            self.bus.write(addr, val);
        }
    }

    fn asl(&mut self, val: u8) -> u8 {
        self.set_flag(Flags::C, val & 0x80 != 0);
        let result = val << 1;
        self.set_zn(result);
        result
    }

    fn lsr(&mut self, val: u8) -> u8 {
        self.set_flag(Flags::C, val & 0x01 != 0);
        let result = val >> 1;
        self.set_zn(result);
        result
    }

    fn rol(&mut self, val: u8) -> u8 {
        let carry = self.get_flag(Flags::C) as u8;
        self.set_flag(Flags::C, val & 0x80 != 0);
        let result = (val << 1) | carry;
        self.set_zn(result);
        result
    }

    fn ror(&mut self, val: u8) -> u8 {
        let carry = self.get_flag(Flags::C) as u8;
        self.set_flag(Flags::C, val & 0x01 != 0);
        let result = (val >> 1) | (carry << 7);
        self.set_zn(result);
        result
    }

    fn branch(&mut self, condition: bool, target: u16, page_crossed: bool) -> u32 {
        if !condition {
            return 0;
        }
        self.program_counter = target;
        1 + page_crossed as u32
    }

    fn store_high_and(&mut self, addr: u16, index: u8, register: u8) {
        let base = addr.wrapping_sub(index as u16);
        let value = register & ((base >> 8) as u8).wrapping_add(1);
        let addr = if base & 0xFF00 == addr & 0xFF00 {
            addr
        } else {
            (value as u16) << 8 | (addr & 0x00FF)
        };
        self.bus.write(addr, value);
    }

    pub fn execute(&mut self, instruction: Instruction, addr: u16, page_crossed: bool) -> u32 {
        let mode = instruction.addressing_mode;
        match instruction.opcode {
            Opcode::LDA => {
                self.accumulator = self.bus.read(addr);
                self.set_zn(self.accumulator);
            }
            Opcode::LDX => {
                self.register_x = self.bus.read(addr);
                self.set_zn(self.register_x);
            }
            Opcode::LDY => {
                self.register_y = self.bus.read(addr);
                self.set_zn(self.register_y);
            }
            Opcode::STA => self.bus.write(addr, self.accumulator),
            Opcode::STX => self.bus.write(addr, self.register_x),
            Opcode::STY => self.bus.write(addr, self.register_y),

            //transfers
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

            //aritmethic
            Opcode::ADC => {
                let val = self.bus.read(addr);
                self.accumulator = self.adc(val, self.accumulator);
            }
            Opcode::SBC => {
                let val = self.bus.read(addr);
                self.accumulator = self.sbc(self.accumulator, val);
            }
            Opcode::INC => {
                let res = self.bus.read(addr).wrapping_add(1);
                self.bus.write(addr, res);
                self.set_zn(res);
            }
            Opcode::INX => {
                self.register_x = self.register_x.wrapping_add(1);
                self.set_zn(self.register_x);
            }
            Opcode::INY => {
                self.register_y = self.register_y.wrapping_add(1);
                self.set_zn(self.register_y);
            }
            Opcode::DEC => {
                let res = self.bus.read(addr).wrapping_sub(1);
                self.bus.write(addr, res);
                self.set_zn(res);
            }
            Opcode::DEX => {
                self.register_x = self.register_x.wrapping_sub(1);
                self.set_zn(self.register_x);
            }
            Opcode::DEY => {
                self.register_y = self.register_y.wrapping_sub(1);
                self.set_zn(self.register_y);
            }

            Opcode::ASL => {
                let val = self.operand(mode, addr);
                let res = self.asl(val);
                self.store_result(mode, addr, res);
            }
            Opcode::LSR => {
                let val = self.operand(mode, addr);
                let res = self.lsr(val);
                self.store_result(mode, addr, res);
            }
            Opcode::ROL => {
                let val = self.operand(mode, addr);
                let res = self.rol(val);
                self.store_result(mode, addr, res);
            }
            Opcode::ROR => {
                let val = self.operand(mode, addr);
                let res = self.ror(val);
                self.store_result(mode, addr, res);
            }
            Opcode::AND => {
                self.accumulator &= self.bus.read(addr);
                self.set_zn(self.accumulator);
            }
            Opcode::ORA => {
                self.accumulator |= self.bus.read(addr);
                self.set_zn(self.accumulator);
            }
            Opcode::EOR => {
                self.accumulator ^= self.bus.read(addr);
                self.set_zn(self.accumulator);
            }
            Opcode::BIT => {
                let val = self.bus.read(addr);
                self.set_flag(Flags::N, val & 0x80 != 0);
                self.set_flag(Flags::V, val & 0x40 != 0);
                self.set_flag(Flags::Z, val & self.accumulator == 0);
            }

            Opcode::CMP => {
                let val = self.bus.read(addr);
                self.compare(self.accumulator, val);
            }
            Opcode::CPX => {
                let val = self.bus.read(addr);
                self.compare(self.register_x, val);
            }
            Opcode::CPY => {
                let val = self.bus.read(addr);
                self.compare(self.register_y, val);
            }
            Opcode::BCC => return self.branch(!self.get_flag(Flags::C), addr, page_crossed),
            Opcode::BCS => return self.branch(self.get_flag(Flags::C), addr, page_crossed),
            Opcode::BEQ => return self.branch(self.get_flag(Flags::Z), addr, page_crossed),
            Opcode::BNE => return self.branch(!self.get_flag(Flags::Z), addr, page_crossed),
            Opcode::BMI => return self.branch(self.get_flag(Flags::N), addr, page_crossed),
            Opcode::BPL => return self.branch(!self.get_flag(Flags::N), addr, page_crossed),
            Opcode::BVC => return self.branch(!self.get_flag(Flags::V), addr, page_crossed),
            Opcode::BVS => return self.branch(self.get_flag(Flags::V), addr, page_crossed),

            Opcode::JMP => self.program_counter = addr,
            Opcode::JSR => {
                let return_addr = self.program_counter.wrapping_sub(1);
                self.push((return_addr >> 8) as u8);
                self.push(return_addr as u8);
                self.program_counter = addr;
            }
            Opcode::RTS => {
                let lo = self.pop() as u16;
                let hi = self.pop() as u16;
                self.program_counter = ((hi << 8) | lo).wrapping_add(1);
            }
            Opcode::RTI => {
                self.status = (self.pop() & !(Flags::B as u8)) | Flags::U as u8;
                let lo = self.pop() as u16;
                let hi = self.pop() as u16;
                self.program_counter = (hi << 8) | lo;
            }
            Opcode::BRK => {
                let return_addr = self.program_counter.wrapping_add(1);
                self.push((return_addr >> 8) as u8);
                self.push(return_addr as u8);
                self.push(self.status | Flags::B as u8 | Flags::U as u8);
                self.set_flag(Flags::I, true);
                self.program_counter = self.bus.read_u16(0xFFFE);
            }

            Opcode::CLC => self.set_flag(Flags::C, false),
            Opcode::CLD => self.set_flag(Flags::D, false),
            Opcode::CLI => self.set_flag(Flags::I, false),
            Opcode::CLV => self.set_flag(Flags::V, false),
            Opcode::SEC => self.set_flag(Flags::C, true),
            Opcode::SED => self.set_flag(Flags::D, true),
            Opcode::SEI => self.set_flag(Flags::I, true),
            Opcode::PHA => self.push(self.accumulator),
            Opcode::PHP => self.push(self.status | Flags::B as u8 | Flags::U as u8),
            Opcode::PLA => {
                self.accumulator = self.pop();
                self.set_zn(self.accumulator);
            }
            Opcode::PLP => {
                self.status = (self.pop() & !(Flags::B as u8)) | Flags::U as u8;
            }

            Opcode::NOP => {
                if instruction.illegal && mode != AddressingMode::Implied {
                    self.bus.read(addr);
                }
            }
            //unofficial
            Opcode::LAX => {
                self.accumulator = self.bus.read(addr);
                self.register_x = self.accumulator;
                self.set_zn(self.accumulator);
            }
            Opcode::SAX => self.bus.write(addr, self.accumulator & self.register_x),
            Opcode::DCP => {
                let val = self.bus.read(addr).wrapping_sub(1);
                self.bus.write(addr, val);
                self.compare(self.accumulator, val);
            }
            Opcode::ISB => {
                let val = self.bus.read(addr).wrapping_add(1);
                self.bus.write(addr, val);
                self.accumulator = self.sbc(self.accumulator, val);
            }
            Opcode::SLO => {
                let val = self.bus.read(addr);
                let res = self.asl(val);
                self.bus.write(addr, res);
                self.accumulator |= res;
                self.set_zn(self.accumulator);
            }
            Opcode::RLA => {
                let val = self.bus.read(addr);
                let res = self.rol(val);
                self.bus.write(addr, res);
                self.accumulator &= res;
                self.set_zn(self.accumulator);
            }
            Opcode::SRE => {
                let val = self.bus.read(addr);
                let res = self.lsr(val);
                self.bus.write(addr, res);
                self.accumulator ^= res;
                self.set_zn(self.accumulator);
            }
            Opcode::RRA => {
                let val = self.bus.read(addr);
                let res = self.ror(val);
                self.bus.write(addr, res);
                self.accumulator = self.adc(res, self.accumulator);
            }
            Opcode::ANC => {
                self.accumulator &= self.bus.read(addr);
                self.set_zn(self.accumulator);
                self.set_flag(Flags::C, self.get_flag(Flags::N));
            }
            Opcode::ALR => {
                self.accumulator &= self.bus.read(addr);
                self.accumulator = self.lsr(self.accumulator);
            }
            Opcode::ARR => {
                self.accumulator &= self.bus.read(addr);
                self.accumulator = self.ror(self.accumulator);
                let bit6 = (self.accumulator >> 6) & 1;
                let bit5 = (self.accumulator >> 5) & 1;
                self.set_flag(Flags::C, bit6 == 1);
                self.set_flag(Flags::V, bit6 ^ bit5 == 1);
            }
            Opcode::AXS => {
                let val = self.bus.read(addr);
                let base = self.accumulator & self.register_x;
                self.set_flag(Flags::C, base >= val);
                self.register_x = base.wrapping_sub(val);
                self.set_zn(self.register_x);
            }
            Opcode::XAA => {
                self.accumulator = self.register_x & self.bus.read(addr);
                self.set_zn(self.accumulator);
            }
            Opcode::LAS => {
                let val = self.bus.read(addr) & self.stack_pointer;
                self.accumulator = val;
                self.register_x = val;
                self.stack_pointer = val;
                self.set_zn(val);
            }
            Opcode::AHX => {
                let value = self.accumulator & self.register_x;
                self.store_high_and(addr, self.register_y, value);
            }
            Opcode::SHY => self.store_high_and(addr, self.register_x, self.register_y),
            Opcode::SHX => self.store_high_and(addr, self.register_y, self.register_x),
            Opcode::TAS => {
                self.stack_pointer = self.accumulator & self.register_x;
                let value = self.stack_pointer;
                self.store_high_and(addr, self.register_y, value);
            }
            Opcode::JAM => {
                self.jammed = true;
            }
        }
        0
    }
}
