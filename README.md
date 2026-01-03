# nurst

NES emulator written in Rust.

## Status

Currently implements:
- 6502 CPU with all official opcodes
- Basic memory bus and ROM loading
- Instruction trace output for debugging

Not implemented:
- PPU (graphics)
- APU (audio)
- Controller input
- Mappers beyond basic ROM

## Build

```bash
cargo build --release
```

## Usage

```bash
cargo run
```

## Testing

Compare CPU execution against nestest:

```bash
cargo run > my_nestest.log 2>&1
python compare_logs.py my_nestest.log nestest.log
```

## Structure

```
src/
├── main.rs          # Entry point
├── bus.rs           # Memory bus
├── rom.rs           # ROM/cartridge handling
└── cpu/
    ├── mod.rs       # CPU struct and public interface
    ├── types.rs     # Opcode/addressing mode enums
    ├── opcodes.rs   # Opcode decode table
    ├── execute.rs   # Instruction execution
    └── addressing.rs # Address mode resolution
```
