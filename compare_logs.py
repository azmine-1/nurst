#!/usr/bin/env python3
"""
NES Test Log Comparison Script

Compares nestest.log with my_nestest.log, ignoring PPU information and
focusing on CPU state (PC, registers, flags, stack pointer).
"""

import sys
import re
from typing import Optional, Tuple

class CPUState:
    def __init__(self, line: str):
        self.raw_line = line.strip()
        self.parse(line)

    def parse(self, line: str):
        """Parse a log line to extract CPU state"""
        # Format: C000  4C F5 C5  JMP $C5F5                       A:00 X:00 Y:00 P:24 SP:FD CYC:7
        # Or:     C000  4C F5 C5  JMP $C5F5                       A:00 X:00 Y:00 P:24 SP:FD PPU:  0,  0 CYC:7

        parts = line.strip()
        if not parts:
            self.valid = False
            return

        try:
            # Extract PC (first 4 hex digits)
            self.pc = parts[0:4]

            # Extract instruction bytes (next section)
            bytes_match = re.search(r'^[0-9A-F]{4}\s+([0-9A-F ]{8})', parts)
            self.instr_bytes = bytes_match.group(1).strip() if bytes_match else ""

            # Extract disassembly (between bytes and registers)
            disasm_match = re.search(r'^[0-9A-F]{4}\s+[0-9A-F ]{8}\s+([^A]*?)(?=A:[0-9A-F]{2})', parts)
            self.disasm = disasm_match.group(1).strip() if disasm_match else ""

            # Extract registers
            a_match = re.search(r'A:([0-9A-F]{2})', parts)
            x_match = re.search(r'X:([0-9A-F]{2})', parts)
            y_match = re.search(r'Y:([0-9A-F]{2})', parts)
            p_match = re.search(r'P:([0-9A-F]{2})', parts)
            sp_match = re.search(r'SP:([0-9A-F]{2})', parts)
            cyc_match = re.search(r'CYC:(\d+)', parts)

            self.a = a_match.group(1) if a_match else "??"
            self.x = x_match.group(1) if x_match else "??"
            self.y = y_match.group(1) if y_match else "??"
            self.p = p_match.group(1) if p_match else "??"
            self.sp = sp_match.group(1) if sp_match else "??"
            self.cyc = cyc_match.group(1) if cyc_match else "?"

            self.valid = True
        except Exception as e:
            self.valid = False
            print(f"Parse error: {e} on line: {line[:50]}", file=sys.stderr)

    def compare(self, other: 'CPUState') -> Tuple[bool, list]:
        """Compare two CPU states and return (matches, differences)"""
        if not self.valid or not other.valid:
            return False, ["Invalid state"]

        diffs = []

        if self.pc != other.pc:
            diffs.append(f"PC: {self.pc} != {other.pc}")

        if self.instr_bytes != other.instr_bytes:
            diffs.append(f"Bytes: {self.instr_bytes} != {other.instr_bytes}")

        # Normalize disassembly for comparison (remove extra spaces)
        self_disasm = ' '.join(self.disasm.split())
        other_disasm = ' '.join(other.disasm.split())
        if self_disasm != other_disasm:
            diffs.append(f"Disasm: '{self_disasm}' != '{other_disasm}'")

        if self.a != other.a:
            diffs.append(f"A: {self.a} != {other.a}")

        if self.x != other.x:
            diffs.append(f"X: {self.x} != {other.x}")

        if self.y != other.y:
            diffs.append(f"Y: {self.y} != {other.y}")

        if self.p != other.p:
            diffs.append(f"P: {self.p} != {other.p}")

        if self.sp != other.sp:
            diffs.append(f"SP: {self.sp} != {other.sp}")

        # Optional: compare cycles (might differ due to PPU implementation)
        # if self.cyc != other.cyc:
        #     diffs.append(f"CYC: {self.cyc} != {other.cyc}")

        return len(diffs) == 0, diffs

    def __str__(self):
        return self.raw_line


def compare_logs(reference_path: str, test_path: str, context_lines: int = 3, max_errors: int = 10):
    """Compare two NES test logs"""

    try:
        with open(reference_path, 'r') as f:
            reference_lines = f.readlines()
    except FileNotFoundError:
        print(f"Error: Could not find reference file: {reference_path}")
        sys.exit(1)

    try:
        with open(test_path, 'r') as f:
            test_lines = f.readlines()
    except FileNotFoundError:
        print(f"Error: Could not find test file: {test_path}")
        sys.exit(1)

    print(f"Comparing logs:")
    print(f"  Reference: {reference_path} ({len(reference_lines)} lines)")
    print(f"  Test:      {test_path} ({len(test_lines)} lines)")
    print(f"{'='*80}\n")

    errors_found = 0
    lines_compared = 0

    for i, (ref_line, test_line) in enumerate(zip(reference_lines, test_lines)):
        ref_state = CPUState(ref_line)
        test_state = CPUState(test_line)

        if not ref_state.valid or not test_state.valid:
            continue

        lines_compared += 1
        matches, diffs = ref_state.compare(test_state)

        if not matches:
            errors_found += 1

            # Print context
            print(f"Difference at line {i+1} (instruction #{lines_compared}):")
            print(f"{'-'*80}")

            # Show context before
            start = max(0, i - context_lines)
            if start < i:
                print(f"Context (lines {start+1}-{i}):")
                for j in range(start, i):
                    if j < len(reference_lines):
                        print(f"  {reference_lines[j].rstrip()}")
                print()

            # Show the differing lines
            print(f"Expected (nestest.log line {i+1}):")
            print(f"  {ref_state}")
            print()
            print(f"Got (my_nestest.log line {i+1}):")
            print(f"  {test_state}")
            print()
            print(f"Differences:")
            for diff in diffs:
                print(f"  - {diff}")
            print(f"{'-'*80}\n")

            if errors_found >= max_errors:
                print(f"Stopping after {max_errors} errors.")
                print(f"Set max_errors higher to see more differences.\n")
                break

    # Check for length mismatch
    if len(reference_lines) != len(test_lines):
        print(f"Warning: Log length mismatch!")
        print(f"  Reference: {len(reference_lines)} lines")
        print(f"  Test:      {len(test_lines)} lines")
        print()

    # Summary
    print(f"{'='*80}")
    print(f"Summary:")
    print(f"  Lines compared: {lines_compared}")
    print(f"  Errors found:   {errors_found}")

    if errors_found == 0:
        print(f"\n✓ All lines match! Your emulator is running correctly!")
        return 0
    else:
        print(f"\n✗ Found {errors_found} difference(s)")
        return 1


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python compare_logs.py [my_nestest.log] [reference_nestest.log] [max_errors]")
        print()
        print("Defaults:")
        print("  my_nestest.log: ./my_nestest.log")
        print("  reference:      ./nestest.log")
        print("  max_errors:     10")
        sys.exit(1)

    test_log = sys.argv[1] if len(sys.argv) > 1 else "my_nestest.log"
    ref_log = sys.argv[2] if len(sys.argv) > 2 else "nestest.log"
    max_errors = int(sys.argv[3]) if len(sys.argv) > 3 else 10

    sys.exit(compare_logs(ref_log, test_log, context_lines=3, max_errors=max_errors))
