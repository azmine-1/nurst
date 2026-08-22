#[derive(Clone, Copy)]
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
    IndirectX,
    IndirectY,
}

impl AddressingMode {
    /// Total instruction length in bytes, opcode included.
    pub fn length(self) -> u16 {
        match self {
            AddressingMode::Implied | AddressingMode::Accumulator => 1,
            AddressingMode::Immediate
            | AddressingMode::ZeroPage
            | AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY
            | AddressingMode::Relative
            | AddressingMode::IndirectX
            | AddressingMode::IndirectY => 2,
            _ => 3,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    // Arithmetic
    ADC, SBC,

    // Logical
    AND, EOR, ORA,

    // Shifts & Rotates
    ASL, LSR, ROL, ROR,

    // Branches
    BCC, BCS, BEQ, BMI, BNE, BPL, BVC, BVS,

    // Status flag changes
    CLC, CLD, CLI, CLV, SEC, SED, SEI,

    // Compare
    CMP, CPX, CPY, BIT,

    // Increments & Decrements
    DEC, DEX, DEY, INC, INX, INY,

    // Jumps & Subroutines
    JMP, JSR, RTI, RTS, BRK,

    // Load/Store
    LDA, LDX, LDY, STA, STX, STY,

    // Register Transfers
    TAX, TAY, TSX, TXA, TXS, TYA,

    // Stack Operations
    PHA, PHP, PLA, PLP,

    NOP,

    // Unofficial opcodes. The first group is stable and used by real games;
    // the second is unstable and only shows up in test ROMs.
    LAX, SAX, DCP, ISB, SLO, RLA, SRE, RRA,
    ANC, ALR, ARR, AXS, XAA, LAS, AHX, SHY, SHX, TAS, JAM,
}

#[derive(Clone, Copy)]
pub struct Instruction {
    pub opcode: Opcode,
    pub addressing_mode: AddressingMode,
    pub cycles: u8,
    /// Instruction takes one extra cycle when indexing crosses a page boundary.
    pub page_penalty: bool,
    /// Unofficial opcode; traced with a leading `*` like nestest does.
    pub illegal: bool,
}
