mod bus;
mod cpu;
mod rom;

use cpu::CPU;
use rom::Rom;
use std::fs;

fn main() {
    // Load the nestest ROM
    let rom_data = fs::read("nestest.nes").expect("Failed to read nestest.nes");
    let rom = Rom::new(&rom_data).expect("Failed to parse ROM");

    println!("ROM loaded successfully!");
    println!("PRG ROM size: {} bytes", rom.prg_rom.len());
    println!("CHR ROM size: {} bytes", rom.chr_rom.len());
    println!("Mapper: {}", rom.mapper);
    println!("Mirroring: {:?}", rom.mirroring);

    // Create CPU and load the ROM
    let mut cpu = CPU::new();
    cpu.load(&rom.prg_rom);
    cpu.reset();

    println!("\nInitial CPU state: {}", cpu);

    // Run a few instructions
    for i in 0..10 {
        println!("Step {}: {}", i, cpu);
        cpu.step();
    }
}
