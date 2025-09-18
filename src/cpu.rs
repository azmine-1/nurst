pub struct Cpu {
    pub register_a: u8, 
    pub status: u8, 
    pub program_counter: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            register_a: 0, 
            status: 0, 
            program_counter: 0,
        }
    }

    pub fn interpret(&mut self, program: &[u8]){
        let opscode = program[self.program_counter as usize];
        self.program_counter += 1;
        match opscode {
            _ => todo! ()
        }
    }
}