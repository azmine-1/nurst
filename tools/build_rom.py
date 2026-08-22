#!/usr/bin/env python3
"""Build demo/nurstrunner.nes from demo/runner.asm.

The background map, the APU note table and the music patterns are generated
here and appended to the hand-written source before assembly.
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import asm6502
import tiles

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SOURCE = os.path.join(ROOT, "demo", "runner.asm")
GENERATED = os.path.join(ROOT, "demo", "generated.asm")
OUTPUT = os.path.join(ROOT, "demo", "nurstrunner.nes")

# Background tile numbers, matching tools/tiles.py.
BLANK, SKY, STAR, CLOUD_L, CLOUD_R = 0x00, 0x01, 0x02, 0x03, 0x04
GRASS, DIRT, DIRT_ROCK = 0x05, 0x06, 0x07
BUSH_L, BUSH_R = 0x08, 0x09
HILL_L, HILL_TOP, HILL_R, SOLID, BAR = 0x0A, 0x0B, 0x0C, 0x0D, 0x0E
HILL_BODY = 0x0F

COLUMNS = 64  # two nametables side by side
ROWS = 30


def build_map():
    """Lay out the 64-column looping background."""
    grid = [[SKY] * COLUMNS for _ in range(ROWS)]

    # Status bar: three blank rows and a divider the sprite 0 hit can land on.
    for row in range(3):
        grid[row] = [BLANK] * COLUMNS
    grid[3] = [BAR] * COLUMNS

    # Static labels, only ever seen in the first nametable at scroll zero.
    for offset, char in enumerate("SCORE"):
        grid[1][2 + offset] = ord(char)
    for offset, char in enumerate("HI"):
        grid[1][18 + offset] = ord(char)

    # Ground.
    grid[22] = [GRASS] * COLUMNS
    for row in range(23, ROWS):
        grid[row] = [DIRT] * COLUMNS

    # A deterministic scatter keeps the 512-pixel loop seamless: nothing is
    # placed in the last few columns, where a decoration would be cut in half.
    state = 0x1F35

    def next_random():
        nonlocal state
        state = (state * 1103515245 + 12345) & 0xFFFFFFFF
        return (state >> 16) & 0xFFFF

    for row in range(24, ROWS):
        for col in range(COLUMNS):
            if next_random() % 9 == 0:
                grid[row][col] = DIRT_ROCK

    # Clouds and stars live in the palette-0 rows.
    col = 3
    while col < COLUMNS - 6:
        row = 5 + next_random() % 7
        grid[row][col] = CLOUD_L
        grid[row][col + 1] = CLOUD_R
        col += 5 + next_random() % 7
    for _ in range(24):
        row = 4 + next_random() % 12
        col = next_random() % (COLUMNS - 4)
        if grid[row][col] == SKY:
            grid[row][col] = STAR

    # Hills and bushes share the palette-2 band just above the ground.
    col = 2
    while col < COLUMNS - 8:
        width = 2 + next_random() % 4
        grid[20][col] = HILL_L
        for offset in range(1, width + 1):
            grid[20][col + offset] = HILL_TOP
        grid[20][col + width + 1] = HILL_R
        for offset in range(width + 2):
            grid[21][col + offset] = HILL_BODY
        col += width + 4 + next_random() % 6

    col = 1
    while col < COLUMNS - 4:
        if grid[21][col] == SKY and grid[21][col + 1] == SKY:
            grid[21][col] = BUSH_L
            grid[21][col + 1] = BUSH_R
        col += 5 + next_random() % 8

    return grid


def build_attributes(grid, nametable):
    """One attribute byte covers 4x4 tiles as four 2x2-tile quadrants."""
    del grid  # the palette bands are fixed, not derived from the tiles
    attributes = []
    for attr_row in range(8):
        for _ in range(8):
            top_rows = attr_row * 4
            if top_rows == 0:
                quadrants = (3, 3, 3, 3)          # status bar
            elif top_rows < 16:
                quadrants = (0, 0, 0, 0)          # sky
            elif top_rows == 16:
                quadrants = (0, 0, 2, 2)          # sky above the hill band
            elif top_rows == 20:
                quadrants = (2, 2, 1, 1)          # hills above the ground
            else:
                quadrants = (1, 1, 1, 1)          # ground
            top_left, top_right, bottom_left, bottom_right = quadrants
            attributes.append(
                top_left | (top_right << 2) | (bottom_left << 4) | (bottom_right << 6)
            )
    del nametable
    return attributes


def build_nametables():
    grid = build_map()
    data = []
    for nametable in range(2):
        for row in range(ROWS):
            data.extend(grid[row][nametable * 32:(nametable + 1) * 32])
        data.extend(build_attributes(grid, nametable))
    assert len(data) == 2048, len(data)
    return data


# ------------------------------------------------------------------- audio

NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"]
CPU_HZ = 1789773.0


def note_frequency(name):
    pitch, octave = name[:-1], int(name[-1])
    semitone = NOTE_NAMES.index(pitch) + 12 * octave
    a4 = NOTE_NAMES.index("A") + 12 * 4
    return 440.0 * 2 ** ((semitone - a4) / 12.0)


# Index 0 is a rest, so the table starts at index 1.
NOTE_TABLE = [
    "%s%d" % (pitch, octave) for octave in range(2, 6) for pitch in NOTE_NAMES
]
NOTE_INDEX = {name: index + 1 for index, name in enumerate(NOTE_TABLE)}


def note_period(name):
    return int(round(CPU_HZ / (16.0 * note_frequency(name)))) - 1


# The triangle reads the same table an octave lower, which is what makes these
# lines a bass part.
MUSIC_LEAD = [
    "A4", 0, "C5", 0, "E5", 0, "D5", 0,
    "C5", 0, "A4", 0, "E4", 0, 0, 0,
    "G4", 0, "B4", 0, "D5", 0, "C5", 0,
    "B4", 0, "G4", 0, "D4", 0, 0, 0,
]
MUSIC_BASS = [
    "A3", 0, 0, 0, "A3", 0, "E4", 0,
    "A3", 0, 0, 0, "C4", 0, 0, 0,
    "G3", 0, 0, 0, "G3", 0, "D4", 0,
    "G3", 0, 0, 0, "B3", 0, 0, 0,
]


def pattern_bytes(pattern):
    return [0 if note == 0 else NOTE_INDEX[note] for note in pattern]


# ------------------------------------------------------------------- output


def rows_of(values, per_row=16, formatter="${:02X}"):
    lines = []
    for start in range(0, len(values), per_row):
        chunk = values[start:start + per_row]
        lines.append("    .byte " + ",".join(formatter.format(v) for v in chunk))
    return "\n".join(lines)


def generate_source():
    periods = [0] + [note_period(name) for name in NOTE_TABLE]
    lead = pattern_bytes(MUSIC_LEAD)
    bass = pattern_bytes(MUSIC_BASS)
    assert len(lead) == len(bass)

    return "\n".join([
        "; ------------------------------------------------------------------",
        "; Generated by tools/build_rom.py - do not edit.",
        "; ------------------------------------------------------------------",
        "",
        "MUSIC_LENGTH = %d" % len(lead),
        "",
        "    .org $C000",
        "",
        "; Two nametables of tiles and attributes, forming one seamless loop.",
        "nametable_data:",
        rows_of(build_nametables()),
        "",
        "; APU timer periods, indexed by note. Index 0 is a rest.",
        "note_lo:",
        rows_of([p & 0xFF for p in periods]),
        "note_hi:",
        rows_of([(p >> 8) & 0x07 for p in periods]),
        "",
        "music_lead:",
        rows_of(lead),
        "music_bass:",
        rows_of(bass),
        "",
        "    .org $FFFA",
        "    .word nmi",
        "    .word reset",
        "    .word irq",
        "",
    ])


def main():
    generated = generate_source()
    with open(GENERATED, "w") as handle:
        handle.write(generated)

    with open(SOURCE) as handle:
        source = handle.read()

    output = asm6502.assemble(source + "\n" + generated)

    prg = bytearray(32768)
    for address, byte in output.items():
        if not 0x8000 <= address <= 0xFFFF:
            raise SystemExit("code at $%04X falls outside PRG ROM" % address)
        prg[address - 0x8000] = byte

    header = bytearray(16)
    header[0:4] = b"NES\x1a"
    header[4] = 2          # 32 KB PRG
    header[5] = 1          # 8 KB CHR
    header[6] = 0x01       # mapper 0, vertical mirroring

    with open(OUTPUT, "wb") as handle:
        handle.write(bytes(header) + bytes(prg) + tiles.build_chr())

    top = max(a for a in output if a < 0xFFFA)
    print("wrote %s" % os.path.relpath(OUTPUT, ROOT))
    print("  code and data end at $%04X" % top)


if __name__ == "__main__":
    main()
