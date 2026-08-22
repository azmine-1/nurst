//! End-to-end tests driven by the demo ROM, which exercises the PPU's
//! scrolling and sprite-0 split, OAM DMA, controller input and the APU.

use nurst::input::Button;
use nurst::ppu::WIDTH;
use nurst::Nes;

const ROM: &str = "demo/nurstrunner.nes";
const STATUS_BAR_ROW: usize = 12; // inside the fixed bar at the top
const PLAYFIELD_ROW: usize = 172; // the hill and bush band, which has detail to track
/// Columns holding the static "SCORE" label, away from the counting digits.
const LABEL_COLUMNS: std::ops::Range<usize> = 16..56;

fn load() -> Nes {
    let data = std::fs::read(ROM).expect("demo ROM (run tools/build_rom.py)");
    Nes::new(&data, 44100).expect("load demo ROM")
}

fn row(nes: &Nes, y: usize) -> Vec<u32> {
    nes.frame()[y * WIDTH..(y + 1) * WIDTH].to_vec()
}

fn run(nes: &mut Nes, frames: usize) {
    for _ in 0..frames {
        nes.step_frame();
    }
}

/// Hold Start for a few frames to leave the title screen.
fn start_game(nes: &mut Nes) {
    run(nes, 12);
    nes.set_button(0, Button::Start, true);
    run(nes, 3);
    nes.set_button(0, Button::Start, false);
    run(nes, 10);
}

#[test]
fn title_screen_renders_something() {
    let mut nes = load();
    run(&mut nes, 20);

    let distinct: std::collections::HashSet<u32> = nes.frame().iter().copied().collect();
    assert!(distinct.len() > 4, "expected a drawn screen, got {:?}", distinct);
}

#[test]
fn sprite_zero_split_holds_the_status_bar_still() {
    let mut nes = load();
    start_game(&mut nes);

    let bar_before = row(&nes, STATUS_BAR_ROW)[LABEL_COLUMNS].to_vec();
    let field_before = row(&nes, PLAYFIELD_ROW);
    run(&mut nes, 30);

    assert_eq!(
        row(&nes, STATUS_BAR_ROW)[LABEL_COLUMNS].to_vec(),
        bar_before,
        "the status bar above the split should not scroll"
    );
    assert_ne!(
        row(&nes, PLAYFIELD_ROW),
        field_before,
        "the playfield below the split should scroll"
    );
}

#[test]
fn the_run_scores_and_then_ends() {
    let mut nes = load();
    start_game(&mut nes);

    // The score counter lives in RAM at $20-$25, one decimal digit per byte.
    run(&mut nes, 40);
    let score: Vec<u8> = (0x20u16..0x26).map(|a| nes.cpu.bus.peek(a)).collect();
    assert!(score.iter().any(|&d| d != 0), "the score should climb while running");
    assert!(score.iter().all(|&d| d < 10), "digits stay in range: {:?}", score);

    // Standing still runs into the first cactus; $02 holds the game state.
    run(&mut nes, 600);
    assert_eq!(nes.cpu.bus.peek(0x02), 2, "the run should have ended in a crash");
}

#[test]
fn the_apu_produces_audio() {
    let mut nes = load();
    start_game(&mut nes);
    nes.drain_audio();

    run(&mut nes, 60);
    let samples = nes.drain_audio();

    assert!(
        (samples.len() as i64 - 44100).abs() < 2000,
        "expected about a second of audio, got {} samples",
        samples.len()
    );
    let peak = samples.iter().fold(0.0f32, |acc, s| acc.max(s.abs()));
    assert!(peak > 0.01, "the music should be audible, peak was {}", peak);
}
