mod addressing;
mod execute;
mod opcodes;
pub mod types;

use crate::bus::Bus;
use crate::mapper::Mapper;
use types::{AddressingMode, Flags, Instruction, Opcode};

const STACK_BASE: u16 = 0x0100;
const NMI_VECTOR: u16 = 0xFFFA;
const RESET_VECTOR: u16 = 0xFFFC;
const IRQ_VECTOR: u16 = 0xFFFE;

pub struct CPU {
    accumulator: u8,
    program_counter: u16,
    register_x: u8,
    register_y: u8,
    stack_pointer: u8,
    status: u8,
    pub bus: Bus,
    pub cycles: u64,
    /// Set by the unofficial JAM opcodes, which halt the real chip.
    pub jammed: bool,
}

impl CPU {
    pub fn new(mapper: Box<dyn Mapper>, sample_rate: u32) -> Self {
        Self {
            accumulator: 0,
            program_counter: 0,
            register_x: 0,
            register_y: 0,
            stack_pointer: 0xFD,
            status: 0x24,
            bus: Bus::new(mapper, sample_rate),
            cycles: 0,
            jammed: false,
        }
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.program_counter = pc;
    }

    pub fn pc(&self) -> u16 {
        self.program_counter
    }

    pub fn reset(&mut self) {
        self.accumulator = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.stack_pointer = 0xFD;
        self.status = 0x24;
        self.jammed = false;
        // The PPU restarts its frame first; the vector fetch below then runs
        // against the freshly reset counters.
        self.bus.ppu.reset();
        self.bus.begin_instruction();
        self.program_counter = self.bus.read_u16(RESET_VECTOR);
        // Reset itself takes seven cycles, which is where nestest's CYC starts.
        self.cycles = 7;
        self.bus.finish_instruction(7);
    }

    /// Run one instruction (or service a pending interrupt) and return the
    /// number of CPU cycles it consumed.
    pub fn step(&mut self) -> u32 {
        // OAM DMA and DMC fetches steal cycles before anything else runs.
        let stall = self.bus.take_stall_cycles();
        if stall > 0 {
            self.advance(stall);
        }

        if self.jammed {
            self.advance(1);
            return 1;
        }

        if self.bus.poll_nmi() {
            self.interrupt(NMI_VECTOR);
            return 7;
        }
        if self.bus.poll_irq() && !self.get_flag(Flags::I) {
            self.interrupt(IRQ_VECTOR);
            return 7;
        }

        self.bus.begin_instruction();
        let opcode = self.fetch_byte();
        let instruction = decode(opcode);
        let (addr, page_crossed) = self.resolve_addr(instruction.addressing_mode);

        let mut cycles = instruction.cycles as u32;
        if instruction.page_penalty && page_crossed {
            cycles += 1;
        }
        cycles += self.execute(instruction, addr, page_crossed);

        self.cycles += cycles as u64;
        self.bus.finish_instruction(cycles);
        cycles
    }

    /// Burn cycles that belong to no instruction: reset, interrupts and DMA.
    fn advance(&mut self, cycles: u32) {
        self.cycles += cycles as u64;
        self.bus.tick(cycles);
    }

    /// Push the return address and status, then jump through `vector`.
    fn interrupt(&mut self, vector: u16) {
        self.bus.begin_instruction();
        self.push((self.program_counter >> 8) as u8);
        self.push(self.program_counter as u8);
        // Hardware interrupts push the status with B clear.
        self.push((self.status & !(Flags::B as u8)) | Flags::U as u8);
        self.set_flag(Flags::I, true);
        self.program_counter = self.bus.read_u16(vector);
        self.cycles += 7;
        self.bus.finish_instruction(7);
    }

    pub fn fetch_byte(&mut self) -> u8 {
        let byte = self.bus.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);
        byte
    }

    pub fn fetch_word(&mut self) -> u16 {
        let word = self.bus.read_u16(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(2);
        word
    }

    pub fn push(&mut self, val: u8) {
        self.bus.write(STACK_BASE | self.stack_pointer as u16, val);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    pub fn pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.bus.read(STACK_BASE | self.stack_pointer as u16)
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

    // --------------------------------------------------------------- tracing

    /// A single trace line in nestest's format, for regression-testing the CPU
    /// against a known-good log.
    pub fn trace(&mut self) -> String {
        let pc = self.program_counter;
        let opcode = self.bus.peek(pc);
        let instruction = decode(opcode);

        let bytes = match instruction.addressing_mode.length() {
            1 => format!("{:02X}", opcode),
            2 => format!("{:02X} {:02X}", opcode, self.bus.peek(pc + 1)),
            _ => format!(
                "{:02X} {:02X} {:02X}",
                opcode,
                self.bus.peek(pc + 1),
                self.bus.peek(pc + 2)
            ),
        };

        let disasm = self.disassemble(pc, &instruction);
        let (scanline, dot) = self.bus.ppu.position();

        format!(
            "{:04X}  {:<8} {}{:<32}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} \
             PPU:{:3},{:3} CYC:{}",
            pc,
            bytes,
            if instruction.illegal { '*' } else { ' ' },
            disasm,
            self.accumulator,
            self.register_x,
            self.register_y,
            self.status,
            self.stack_pointer,
            scanline,
            dot,
            self.cycles
        )
    }

    fn disassemble(&mut self, pc: u16, instruction: &Instruction) -> String {
        let mnemonic = format!("{:?}", instruction.opcode);

        match instruction.addressing_mode {
            AddressingMode::Implied => mnemonic,
            AddressingMode::Accumulator => format!("{} A", mnemonic),
            AddressingMode::Immediate => {
                format!("{} #${:02X}", mnemonic, self.bus.peek(pc + 1))
            }
            AddressingMode::ZeroPage => {
                let addr = self.bus.peek(pc + 1);
                let value = self.bus.peek(addr as u16);
                format!("{} ${:02X} = {:02X}", mnemonic, addr, value)
            }
            AddressingMode::ZeroPageX => {
                let addr = self.bus.peek(pc + 1);
                let effective = addr.wrapping_add(self.register_x);
                let value = self.bus.peek(effective as u16);
                format!("{} ${:02X},X @ {:02X} = {:02X}", mnemonic, addr, effective, value)
            }
            AddressingMode::ZeroPageY => {
                let addr = self.bus.peek(pc + 1);
                let effective = addr.wrapping_add(self.register_y);
                let value = self.bus.peek(effective as u16);
                format!("{} ${:02X},Y @ {:02X} = {:02X}", mnemonic, addr, effective, value)
            }
            AddressingMode::Absolute => {
                let addr = self.bus.peek_u16(pc + 1);
                // Jumps show only the target; everything else shows the operand.
                if matches!(instruction.opcode, Opcode::JMP | Opcode::JSR) {
                    format!("{} ${:04X}", mnemonic, addr)
                } else {
                    format!("{} ${:04X} = {:02X}", mnemonic, addr, self.bus.peek(addr))
                }
            }
            AddressingMode::AbsoluteX => {
                let addr = self.bus.peek_u16(pc + 1);
                let effective = addr.wrapping_add(self.register_x as u16);
                let value = self.bus.peek(effective);
                format!("{} ${:04X},X @ {:04X} = {:02X}", mnemonic, addr, effective, value)
            }
            AddressingMode::AbsoluteY => {
                let addr = self.bus.peek_u16(pc + 1);
                let effective = addr.wrapping_add(self.register_y as u16);
                let value = self.bus.peek(effective);
                format!("{} ${:04X},Y @ {:04X} = {:02X}", mnemonic, addr, effective, value)
            }
            AddressingMode::Indirect => {
                let ptr = self.bus.peek_u16(pc + 1);
                let addr = if ptr & 0x00FF == 0x00FF {
                    (self.bus.peek(ptr & 0xFF00) as u16) << 8 | self.bus.peek(ptr) as u16
                } else {
                    self.bus.peek_u16(ptr)
                };
                format!("{} (${:04X}) = {:04X}", mnemonic, ptr, addr)
            }
            AddressingMode::IndirectX => {
                let ptr = self.bus.peek(pc + 1);
                let ptr_addr = ptr.wrapping_add(self.register_x);
                let addr = (self.bus.peek(ptr_addr.wrapping_add(1) as u16) as u16) << 8
                    | self.bus.peek(ptr_addr as u16) as u16;
                let value = self.bus.peek(addr);
                format!(
                    "{} (${:02X},X) @ {:02X} = {:04X} = {:02X}",
                    mnemonic, ptr, ptr_addr, addr, value
                )
            }
            AddressingMode::IndirectY => {
                let ptr = self.bus.peek(pc + 1);
                let addr = (self.bus.peek(ptr.wrapping_add(1) as u16) as u16) << 8
                    | self.bus.peek(ptr as u16) as u16;
                let effective = addr.wrapping_add(self.register_y as u16);
                let value = self.bus.peek(effective);
                format!(
                    "{} (${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                    mnemonic, ptr, addr, effective, value
                )
            }
            AddressingMode::Relative => {
                let offset = self.bus.peek(pc + 1) as i8;
                let target = pc.wrapping_add(2).wrapping_add(offset as u16);
                format!("{} ${:04X}", mnemonic, target)
            }
        }
    }
}

fn decode(opcode: u8) -> Instruction {
    opcodes::decode(opcode)
}
