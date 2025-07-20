# 🕹️ NES Emulator in Rust

## 🎯 Objective
Build a cycle-accurate NES emulator written in Rust that can load `.nes` ROMs, emulate CPU, PPU, and APU behavior, and support controller inputs with a simple GUI.

---

## 🧩 Emulated Components

- **CPU**: Ricoh 2A03 (based on MOS 6502, no decimal mode)
- **Memory Map**: RAM, PPU/APU registers, PRG/CHR ROM/RAM
- **PPU**: Picture Processing Unit (tile-based graphics, scanline rendering)
- **APU**: Audio Processing Unit (basic audio channels)
- **Cartridge & Mappers**: iNES loader, NROM + MMC1/MMC3 support
- **Controller Input**: Standard NES controller

---

## ⚙️ Tech Stack

- **Language**: Rust
- **Graphics**: [`pixels`](https://crates.io/crates/pixels), [`minifb`](https://crates.io/crates/minifb), or [`winit`](https://crates.io/crates/winit)
- **Audio**: [`cpal`](https://crates.io/crates/cpal) or [`rodio`](https://crates.io/crates/rodio)
- **Input**: [`gilrs`](https://crates.io/crates/gilrs) or handled through `winit`
- **Testing ROMs**: [nestest.nes](https://www.qmtpro.com/~nes/nestest.log), Blargg’s test suite

---

## 📅 Project Roadmap

### ✅ Phase 1: Project Setup & CPU Emulation (2–3 weeks)

- Set up Rust crate and module structure
- Implement core 6502 CPU (instruction set, flags, cycle timing)
- Load `.nes` files and run `nestest.nes` for validation
- ✅ **Deliverable**: Headless CPU with instruction trace

---

### ✅ Phase 2: Memory Map & Bus (1 week)

- Emulate memory mirroring and address decoding
- Implement `MemoryBus` trait to map RAM, PPU, APU, cartridge
- Memory-mapped IO setup

---

### ✅ Phase 3: Cartridge Loader & Mapper (1 week)

- Implement iNES header parsing
- Add support for NROM (no bank switching)
- Define `Mapper` trait for extensible mapper implementations

---

### 🎨 Phase 4: PPU Implementation (3 weeks)

- Implement PPU registers, VRAM, palette, OAM
- Emulate scanline and frame rendering
- Display nametables and pattern tables
- ✅ **Deliverable**: Render background graphics

---

### 🕹️ Phase 5: Controller Input & Game Loop (1 week)

- Add emulation loop: CPU ↔ PPU ↔ APU synchronization
- Poll keyboard/controller input and forward to memory-mapped IO
- Load and play NROM games like Mario Bros. or Donkey Kong

---

### 🔊 Phase 6: APU Emulation (Optional, 2 weeks)

- Emulate square, triangle, noise, DMC channels
- Hook up timing to CPU cycles
- Stream audio via `cpal` or `rodio`

---

### 🧠 Phase 7: Advanced Mappers + Polishing (2 weeks)

- Implement MMC1/MMC3 (bank switching, scanline IRQs)
- Add scrolling, sprite 0 hit, and sprite overflow
- Save/load state support

---

### 🖼️ Phase 8: GUI & Packaging (1–2 weeks)

- Create windowed UI with FPS, pause/reset
- Add debug overlay (CPU state, PPU state)
- Package as cross-platform binary

---

## 📐 Design Goals

- ✅ Modular structure: separate CPU, PPU, APU, Mapper crates/modules
- ✅ Timing accuracy: sync all components per CPU cycle
- ✅ Robust testing: verified with nestest + Blargg test ROMs
- ✅ Safe and idiomatic Rust
- ✅ Performance: potential `no_std` core for embedded/WASM builds

---

## 💡 Stretch Goals

- 🌐 WebAssembly (WASM) build using `wasm-bindgen`
- 🔄 Rewind and save state snapshots
- 🐛 Interactive debugger (step-through, memory viewer)
- 🕸️ Netplay support for multiplayer games

---

## 🧪 Milestone Demos

| Week | Milestone                          | Deliverable                       |
|------|-----------------------------------|-----------------------------------|
| 3    | CPU executes nestest ROM          | Pass CPU test cases               |
| 5    | NROM game boots                   | Static screen rendered            |
| 7    | Input works                       | Playable Mario                    |
| 10   | Advanced mappers working          | Games like Zelda boot             |
| 12+  | Polished GUI + Audio              | Near-full NES experience          |

---