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