mod frontend;
mod trace;

use nurst::cpu::CPU;
use nurst::input::Button;
use nurst::ppu::{HEIGHT, WIDTH};
use nurst::rom::Rom;
use nurst::{mapper, png};
use std::fs;
use std::process::ExitCode;

const SAMPLE_RATE: u32 = 44100;

struct Options {
    rom_path: String,
    scale: u32,
    headless: bool,
    frames: Option<u64>,
    screenshot: Option<String>,
    trace: Option<String>,
    trace_start: Option<u16>,
    trace_lines: usize,
    mute: bool,
    input_script: Vec<(u64, u8)>,
}

impl Options {
    fn parse(args: &[String]) -> Result<Options, String> {
        let mut options = Options {
            rom_path: String::new(),
            scale: 3,
            headless: false,
            frames: None,
            screenshot: None,
            trace: None,
            trace_start: None,
            trace_lines: 10_000,
            mute: false,
            input_script: Vec::new(),
        };

        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            let mut value = |name: &str| {
                iter.next().cloned().ok_or_else(|| format!("{} needs a value", name))
            };
            match arg.as_str() {
                "--scale" => options.scale = value("--scale")?.parse().map_err(us)?,
                "--headless" => options.headless = true,
                "--frames" => options.frames = Some(value("--frames")?.parse().map_err(us)?),
                "--screenshot" => options.screenshot = Some(value("--screenshot")?),
                "--trace" => options.trace = Some(value("--trace")?),
                "--trace-start" => {
                    let text = value("--trace-start")?;
                    let text = text.trim_start_matches("0x").trim_start_matches('$');
                    options.trace_start =
                        Some(u16::from_str_radix(text, 16).map_err(us)?);
                }
                "--trace-lines" => {
                    options.trace_lines = value("--trace-lines")?.parse().map_err(us)?
                }
                "--mute" => options.mute = true,
                "--input" => options.input_script = parse_input_script(&value("--input")?)?,
                "--help" | "-h" => return Err(usage()),
                other if other.starts_with('-') => {
                    return Err(format!("unknown option {}\n\n{}", other, usage()));
                }
                other => options.rom_path = other.to_string(),
            }
        }

        if options.rom_path.is_empty() {
            return Err(usage());
        }
        Ok(options)
    }
}

fn us<E: std::fmt::Display>(err: E) -> String {
    err.to_string()
}

fn parse_input_script(text: &str) -> Result<Vec<(u64, u8)>, String> {
    let mut script = Vec::new();
    for entry in text.split(';').filter(|e| !e.trim().is_empty()) {
        let (frame, buttons) = entry
            .split_once(':')
            .ok_or_else(|| format!("expected frame:buttons in {:?}", entry))?;
        let frame = frame.trim().parse::<u64>().map_err(us)?;
        let mut mask = 0u8;
        for name in buttons.split(',').filter(|n| !n.trim().is_empty()) {
            mask |= match name.trim().to_ascii_lowercase().as_str() {
                "a" => 1 << Button::A as u8,
                "b" => 1 << Button::B as u8,
                "select" => 1 << Button::Select as u8,
                "start" => 1 << Button::Start as u8,
                "up" => 1 << Button::Up as u8,
                "down" => 1 << Button::Down as u8,
                "left" => 1 << Button::Left as u8,
                "right" => 1 << Button::Right as u8,
                other => return Err(format!("unknown button {:?}", other)),
            };
        }
        script.push((frame, mask));
    }
    script.sort_by_key(|(frame, _)| *frame);
    Ok(script)
}

pub fn apply_input_script(cpu: &mut CPU, script: &[(u64, u8)], frame: u64) {
    for (at, mask) in script {
        if *at != frame {
            continue;
        }
        for button in ALL_BUTTONS {
            cpu.bus.controllers[0].set_button(button, mask & (1 << button as u8) != 0);
        }
    }
}

const ALL_BUTTONS: [Button; 8] = [
    Button::A,
    Button::B,
    Button::Select,
    Button::Start,
    Button::Up,
    Button::Down,
    Button::Left,
    Button::Right,
];

fn usage() -> String {
    "\
nurst - a NES emulator

USAGE:
    nurst <rom.nes> [OPTIONS]

OPTIONS:
    --scale <n>          Window scale factor (default 3)
    --mute               Disable audio output
    --headless           Run without a window; useful with --frames
    --frames <n>         Stop after n frames
    --screenshot <path>  Write a PNG of the final frame
    --trace <path>       Write a nestest-style CPU trace
    --trace-start <hex>  Force the program counter before tracing (e.g. C000)
    --trace-lines <n>    How many instructions to trace (default 10000)
    --input <script>     Scripted pad input, e.g. 10:start;12:;90:a;92:
                         Holds the named buttons from that frame onwards

CONTROLS:
    Arrow keys  D-pad          Z / X       B / A
    Enter       Start          Right shift Select
    R           Reset          Tab (hold)  Fast forward
    P           Pause          Esc         Quit
    F12         Save a screenshot next to the ROM"
        .to_string()
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let options = match Options::parse(&args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{}", message);
            return ExitCode::FAILURE;
        }
    };

    match run(options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("nurst: {}", message);
            ExitCode::FAILURE
        }
    }
}

fn run(options: Options) -> Result<(), String> {
    let rom_data = fs::read(&options.rom_path)
        .map_err(|e| format!("cannot read {}: {}", options.rom_path, e))?;
    let rom = Rom::new(&rom_data)?;

    println!(
        "{}: mapper {}, {} KB PRG, {} KB {}, {:?} mirroring",
        options.rom_path,
        rom.mapper,
        rom.prg_rom.len() / 1024,
        rom.chr_rom.len() / 1024,
        if rom.chr_ram { "CHR RAM" } else { "CHR ROM" },
        rom.mirroring,
    );

    let mut cpu = CPU::new(mapper::from_rom(rom)?, SAMPLE_RATE);
    cpu.reset();

    if let Some(path) = &options.trace {
        trace::write_trace(&mut cpu, path, options.trace_start, options.trace_lines)?;
        return Ok(());
    }

    if options.headless {
        run_headless(&mut cpu, &options)
    } else {
        frontend::run(cpu, &options)
    }
}

fn run_headless(cpu: &mut CPU, options: &Options) -> Result<(), String> {
    let target = options.frames.unwrap_or(60);
    let mut frames = 0;
    let mut peak = 0.0f32;
    while frames < target {
        apply_input_script(cpu, &options.input_script, frames);
        cpu.step();
        if cpu.bus.ppu.frame_complete {
            cpu.bus.ppu.frame_complete = false;
            frames += 1;
        }
        let samples = cpu.bus.apu.drain_samples();
        peak = samples.iter().fold(peak, |acc, s| acc.max(s.abs()));
    }
    println!(
        "ran {} frames ({} CPU cycles), peak audio level {:.3}",
        frames, cpu.cycles, peak
    );

    if let Some(path) = &options.screenshot {
        save_screenshot(&cpu.bus.ppu.frame, path)?;
    }
    Ok(())
}

pub fn save_screenshot(frame: &[u32], path: &str) -> Result<(), String> {
    let data = png::encode(frame, WIDTH, HEIGHT);
    fs::write(path, data).map_err(|e| format!("cannot write {}: {}", path, e))?;
    println!("wrote {}", path);
    Ok(())
}
