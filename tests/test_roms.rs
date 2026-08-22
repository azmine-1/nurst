//! Runs the nesdev test ROM suite. These ROMs report their result through
//! cartridge RAM: $6001-$6003 hold a magic signature, $6000 is the status
//! (0x80 while running) and $6004 onwards is a NUL-terminated message.
//!
//! The ROMs are not redistributed here; run `tools/fetch_roms.sh` first. The
//! tests skip themselves when the files are missing.

use nurst::Nes;

const DIRECTORY: &str = "roms/tests";
const MAX_FRAMES: usize = 3000;

struct Outcome {
    status: u8,
    message: String,
}

fn run_test_rom(name: &str) -> Option<Outcome> {
    let path = format!("{}/{}", DIRECTORY, name);
    let data = std::fs::read(&path).ok()?;
    let mut nes = Nes::new(&data, 44100).expect("load test ROM");

    let mut started = false;
    for _ in 0..MAX_FRAMES {
        nes.step_frame();

        let signature: Vec<u8> = (0x6001u16..0x6004).map(|a| nes.cpu.bus.peek(a)).collect();
        if signature != [0xDE, 0xB0, 0x61] {
            continue;
        }
        let status = nes.cpu.bus.peek(0x6000);
        if status == 0x80 {
            started = true;
            continue;
        }
        if started {
            let mut message = String::new();
            for offset in 0..256u16 {
                match nes.cpu.bus.peek(0x6004 + offset) {
                    0 => break,
                    byte => message.push(byte as char),
                }
            }
            return Some(Outcome { status, message: message.trim().to_string() });
        }
    }
    Some(Outcome { status: 0xFF, message: "timed out".to_string() })
}

fn expect_pass(name: &str) {
    match run_test_rom(name) {
        None => eprintln!("skipping {}: run tools/fetch_roms.sh to download it", name),
        Some(outcome) => assert_eq!(
            outcome.status, 0,
            "{} failed with status {}: {}",
            name, outcome.status, outcome.message
        ),
    }
}

#[test]
fn official_instructions() {
    expect_pass("instr_test-v5-official_only.nes");
}

#[test]
fn all_instructions_including_unofficial() {
    expect_pass("instr_test-v5-all_instrs.nes");
}

#[test]
fn mmc3_irq_clocking() {
    expect_pass("rom_singles-1-clocking.nes");
}

/// Known gap. The PPU is stepped one bus access at a time rather than one CPU
/// cycle at a time, so a $2002 read lands within a cycle or two of where real
/// hardware would put it. That is close enough for games but not for this
/// test, which measures the vblank flag to the exact cycle.
#[test]
#[ignore = "needs cycle-exact CPU/PPU interleaving; see TESTING.md"]
fn ppu_vblank_and_nmi() {
    expect_pass("ppu_vbl_nmi-ppu_vbl_nmi.nes");
}

// The 2005-era blargg PPU ROMs and cpu_timing_test6 predate the $6000
// protocol and only print their result on screen, so they are checked by eye.
// TESTING.md lists the commands and the expected output.
