use crate::bus::Bus;

pub struct CPU {
    accumulator: u8,
    program_counter: u16,
    register_x: u8,
    register_y: u8,
    stack_pointer: u8,
    status: u8,
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
    IndexedIndirect, // (Indirect, X)
    IndirectIndexed, // (Indirect, Y)
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

impl CPU {
    pub fn new() -> Self {
        CPU {
            accumulator: 0,
            program_counter: 0x8000,
            register_x: 0,
            register_y: 0,
            stack_pointer: 0xFD,
            status: 0x24,
        }
    }

    pub fn read(&self, bus: &Bus, addr: u16) -> u8 {
        bus.read(addr)
    }

    pub fn write(&self, bus: &mut Bus, addr: u16, data: u8) {
        bus.write(addr, data);
    }

    pub fn fetch(&mut self, bus: &Bus) -> u8 {
        let data = self.read(bus, self.program_counter);
        self.program_counter += 1;
        data
    }

    pub fn clock(&mut self, bus: &mut Bus) {
        let opcode = self.fetch(bus);
        let instruction = self.decode(opcode);
        // TODO: Execute instruction
    }

    fn decode(&self, opcode: u8) -> Instruction {
        match opcode {
            0x00 => Instruction { opcode: Opcode::BRK, addressing_mode: AddressingMode::Implied, cycles: 7 },
            0x01 => Instruction { opcode: Opcode::ORA, addressing_mode: AddressingMode::IndexedIndirect, cycles: 6 },
            0x05 => Instruction { opcode: Opcode::ORA, addressing_mode: AddressingMode::ZeroPage, cycles: 3 },
            0x06 => Instruction { opcode: Opcode::ASL, addressing_mode: AddressingMode::ZeroPage, cycles: 5 },
            0x08 => Instruction { opcode: Opcode::PHP, addressing_mode: AddressingMode::Implied, cycles: 3 },
            0x09 => Instruction { opcode: Opcode::ORA, addressing_mode: AddressingMode::Immediate, cycles: 2 },
            0x0A => Instruction { opcode: Opcode::ASL, addressing_mode: AddressingMode::Accumulator, cycles: 2 },
            0x0D => Instruction { opcode: Opcode::ORA, addressing_mode: AddressingMode::Absolute, cycles: 4 },
            0x0E => Instruction { opcode: Opcode::ASL, addressing_mode: AddressingMode::Absolute, cycles: 6 },
            0x10 => Instruction { opcode: Opcode::BPL, addressing_mode: AddressingMode::Relative, cycles: 2 },
            0x18 => Instruction { opcode: Opcode::CLC, addressing_mode: AddressingMode::Implied, cycles: 2 },
            0x20 => Instruction { opcode: Opcode::JSR, addressing_mode: AddressingMode::Absolute, cycles: 6 },
            0x29 => Instruction { opcode: Opcode::AND, addressing_mode: AddressingMode::Immediate, cycles: 2 },
            0x2A => Instruction { opcode: Opcode::ROL, addressing_mode: AddressingMode::Accumulator, cycles: 2 },
            0x38 => Instruction { opcode: Opcode::SEC, addressing_mode: AddressingMode::Implied, cycles: 2 },
            0x40 => Instruction { opcode: Opcode::RTI, addressing_mode: AddressingMode::Implied, cycles: 6 },
            0x48 => Instruction { opcode: Opcode::PHA, addressing_mode: AddressingMode::Implied, cycles: 3 },
            0x49 => Instruction { opcode: Opcode::EOR, addressing_mode: AddressingMode::Immediate, cycles: 2 },
            0x4A => Instruction { opcode: Opcode::LSR, addressing_mode: AddressingMode::Accumulator, cycles: 2 },
            0x4C => Instruction { opcode: Opcode::JMP, addressing_mode: AddressingMode::Absolute, cycles: 3 },
            0x58 => Instruction { opcode: Opcode::CLI, addressing_mode: AddressingMode::Implied, cycles: 2 },
            0x60 => Instruction { opcode: Opcode::RTS, addressing_mode: AddressingMode::Implied, cycles: 6 },
            0x68 => Instruction { opcode: Opcode::PLA, addressing_mode: AddressingMode::Implied, cycles: 4 },
            0x69 => Instruction { opcode: Opcode::ADC, addressing_mode: AddressingMode::Immediate, cycles: 2 },
            0x6A => Instruction { opcode: Opcode::ROR, addressing_mode: AddressingMode::Accumulator, cycles: 2 },
            0x6C => Instruction { opcode: Opcode::JMP, addressing_mode: AddressingMode::Indirect, cycles: 5 },
            0x78 => Instruction { opcode: Opcode::SEI, addressing_mode: AddressingMode::Implied, cycles: 2 },
            0x84 => Instruction { opcode: Opcode::STY, addressing_mode: AddressingMode::ZeroPage, cycles: 3 },
            0x85 => Instruction { opcode: Opcode::STA, addressing_mode: AddressingMode::ZeroPage, cycles: 3 },
            0x86 => Instruction { opcode: Opcode::STX, addressing_mode: AddressingMode::ZeroPage, cycles: 3 },
            0x88 => Instruction { opcode: Opcode::DEY, addressing_mode: AddressingMode::Implied, cycles: 2 },
            0x8A => Instruction { opcode: Opcode::TXA, addressing_mode: AddressingMode::Implied, cycles: 2 },
            0x8D => Instruction { opcode: Opcode::STA, addressing_mode: AddressingMode::Absolute, cycles: 4 },
            0x90 => Instruction { opcode: Opcode::BCC, addressing_mode: AddressingMode::Relative, cycles: 2 },
            0x98 => Instruction { opcode: Opcode::TYA, addressing_mode: AddressingMode::Implied, cycles: 2 },
            0x9A => Instruction { opcode: Opcode::TXS, addressing_mode: AddressingMode::Implied, cycles: 2 },
            0xA0 => Instruction { opcode: Opcode::LDY, addressing_mode: AddressingMode::Immediate, cycles: 2 },
            0xA2 => Instruction { opcode: Opcode::LDX, addressing_mode: AddressingMode::Immediate, cycles: 2 },
            0xA5 => Instruction { opcode: Opcode::LDA, addressing_mode: AddressingMode::ZeroPage, cycles: 3 },
            0xA8 => Instruction { opcode: Opcode::TAY, addressing_mode: AddressingMode::Implied, cycles: 2 },
            0xA9 => Instruction { opcode: Opcode::LDA, addressing_mode: AddressingMode::Immediate, cycles: 2 },
            0xAA => Instruction { opcode: Opcode::TAX, addressing_mode: AddressingMode::Implied, cycles: 2 },
            0xAD => Instruction { opcode: Opcode::LDA, addressing_mode: AddressingMode::Absolute, cycles: 4 },
            0xB0 => Instruction { opcode: Opcode::BCS, addressing_mode: AddressingMode::Relative, cycles: 2 },
            0xB8 => Instruction { opcode: Opcode::CLV, addressing_mode: AddressingMode::Implied, cycles: 2 },
            0xBA => Instruction { opcode: Opcode::TSX, addressing_mode: AddressingMode::Implied, cycles: 2 },
            0xC0 => Instruction { opcode: Opcode::CPY, addressing_mode: AddressingMode::Immediate, cycles: 2 },
            0xC8 => Instruction { opcode: Opcode::INY, addressing_mode: AddressingMode::Implied, cycles: 2 },
            0xC9 => Instruction { opcode: Opcode::CMP, addressing_mode: AddressingMode::Immediate, cycles: 2 },
            0xCA => Instruction { opcode: Opcode::DEX, addressing_mode: AddressingMode::Implied, cycles: 2 },
            0xD0 => Instruction { opcode: Opcode::BNE, addressing_mode: AddressingMode::Relative, cycles: 2 },
            0xD8 => Instruction { opcode: Opcode::CLD, addressing_mode: AddressingMode::Implied, cycles: 2 },
            0xE0 => Instruction { opcode: Opcode::CPX, addressing_mode: AddressingMode::Immediate, cycles: 2 },
            0xE8 => Instruction { opcode: Opcode::INX, addressing_mode: AddressingMode::Implied, cycles: 2 },
            0xE9 => Instruction { opcode: Opcode::SBC, addressing_mode: AddressingMode::Immediate, cycles: 2 },
            0xEA => Instruction { opcode: Opcode::NOP, addressing_mode: AddressingMode::Implied, cycles: 2 },
            0xF0 => Instruction { opcode: Opcode::BEQ, addressing_mode: AddressingMode::Relative, cycles: 2 },
            0xF8 => Instruction { opcode: Opcode::SED, addressing_mode: AddressingMode::Implied, cycles: 2 },
            _ => Instruction { opcode: Opcode::Unknown, addressing_mode: AddressingMode::Implied, cycles: 2 },
        }
    }

    pub fn reset(&mut self) {
        self.accumulator = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.stack_pointer = 0xFD;
        self.status = 0x24;
        self.program_counter = 0x8000;
    }

    pub fn irq(&mut self) {
        
    }

    pub fn nmi(&mut self) {
        
    }
}