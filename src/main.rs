mod bus;
mod cpu;
mod rom;

use cpu::CPU;
use ppu::PPU;
use rom::Rom;
use std::fs::{self, File};
use std::io::Write;

fn main() {
    // Load the nestest ROM
    let rom_data = fs::read("nestest.nes").expect("Failed to read nestest.nes");
    let rom = Rom::new(&rom_data).expect("Failed to parse ROM");

    println!("ROM loaded successfully!");
    println!("PRG ROM size: {} bytes", rom.prg_rom.len());
    println!("CHR ROM size: {} bytes", rom.chr_rom.len());

    let mut cpu = CPU::new();
    cpu.load(&rom.prg_rom);
    cpu.reset();

    cpu.set_pc(0xC000);

    let mut log_file = File::create("my_nestest.log").expect("Failed to create log file");

    println!("Running nestest ROM...");

    let max_instructions = 10000; // Safety limit
    for _ in 0..max_instructions {
        let trace = cpu.trace();
        writeln!(log_file, "{}", trace).expect("Failed to write to log");

        cpu.step();
    }

    println!("Nestest execution complete!");
    println!("Output written to my_nestest.log");
    println!("Compare with: diff my_nestest.log nestest.log");
}
