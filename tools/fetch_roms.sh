#!/usr/bin/env bash
# Download freely distributable homebrew games and the standard nesdev test
# ROMs into roms/. Nothing here is commercial: the games come from the
# retrobrews collection, which only carries titles the authors released for
# free redistribution.
set -euo pipefail

cd "$(dirname "$0")/.."
mkdir -p roms/games roms/tests

GAMES_URL="https://raw.githubusercontent.com/retrobrews/nes-games/master"
GAMES=(
    nomolos          # UNROM platformer
    owlia            # MMC1 action-adventure, "The Legends of Owlia"
    sir-ababol-remastered
    driar            # MMC1 puzzle-platformer
    twindragons      # MMC3
    super-tilt-bro   # NROM fighter
    starevil         # UNROM shmup
    lala             # NROM puzzler
    thwaite          # NROM
    nesertbus
)

TESTS_URL="https://raw.githubusercontent.com/christopherpow/nes-test-roms/master"
TESTS=(
    "other/nestest.nes"
    "instr_test-v5/official_only.nes"
    "instr_test-v5/all_instrs.nes"
    "cpu_timing_test6/cpu_timing_test.nes"
    "ppu_vbl_nmi/ppu_vbl_nmi.nes"
    "sprite_hit_tests_2005.10.05/01.basics.nes"
    "blargg_ppu_tests_2005.09.15b/palette_ram.nes"
    "blargg_ppu_tests_2005.09.15b/vram_access.nes"
    "apu_test/apu_test.nes"
    "mmc3_test_2/rom_singles/1-clocking.nes"
)

for game in "${GAMES[@]}"; do
    echo "fetching $game"
    curl -fsSL "$GAMES_URL/$game.nes" -o "roms/games/$game.nes"
done

for test in "${TESTS[@]}"; do
    name="$(basename "$test")"
    dir="$(basename "$(dirname "$test")")"
    echo "fetching $test"
    curl -fsSL "$TESTS_URL/$test" -o "roms/tests/${dir}-${name}"
done

echo
echo "downloaded:"
ls -la roms/games roms/tests
