mod cpu;
mod memory;
mod bus;

use cpu::CPU;
use bus::Bus;
use std::fs;

fn main() {
    println!("Starting NURST");

    let data = fs::read("roms/nestest.nes").expect("Failed to read ROM");
    let rom = data[16..].to_vec(); // skip 16-byte iNES header

    let mut bus = Bus::new(rom);
    let mut cpu = CPU::new();
    cpu.reset(&bus);

    for _ in 0..10 {
        cpu.step(&mut bus);
    }
}
