use std::ops::Add;

use crate::bus::Bus;

pub struct CPU {
    accumulator: u8,
    program_counter: u16,
    register_x: u8,
    register_y: u8,
    stack_pointer: u8,
    status: u8,
    bus: Bus,
}

pub struct Instruction {
    opcode: Opcode,
    addressing_mode: AddressingMode,
    cycles: u8,
}

pub enum Flags {
    C = (1 << 0), // Carry flag
    Z = (1 << 1), // Zero flag
    I = (1 << 2), // Disable interrupts
    D = (1 << 3), // Decimal mode
    B = (1 << 4), // Break
    U = (1 << 5), // Unused
    V = (1 << 6), // Overflow
    N = (1 << 7), // Negative
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX, // (Indirect, X)
    IndirectY, // (Indirect, Y)
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    // Arithmetic
    ADC, // Add with Carry
    SBC, // Subtract with Carry

    // Logical
    AND, // Logical AND
    EOR, // Exclusive OR
    ORA, // Logical OR

    // Shifts & Rotates
    ASL, // Arithmetic Shift Left
    LSR, // Logical Shift Right
    ROL, // Rotate Left
    ROR, // Rotate Right

    // Branches
    BCC, // Branch if Carry Clear
    BCS, // Branch if Carry Set
    BEQ, // Branch if Equal (zero set)
    BMI, // Branch if Minus (negative set)
    BNE, // Branch if Not Equal (zero clear)
    BPL, // Branch if Plus (negative clear)
    BVC, // Branch if Overflow Clear
    BVS, // Branch if Overflow Set

    // Status flag changes
    CLC, // Clear Carry Flag
    CLD, // Clear Decimal Mode
    CLI, // Clear Interrupt Disable
    CLV, // Clear Overflow Flag
    SEC, // Set Carry Flag
    SED, // Set Decimal Flag
    SEI, // Set Interrupt Disable

    // Compare
    CMP, // Compare Accumulator
    CPX, // Compare X Register
    CPY, // Compare Y Register
    BIT, // Test Bits in Memory

    // Increments & Decrements
    DEC, // Decrement Memory
    DEX, // Decrement X Register
    DEY, // Decrement Y Register
    INC, // Increment Memory
    INX, // Increment X Register
    INY, // Increment Y Register

    // Jumps & Subroutines
    JMP, // Jump to Address
    JSR, // Jump to Subroutine
    RTI, // Return from Interrupt
    RTS, // Return from Subroutine
    BRK, // Force Interrupt (break)

    // Load/Store
    LDA, // Load Accumulator
    LDX, // Load X Register
    LDY, // Load Y Register
    STA, // Store Accumulator
    STX, // Store X Register
    STY, // Store Y Register

    // Register Transfers
    TAX, // Transfer Accumulator to X
    TAY, // Transfer Accumulator to Y
    TSX, // Transfer Stack Pointer to X
    TXA, // Transfer X to Accumulator
    TXS, // Transfer X to Stack Pointer
    TYA, // Transfer Y to Accumulator

    // Stack Operations
    PHA, // Push Accumulator
    PHP, // Push Processor Status
    PLA, // Pull Accumulator
    PLP, // Pull Processor Status

    // Other
    NOP, // No Operation

    // For unmapped/illegal opcodes
    Unknown,
}

pub trait Mem {
    fn mem_read(&self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, data: u8);
    fn mem_read_u16(&self, pos: u16) -> u16;
    fn mem_write_u16(&mut self, pos: u16, data: u16);
}

impl Mem for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data)
    }

    fn mem_read_u16(&self, pos: u16) -> u16 {
        self.bus.mem_read_u16(pos)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        self.bus.mem_write_u16(pos, data)
    }
}

impl CPU {
    pub fn new() -> Self {
        Self {
            accumulator: 0,
            program_counter: 0x8000,
            register_x: 0,
            register_y: 0,
            stack_pointer: 0xFD,
            status: 0x24,
            bus: Bus::new(),
        }
    }

    pub fn step(&mut self) {
        let opcode = self.fetch_byte();
        let instruction = self.decode(opcode);
        self.execute(instruction);
    }

    pub fn fetch_byte(&mut self) -> u8 {
        let opcode = self.mem_read(self.program_counter) as u8;
        self.program_counter += 1;
        opcode
    }

    pub fn fetch_word(&mut self) -> u16 {
        let opcode = self.mem_read_u16(self.program_counter);
        self.program_counter += 2;
        opcode
    }

    fn decode(&self, opcode: u8) -> Instruction {
        match opcode {
            0x00 => Instruction {
                opcode: Opcode::BRK,
                addressing_mode: AddressingMode::Implied,
                cycles: 7,
            },
            0x01 => Instruction {
                opcode: Opcode::ORA,
                addressing_mode: AddressingMode::IndirectX,
                cycles: 6,
            },
            0x05 => Instruction {
                opcode: Opcode::ORA,
                addressing_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0x06 => Instruction {
                opcode: Opcode::ASL,
                addressing_mode: AddressingMode::ZeroPage,
                cycles: 5,
            },
            0x08 => Instruction {
                opcode: Opcode::PHP,
                addressing_mode: AddressingMode::Implied,
                cycles: 3,
            },
            0x09 => Instruction {
                opcode: Opcode::ORA,
                addressing_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0x0A => Instruction {
                opcode: Opcode::ASL,
                addressing_mode: AddressingMode::Accumulator,
                cycles: 2,
            },
            0x0D => Instruction {
                opcode: Opcode::ORA,
                addressing_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            0x0E => Instruction {
                opcode: Opcode::ASL,
                addressing_mode: AddressingMode::Absolute,
                cycles: 6,
            },
            0x10 => Instruction {
                opcode: Opcode::BPL,
                addressing_mode: AddressingMode::Relative,
                cycles: 2,
            },
            0x18 => Instruction {
                opcode: Opcode::CLC,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0x20 => Instruction {
                opcode: Opcode::JSR,
                addressing_mode: AddressingMode::Absolute,
                cycles: 6,
            },
            0x29 => Instruction {
                opcode: Opcode::AND,
                addressing_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0x2A => Instruction {
                opcode: Opcode::ROL,
                addressing_mode: AddressingMode::Accumulator,
                cycles: 2,
            },
            0x38 => Instruction {
                opcode: Opcode::SEC,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0x40 => Instruction {
                opcode: Opcode::RTI,
                addressing_mode: AddressingMode::Implied,
                cycles: 6,
            },
            0x48 => Instruction {
                opcode: Opcode::PHA,
                addressing_mode: AddressingMode::Implied,
                cycles: 3,
            },
            0x49 => Instruction {
                opcode: Opcode::EOR,
                addressing_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0x4A => Instruction {
                opcode: Opcode::LSR,
                addressing_mode: AddressingMode::Accumulator,
                cycles: 2,
            },
            0x4C => Instruction {
                opcode: Opcode::JMP,
                addressing_mode: AddressingMode::Absolute,
                cycles: 3,
            },
            0x58 => Instruction {
                opcode: Opcode::CLI,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0x60 => Instruction {
                opcode: Opcode::RTS,
                addressing_mode: AddressingMode::Implied,
                cycles: 6,
            },
            0x68 => Instruction {
                opcode: Opcode::PLA,
                addressing_mode: AddressingMode::Implied,
                cycles: 4,
            },
            0x69 => Instruction {
                opcode: Opcode::ADC,
                addressing_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0x6A => Instruction {
                opcode: Opcode::ROR,
                addressing_mode: AddressingMode::Accumulator,
                cycles: 2,
            },
            0x6C => Instruction {
                opcode: Opcode::JMP,
                addressing_mode: AddressingMode::Indirect,
                cycles: 5,
            },
            0x78 => Instruction {
                opcode: Opcode::SEI,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0x84 => Instruction {
                opcode: Opcode::STY,
                addressing_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0x85 => Instruction {
                opcode: Opcode::STA,
                addressing_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0x86 => Instruction {
                opcode: Opcode::STX,
                addressing_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0x88 => Instruction {
                opcode: Opcode::DEY,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0x8A => Instruction {
                opcode: Opcode::TXA,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0x8D => Instruction {
                opcode: Opcode::STA,
                addressing_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            0x90 => Instruction {
                opcode: Opcode::BCC,
                addressing_mode: AddressingMode::Relative,
                cycles: 2,
            },
            0x98 => Instruction {
                opcode: Opcode::TYA,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0x9A => Instruction {
                opcode: Opcode::TXS,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xA0 => Instruction {
                opcode: Opcode::LDY,
                addressing_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0xA2 => Instruction {
                opcode: Opcode::LDX,
                addressing_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0xA5 => Instruction {
                opcode: Opcode::LDA,
                addressing_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0xA8 => Instruction {
                opcode: Opcode::TAY,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xA9 => Instruction {
                opcode: Opcode::LDA,
                addressing_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0xAA => Instruction {
                opcode: Opcode::TAX,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xAD => Instruction {
                opcode: Opcode::LDA,
                addressing_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            0xB0 => Instruction {
                opcode: Opcode::BCS,
                addressing_mode: AddressingMode::Relative,
                cycles: 2,
            },
            0xB8 => Instruction {
                opcode: Opcode::CLV,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xBA => Instruction {
                opcode: Opcode::TSX,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xC0 => Instruction {
                opcode: Opcode::CPY,
                addressing_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0xC8 => Instruction {
                opcode: Opcode::INY,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xC9 => Instruction {
                opcode: Opcode::CMP,
                addressing_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0xCA => Instruction {
                opcode: Opcode::DEX,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xD0 => Instruction {
                opcode: Opcode::BNE,
                addressing_mode: AddressingMode::Relative,
                cycles: 2,
            },
            0xD8 => Instruction {
                opcode: Opcode::CLD,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xE0 => Instruction {
                opcode: Opcode::CPX,
                addressing_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0xE8 => Instruction {
                opcode: Opcode::INX,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xE9 => Instruction {
                opcode: Opcode::SBC,
                addressing_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0xEA => Instruction {
                opcode: Opcode::NOP,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xF0 => Instruction {
                opcode: Opcode::BEQ,
                addressing_mode: AddressingMode::Relative,
                cycles: 2,
            },
            0xF8 => Instruction {
                opcode: Opcode::SED,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
            _ => Instruction {
                opcode: Opcode::Unknown,
                addressing_mode: AddressingMode::Implied,
                cycles: 2,
            },
        }
    }

    pub fn adc(&mut self, val: u8, acc: u8) -> u8 {
        let carry = if self.get_flag(Flags::C) { 1 } else { 0 };
        self.set_overflow(val, acc, val + acc);
        let sum = acc as u16 + val as u16 + carry as u16;
        self.set_carry(sum);
        let result = sum as u8;
        self.set_zn(result);
        result
    }

    pub fn sbc(&mut self, acc: u8, mem: u8) -> u8 {
        let carry = if self.get_flag(Flags::C) { 0 } else { 1 };
        let sub = acc as i16 - mem as i16 - carry as i16;
        let overflow: i16 = (sub ^ acc as i16) & (sub ^ (mem as i16)) & 0x80;
        self.set_flag(Flags::V, overflow != 0);
        self.set_flag(Flags::C, sub >= 0);
        let result = sub as u8;
        self.set_zn(result);
        result
    }
    pub fn execute(&mut self, instruction: Instruction) {
        let addr = self.resolve_addr(&instruction.addressing_mode);
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
                let result = self.accumulator - val;
                self.set_flag(Flags::C, self.accumulator >= val);
                self.set_flag(Flags::Z, self.accumulator == val);
                self.set_flag(Flags::N, (result & 0x80) != 0);
            }
            Opcode::CPX => {
                let val = self.mem_read(addr);
                let result = self.register_x - val;
                self.set_flag(Flags::C, self.register_x >= val);
                self.set_flag(Flags::Z, self.register_x == val);
                self.set_flag(Flags::N, (result & 0x80) != 0);
            }
            _ => println!("Opcode not yet supported"),
        }
    }

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
                self.mem_read_u16(ptr as u16)
            }
            AddressingMode::IndirectY => {
                let base = self.fetch_byte();
                let ptr = self.mem_read_u16(base as u16);
                ptr.wrapping_add(self.register_y as u16)
            }
            _ => {
                println!("Addressmode not yet supported");
                0
            }
        }
    }

    pub fn set_flag(&mut self, flag: Flags, condition: bool) {
        if condition {
            self.status |= flag as u8;
        } else {
            self.status &= !(flag as u8);
        }
    }

    pub fn get_flag(&self, flag: Flags) -> bool {
        (self.status & (flag as u8)) != 0
    }

    pub fn set_zn(&mut self, value: u8) {
        self.set_flag(Flags::Z, value == 0);
        self.set_flag(Flags::N, (value & 0x80) != 0);
    }

    pub fn set_carry(&mut self, value: u16) {
        self.set_flag(Flags::C, value > 0xFF);
    }

    pub fn set_overflow(&mut self, a: u8, b: u8, result: u8) {
        let overflow = (a ^ result) & (b ^ result) & 0x80 != 0;
        self.set_flag(Flags::V, overflow);
    }

    pub fn reset(&mut self) {
        self.accumulator = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.stack_pointer = 0xFD;
        self.status = 0x24;
        self.program_counter = 0x8000;
    }

    pub fn push(&mut self, val: u8) {
        self.mem_write(0x0100 | self.stack_pointer as u16, val);
        self.stack_pointer -= 1;
    }

    pub fn load_irq_pc(&mut self) {
        let high = self.mem_read(0xFFFF);
        let low = self.mem_read(0xFFFE);
        self.program_counter = (high as u16) << 8 | low as u16;
    }
    pub fn irq(&mut self) {
        if !self.get_flag(Flags::I) {
            let high = (self.program_counter >> 8) as u8;
            let low = (self.program_counter & 0xFF) as u8;
            self.push(high);
            self.push(low);
            self.load_irq_pc();
            self.push(self.status | 0x20);
            self.set_flag(Flags::I, true);
        } else {
        }
    }

    pub fn nmi(&mut self) {
        let high = (self.program_counter >> 8) as u8;
        let low = (self.program_counter & 0xFF) as u8;
        self.push(high);
        self.push(low);
        self.load_irq_pc();
        self.push(self.status | 0x20);
        self.set_flag(Flags::I, true);
    }
}
