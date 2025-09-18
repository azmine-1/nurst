mod bus; 
mod cpu; 

use bus::Bus;
use cpu::Cpu;
fn main() {
    println!("Hello, world!");
    let mut bus = Bus::new();
    let mut cpu = Cpu::new();

    let program = vec![0x00];
    bus.write(0x0000, 0x00);

    println!("Cpu initialized with PC: {} ", cpu.program_counter );
    println!("Bus initalized with {} bytes of memory", bus.ram.len());
}
