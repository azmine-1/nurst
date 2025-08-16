use crate::memory::Memory;

pub struct CPU {
    pub pc: u16,  // program counter 
    pub sp: u16, // stack pointer 
    pub ac: u8,  // accumulator 
    pub x: u8,  // index register x 
    pub y: u8,  //index register y
    pub status: u8, //status flags
    pub cycles: usize,
}

pub enum Op {
    // Load/Store Operations
    LDA,  // Load Accumulator
    LDX,  // Load X Register
    LDY,  // Load Y Register
    STA,  // Store Accumulator
    STX,  // Store X Register
    STY,  // Store Y Register
    
    // Arithmetic Operations
    ADC,  // Add with Carry
    SBC,  // Subtract with Carry
    
    // Increment/Decrement Operations
    INC,  // Increment Memory
    INX,  // Increment X Register
    INY,  // Increment Y Register
    DEC,  // Decrement Memory
    DEX,  // Decrement X Register
    DEY,  // Decrement Y Register
    
    // Logical Operations
    AND,  // Logical AND
    ORA,  // Logical OR (Inclusive)
    EOR,  // Exclusive OR
    
    // Shift and Rotate Operations
    ASL,  // Arithmetic Shift Left
    LSR,  // Logical Shift Right
    ROL,  // Rotate Left
    ROR,  // Rotate Right
    
    // Branch Operations
    BCC,  // Branch if Carry Clear
    BCS,  // Branch if Carry Set
    BEQ,  // Branch if Equal (Zero Set)
    BMI,  // Branch if Minus (Negative Set)
    BNE,  // Branch if Not Equal (Zero Clear)
    BPL,  // Branch if Plus (Negative Clear)
    BVC,  // Branch if Overflow Clear
    BVS,  // Branch if Overflow Set
    
    // Jump and Subroutine Operations
    JMP,  // Jump
    JSR,  // Jump to Subroutine
    RTS,  // Return from Subroutine
    RTI,  // Return from Interrupt
    
    // Compare Operations
    CMP,  // Compare Accumulator
    CPX,  // Compare X Register
    CPY,  // Compare Y Register
    
    // Bit Operations
    BIT,  // Bit Test
    
    // Transfer Operations
    TAX,  // Transfer A to X
    TAY,  // Transfer A to Y
    TXA,  // Transfer X to A
    TYA,  // Transfer Y to A
    TSX,  // Transfer Stack Pointer to X
    TXS,  // Transfer X to Stack Pointer
    
    // Stack Operations
    PHA,  // Push Accumulator
    PHP,  // Push Processor Status
    PLA,  // Pull Accumulator
    PLP,  // Pull Processor Status
    
    // Status Flag Operations
    CLC,  // Clear Carry Flag
    CLD,  // Clear Decimal Flag
    CLI,  // Clear Interrupt Disable Flag
    CLV,  // Clear Overflow Flag
    SEC,  // Set Carry Flag
    SED,  // Set Decimal Flag
    SEI,  // Set Interrupt Disable Flag
    
    // System Operations
    BRK,  // Break (Software Interrupt)
    NOP,  // No Operation
    
    // Illegal/Unofficial Opcodes (commonly used)
    LAX,  // Load A and X
    SAX,  // Store A AND X
    DCP,  // Decrement and Compare
    ISC,  // Increment and Subtract with Carry
    SLO,  // Shift Left and OR
    RLA,  // Rotate Left and AND
    SRE,  // Shift Right and EOR
    RRA,  // Rotate Right and Add
    ANC,  // AND and set Carry
    ALR,  // AND and Logical Shift Right
    ARR,  // AND and Rotate Right
    XAA,  // Transfer X to A and AND
    AXS,  // AND X with A and Subtract
    AHX,  // AND A with X and Store
    SHY,  // Store Y AND (high byte of address + 1)
    SHX,  // Store X AND (high byte of address + 1)
    TAS,  // Transfer A AND X to Stack Pointer
    LAS,  // Load A, X, and Stack Pointer
}

impl CPU {
    pub fn new() -> Self {
        CPU{
            pc: 0, 
            sp: 0xFF, 
            ac: 0, 
            x: 0, 
            y: 0, 
            status: 0, 
            cycles: 0
        }
    }

    pub fn reset(&mut self, _mem: &impl Memory) {
        self.pc = 0xC000;
        self.sp = 0xFD;
        self.status = 0x24;
        self.ac = 0;
        self.x = 0;
        self.y = 0;
        self.cycles = 7;
    }

    pub fn fetch(&mut self, memory: &impl Memory) -> u8 {
        let byte = memory.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    pub fn decode(&mut self, opcode: u8, mem: &mut impl Memory){
        match opcode {
            0xA9 => {
                let value = self.fetch(mem);
                self.ac = value; 
                self.pc += 1;
            }
        }
    }
    pub fn tick(&mut self, mem: &mut impl Memory) {
        let opcode = self.fetch(mem);
        self.execute(opcode, mem);
    }

    pub fn execute(&mut self, opcode: u8, mem: &mut impl Memory) {
        match opcode {
            0xA9 => { // LDA immediate
                let value = self.fetch(mem);
                self.ac = value;
                self.update_zn_flags(self.ac);
                self.cycles += 2;
            }
            _ => panic!("Unhandled opcode: {:02X} at PC: {:04X}", opcode, self.pc - 1),
        }
    }

    fn update_zn_flags(&mut self, value: u8) {
        if value == 0 {
            self.status |= 0b0000_0010; // set zero flag
        } else {
            self.status &= 0b1111_1101; // clear zero flag
        }

        if value & 0x80 != 0 {
            self.status |= 0b1000_0000; // set negative flag
        } else {
            self.status &= 0b0111_1111; // clear negative flag
        }
    }

    pub fn trace(&self, memory: &impl Memory) {
        let pc = self.pc;
        let opcode = memory.read(pc);
        let operand1 = memory.read(pc.wrapping_add(1));
        let operand2 = memory.read(pc.wrapping_add(2));
    
        print!("{:04X}  {:02X} {:02X} {:02X}   ", pc, opcode, operand1, operand2);
    
        print!("A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} ",
            self.ac, self.x, self.y, self.status, self.sp);
    
        // fake values to for nestest.nes
        print!("PPU:{:>3},{:>3} ", 0, 0);
    
        println!("CYC:{}", self.cycles);
    }

    pub fn step(&mut self, memory: &mut impl Memory) {
        self.trace(memory);
        self.tick(memory);
    }
}