//! nestest-style CPU tracing, used to regression-test the 6502 core against a
//! known-good log.

use nurst::cpu::CPU;
use std::fs::File;
use std::io::{BufWriter, Write};

pub fn write_trace(
    cpu: &mut CPU,
    path: &str,
    start_pc: Option<u16>,
    lines: usize,
) -> Result<(), String> {
    // nestest's automated test entry point is $C000, which the reset vector
    // does not point at.
    if let Some(pc) = start_pc {
        cpu.set_pc(pc);
    }

    let file = File::create(path).map_err(|e| format!("cannot write {}: {}", path, e))?;
    let mut out = BufWriter::new(file);

    for _ in 0..lines {
        let line = cpu.trace();
        writeln!(out, "{}", line).map_err(|e| e.to_string())?;
        cpu.step();
        if cpu.jammed {
            break;
        }
    }

    out.flush().map_err(|e| e.to_string())?;
    println!("wrote {} ({} instructions)", path, lines);
    Ok(())
}
