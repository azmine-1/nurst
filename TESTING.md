# CPU Testing Guide

## Quick Start

```bash
# Build and run nestest
cargo build --release
./target/release/nurst

# Compare your output with expected
./compare_logs.sh

# Or manually
diff my_nestest.log nestest.log | head -20
```

## What nestest Tests

The nestest ROM tests all 6502 CPU instructions by executing them and logging CPU state after each instruction. Your output should match `nestest.log` exactly.

## Log Format

Each line shows:
```
PC  BYTES  INSTRUCTION                      A:XX X:XX Y:XX P:XX SP:XX CYC:XXX
C000  4C F5 C5  JMP $C5F5                   A:00 X:00 Y:00 P:24 SP:FD CYC:7
```

- **PC**: Program counter (where instruction is)
- **BYTES**: Raw instruction bytes (1-3 bytes)
- **INSTRUCTION**: Disassembled instruction
- **Registers**: A, X, Y, P (status), SP
- **CYC**: Total CPU cycles executed

## Current Issues Found

1. **ROM mirroring**: nestest.nes is 16KB but needs to mirror across 0x8000-0xFFFF
   - 0x8000-0xBFFF: ROM data
   - 0xC000-0xFFFF: Should mirror 0x8000-0xBFFF
   - Fix: `src/bus.rs:50` - Handle mirroring in mem_read()

2. **First instruction**: Should read JMP at 0xC000, currently reading 0x00 (BRK)

## Debugging Tips

**Find first difference:**
```bash
./compare_logs.sh
```

**Check specific addresses:**
```rust
// Add debug prints in bus.rs mem_read() to see what's being read
println!("Reading addr {:04X} = {:02X}", addr, value);
```

**Common issues:**
- Missing opcodes in `decode()` → Returns `Unknown` opcode
- Wrong addressing mode → Reads wrong memory location
- Flags not set correctly → Status register (P) wrong
- Cycle count wrong → CYC value differs

## Expected Flow

1. PC starts at 0xC000 (set by `cpu.set_pc()`)
2. First instruction: `4C F5 C5` = JMP $C5F5
3. Should execute ~8000 instructions before completion
4. Each line should match nestest.log exactly

## Debug Workflow

1. Find first divergence line in logs
2. Check what opcode is being executed
3. Verify opcode is in `decode()` table
4. Check addressing mode calculates correct address
5. Verify instruction modifies registers/flags correctly
6. Check cycle count matches expected

## Quick Fixes Needed

- [ ] Fix ROM mirroring in `src/bus.rs` line 50-52
- [ ] Ensure all opcodes in nestest are in decode table
- [ ] Verify flag updates (especially N, Z, C, V)
