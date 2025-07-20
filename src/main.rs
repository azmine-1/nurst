mod cpu;

use cpu::CPU;

fn main() {
    let mut cpu = CPU::new(); 

    cpu.load(&[0xA9, 0x01, 0xA9, 0x95], 0x8000); 

    println!("Firt opcode: 0x{:02x}", cpu.fetch()); 
    println!("Second opcode: 0x{:02x}", cpu.fetch());
    
}
