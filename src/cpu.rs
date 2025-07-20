pub struct CPU {
    pc: u16,  // program counter 
    sp: u16, // stack pointer 
    ac: u8,  // accumulator 
    x: u8,  // index register x 
    y: u8,  //index register y
    status: u8, //status flags

    memory: [u8; 65536 ], //64 kb of memory 
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
            memory: [0; 65536]
        }
    }

    pub fn reset(&mut self){
        self.pc = 0;
        self.sp = 0xFF; 
        self.ac = 0; 
        self.x = 0;
        self.y = 0;
        self.status = 0;
    }

    pub fn load(&mut self, program: &[u8], start_addr: usize){
        self.memory[start_addr..start_addr + program.len()]
            .copy_from_slice(program);
        self.pc = start_addr as u16;
    }

    pub fn fetch(&mut self) -> u8{
        let opcode = self.memory[self.pc as usize]; 
        self.pc += 1;
        opcode
    }
}