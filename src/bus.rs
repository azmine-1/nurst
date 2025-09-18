pub struct Bus { 
    pub address: u16, 
    pub bReadOnly: bool, 
    pub ram: [u8; 10 * 1024]
}

impl Bus { 
    pub fn new() -> Self {
        Self { 
            address: 0, 
            bReadOnly: false, 
            ram: [0; 10 * 1024]
        }
    }
    
    pub fn write(&mut self, addr: u16, data: u8){
        if addr >= 0x0000 && addr <= 0xFFFF {
            self.ram[addr as usize] = data; 
        }

    }

    pub fn read(&self, addr: u16) -> u8 {
        if addr < self.ram.len() as u16 {
            return self.ram[addr as usize];
        }
        0 
    }

}