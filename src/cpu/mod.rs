mod addressing;
mod execute;
mod opcodes;
pub mod types;

use crate::bus::Bus;
use types::{AddressingMode, Flags, Instruction, Opcode};

pub struct CPU {
    accumulator: u8,
    program_counter: u16,
    register_x: u8,
    register_y: u8,
    stack_pointer: u8,
    status: u8,
    bus: Bus,
    cycles: u64,
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
            cycles: 0,
        }
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.program_counter = pc;
    }

    pub fn load(&mut self, rom: &[u8]) {
        self.bus.load_rom(rom, 0x8000);
    }

    pub fn step(&mut self) {
        let opcode = self.fetch_byte();
        let instruction = self.decode(opcode);
        let cycles_used = instruction.cycles as u64;
        self.execute(instruction);
        self.cycles += cycles_used;
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

    pub fn trace(&self) -> String {
        let pc = self.program_counter;
        let opcode = self.mem_read(pc);
        let instruction = self.decode(opcode);

        // Read instruction bytes (1-3 bytes)
        let bytes = match instruction.addressing_mode {
            AddressingMode::Implied | AddressingMode::Accumulator => {
                format!("{:02X}      ", opcode)
            }
            AddressingMode::Immediate
            | AddressingMode::ZeroPage
            | AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY
            | AddressingMode::IndirectX
            | AddressingMode::IndirectY
            | AddressingMode::Relative => {
                let byte1 = self.mem_read(pc + 1);
                format!("{:02X} {:02X}   ", opcode, byte1)
            }
            _ => {
                let byte1 = self.mem_read(pc + 1);
                let byte2 = self.mem_read(pc + 2);
                format!("{:02X} {:02X} {:02X}", opcode, byte1, byte2)
            }
        };

        // Disassemble instruction
        let disasm = self.disassemble(pc, &instruction);

        // Format: PC  BYTES  INSTRUCTION                      A:XX X:XX Y:XX P:XX SP:XX PPU:XXX,XXX CYC:XXX
        format!(
            "{:04X}  {}  {:<32}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
            pc,
            bytes,
            disasm,
            self.accumulator,
            self.register_x,
            self.register_y,
            self.status,
            self.stack_pointer,
            self.cycles
        )
    }

    fn disassemble(&self, pc: u16, instruction: &Instruction) -> String {
        let mnemonic = format!("{:?}", instruction.opcode);

        match instruction.addressing_mode {
            AddressingMode::Implied => mnemonic,
            AddressingMode::Accumulator => format!("{:?} {}", instruction.opcode, "A"),
            AddressingMode::Immediate => {
                let value = self.mem_read(pc + 1);
                format!("{} #${:02X}", mnemonic, value)
            }
            AddressingMode::ZeroPage => {
                let addr = self.mem_read(pc + 1);
                let value = self.mem_read(addr as u16);
                format!("{} ${:02X} = {:02X}", mnemonic, addr, value)
            }
            AddressingMode::ZeroPageX => {
                let addr = self.mem_read(pc + 1);
                let effective = addr.wrapping_add(self.register_x);
                let value = self.mem_read(effective as u16);
                format!(
                    "{} ${:02X},X @ {:02X} = {:02X}",
                    mnemonic, addr, effective, value
                )
            }
            AddressingMode::ZeroPageY => {
                let addr = self.mem_read(pc + 1);
                let effective = addr.wrapping_add(self.register_y);
                let value = self.mem_read(effective as u16);
                format!(
                    "{} ${:02X},Y @ {:02X} = {:02X}",
                    mnemonic, addr, effective, value
                )
            }
            AddressingMode::Absolute => {
                let addr = self.mem_read_u16(pc + 1);
                if instruction.opcode == Opcode::JMP || instruction.opcode == Opcode::JSR {
                    format!("{} ${:04X}", mnemonic, addr)
                } else {
                    let value = self.mem_read(addr);
                    format!("{} ${:04X} = {:02X}", mnemonic, addr, value)
                }
            }
            AddressingMode::AbsoluteX => {
                let addr = self.mem_read_u16(pc + 1);
                let effective = addr.wrapping_add(self.register_x as u16);
                let value = self.mem_read(effective);
                format!(
                    "{} ${:04X},X @ {:04X} = {:02X}",
                    mnemonic, addr, effective, value
                )
            }
            AddressingMode::AbsoluteY => {
                let addr = self.mem_read_u16(pc + 1);
                let effective = addr.wrapping_add(self.register_y as u16);
                let value = self.mem_read(effective);
                format!(
                    "{} ${:04X},Y @ {:04X} = {:02X}",
                    mnemonic, addr, effective, value
                )
            }
            AddressingMode::Indirect => {
                let ptr = self.mem_read_u16(pc + 1);
                if ptr & 0x00FF == 0x00FF {
                    let lo = self.mem_read(ptr) as u16;
                    let hi = self.mem_read(ptr & 0xFF00) as u16;
                    let addr = (hi << 8) | lo;
                    format!("{} (${:04X}) = {:04X}", mnemonic, ptr, addr)
                } else {
                    let addr = self.mem_read_u16(ptr);
                    format!("{} (${:04X}) = {:04X}", mnemonic, ptr, addr)
                }
            }
            AddressingMode::IndirectX => {
                let ptr = self.mem_read(pc + 1);
                let ptr_addr = ptr.wrapping_add(self.register_x);
                let addr = self.mem_read_u16(ptr_addr as u16);
                let value = self.mem_read(addr);
                format!(
                    "{} (${:02X},X) @ {:02X} = {:04X} = {:02X}",
                    mnemonic, ptr, ptr_addr, addr, value
                )
            }
            AddressingMode::IndirectY => {
                let ptr = self.mem_read(pc + 1);
                let addr = self.mem_read_u16(ptr as u16);
                let effective = addr.wrapping_add(self.register_y as u16);
                let value = self.mem_read(effective);
                format!(
                    "{} (${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                    mnemonic, ptr, addr, effective, value
                )
            }
            AddressingMode::Relative => {
                let offset = self.mem_read(pc + 1) as i8;
                let target = (pc as i32 + 2 + offset as i32) as u16;
                format!("{} ${:04X}", mnemonic, target)
            }
        }
    }

    fn decode(&self, opcode: u8) -> Instruction {
        opcodes::decode(opcode)
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
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    pub fn pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read(0x0100 | self.stack_pointer as u16)
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
