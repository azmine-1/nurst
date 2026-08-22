# Testing nurst

## The automated suite

```bash
cargo test --release
```

That runs, in order of how much they tell you:

| Test | What it proves |
| --- | --- |
| `tests/nestest.rs` | Every one of 8991 trace lines from the nestest ROM matches the reference log **byte for byte** — registers, flags, stack pointer, cycle count and PPU dot. |
| `tests/test_roms.rs` | blargg's `instr_test-v5` (official *and* unofficial opcodes) and the MMC3 IRQ clocking test report success. |
| `tests/demo_rom.rs` | The demo ROM boots, the sprite 0 split holds the status bar still while the playfield scrolls, the run scores and ends on a collision, and the APU emits a second of audible samples per second of play. |
| unit tests | iNES parsing, opcode table coverage. |

`tests/test_roms.rs` needs ROMs that are not committed here. Fetch them first;
without them those tests print a skip notice instead of failing:

```bash
./tools/fetch_roms.sh
```

## Checks that still need eyes

Four of blargg's ROMs predate the `$6000` result protocol and only print to the
screen, so they are not in the automated suite:

```bash
cargo run --release -- roms/tests/cpu_timing_test6-cpu_timing_test.nes \
    --headless --frames 700 --screenshot /tmp/timing.png
```

| ROM | Expected |
| --- | --- |
| `cpu_timing_test6-cpu_timing_test.nes` | `6502 TIMING TEST … PASSED` |
| `sprite_hit_tests_2005.10.05-01.basics.nes` | `SPRITE HIT BASICS PASSED` |
| `blargg_ppu_tests_2005.09.15b-palette_ram.nes` | `$01` (code 1 is a pass; failures start at 2) |
| `blargg_ppu_tests_2005.09.15b-vram_access.nes` | `$01` |

All four pass as of this writing.

## Known gap

`ppu_vbl_nmi` fails its first subtest and is marked `#[ignore]`. The PPU is
advanced one *bus access* at a time rather than one CPU cycle at a time, so a
read of `$2002` lands within a cycle or two of where hardware would put it.
Games do not notice; this test measures the vblank flag to the exact cycle.
Closing it means ticking the PPU inside the CPU's internal cycles too — every
dummy read and every idle cycle — which is a real change to `CPU::step` and
`Bus::read`/`Bus::write`.

## Comparing a CPU trace by hand

The trace format is nestest's, so any divergence can be diffed directly:

```bash
cargo run --release -- nestest.nes --trace my_nestest.log \
    --trace-start C000 --trace-lines 8991
diff my_nestest.log nestest.log
python3 compare_logs.py my_nestest.log nestest.log   # ignores the PPU column
```

# When a ROM misbehaves

This is the loop to work through, cheapest step first.

## 1. Reproduce it without a window

Every run is deterministic, so a headless run with a scripted controller is
reproducible and diffable:

```bash
cargo run --release -- roms/games/thegame.nes --headless --frames 400 \
    --input "60:start;64:" --screenshot /tmp/shot.png
```

`--input` takes `frame:buttons` checkpoints separated by `;`. The named buttons
are held from that frame until the next checkpoint, so `90:a;92:` taps A. Button
names are `a b select start up down left right`.

Note that title screens often want a button held for more than a couple of
frames before they react — if `60:start;62:` does nothing, try holding it for
thirty.

## 2. Read the screenshot

Sort the symptom into one of these, because each points at a different file:

| Symptom | Where to look |
| --- | --- |
| Black screen, no output at all | `src/mapper/` — wrong bank, so the reset vector points at garbage |
| Right tiles, wrong colours | `src/ppu/mod.rs`, palette handling and attribute fetch |
| Tiles are garbage or from the wrong bank | `src/mapper/` CHR banking |
| Picture is offset, torn, or wobbles | `src/ppu/mod.rs`, the loopy scroll registers and `copy_horizontal` / `copy_vertical` |
| Status bar scrolls with the playfield | sprite 0 hit, in `PPU::render_pixel` |
| Sprites missing or flickering wrongly | `PPU::evaluate_sprites`, or OAM DMA in `src/bus.rs` |
| Game hangs on a loading screen | usually a missing IRQ — `Mapper::irq_pending` and `CPU::step`'s IRQ poll |
| Silence, or one channel wrong | `src/apu/channels.rs` |

## 3. Check the mapper first

It is the most common cause and the cheapest to rule out. The banner printed at
startup names the mapper:

```
roms/games/nomolos.nes: mapper 2, 512 KB PRG, 8 KB CHR RAM, Vertical mirroring
```

If the mapper is unsupported, nurst says so and exits. Adding one is usually
under a hundred lines: copy `src/mapper/cnrom.rs`, implement the five required
`Mapper` methods, and register it in the match in `src/mapper/mod.rs`. The bank
arithmetic is already done for you by `Banks::prg` and `Banks::chr`, which take
a bank number and a window size and fold out-of-range banks automatically.

## 4. Trace the CPU

If the game is running the wrong code rather than drawing it wrongly:

```bash
cargo run --release -- roms/games/thegame.nes --trace /tmp/trace.log --trace-lines 200000
tail -50 /tmp/trace.log
```

A trace that ends up spinning on three or four lines is a game waiting for
something that never arrives — an IRQ, a vblank flag, or a controller read.
Look at what address the loop reads and work backwards from there.

## 5. Read the machine's state from a test

For anything finer, drive the emulator from a throwaway integration test, where
the whole `Nes` is in reach. Put this in `tests/probe.rs` and run
`cargo test --release --test probe -- --nocapture`:

```rust
#[test]
fn probe() {
    let data = std::fs::read("roms/games/thegame.nes").unwrap();
    let mut nes = nurst::Nes::new(&data, 44100).unwrap();
    for frame in 0..600 {
        nes.step_frame();
        if frame % 60 == 0 {
            // Any CPU-visible address; peek has no side effects.
            println!("frame {} pc={:04X} $00={:02X}",
                     frame, nes.cpu.pc(), nes.cpu.bus.peek(0x0000));
        }
    }
    // What the ROM printed, if it uses ASCII-numbered tiles as blargg's do.
    println!("{}", nes.screen_text());
}
```

Useful handles:

- `nes.cpu.bus.peek(addr)` — read CPU memory without side effects
- `nes.cpu.pc()`, `nes.cpu.cycles`, `nes.cpu.trace()` — CPU state
- `nes.cpu.bus.ppu.position()` — current scanline and dot
- `nes.frame()` / `nes.pixel(x, y)` — the rendered framebuffer
- `nes.screen_text()` — decode the first nametable as text

## 6. Prove the fix

Add a case to `tests/demo_rom.rs` or `tests/test_roms.rs` so the bug cannot come
back, then re-run the whole suite. `tests/nestest.rs` is the tripwire for the
CPU: if a change breaks it, the change is wrong.
