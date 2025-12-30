mod bus;
mod cpu;

use bus::Bus;
use cpu::CPU;
fn main() {
    println!("Hello, world!");
    let mut bus = Bus::new();
    let mut cpu = CPU::new();

    let program = vec![0x00];
}
