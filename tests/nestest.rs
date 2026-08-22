//! Runs the nestest ROM in its automated mode and compares every trace line
//! with the reference log. The log covers all official opcodes and the stable
//! unofficial ones, plus per-instruction cycle and PPU-dot counts.

use nurst::cpu::CPU;
use nurst::mapper;
use nurst::rom::Rom;

const TRACE_LINES: usize = 8991;

#[test]
fn cpu_matches_the_nestest_log() {
    let rom_data = std::fs::read("nestest.nes").expect("nestest.nes");
    let reference = std::fs::read_to_string("nestest.log").expect("nestest.log");
    let rom = Rom::new(&rom_data).expect("parse nestest.nes");

    let mut cpu = CPU::new(mapper::from_rom(rom).expect("mapper"), 44100);
    cpu.reset();
    cpu.set_pc(0xC000); // nestest's automated entry point

    for (index, expected) in reference.lines().take(TRACE_LINES).enumerate() {
        let actual = cpu.trace();
        assert_eq!(
            actual,
            expected,
            "\ntrace diverged on instruction {}\n  expected: {}\n  actual:   {}",
            index + 1,
            expected,
            actual
        );
        cpu.step();
    }
}
