//! SDL2 window, keyboard and audio.

use nurst::cpu::CPU;
use nurst::input::Button;
use nurst::ppu::{HEIGHT, WIDTH};
use crate::{Options, SAMPLE_RATE, apply_input_script, save_screenshot};

use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::time::{Duration, Instant};

/// Keep roughly this many milliseconds of audio queued. Too little starves the
/// device and crackles; too much adds latency between input and sound.
const AUDIO_TARGET_MS: u32 = 60;

fn button_for(key: Keycode) -> Option<Button> {
    Some(match key {
        Keycode::Up | Keycode::W => Button::Up,
        Keycode::Down | Keycode::S => Button::Down,
        Keycode::Left | Keycode::A => Button::Left,
        Keycode::Right | Keycode::D => Button::Right,
        Keycode::Z | Keycode::K => Button::B,
        Keycode::X | Keycode::L => Button::A,
        Keycode::Return | Keycode::KpEnter => Button::Start,
        Keycode::RShift | Keycode::Backspace => Button::Select,
        _ => return None,
    })
}

pub fn run(mut cpu: CPU, options: &Options) -> Result<(), String> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;

    let window = video
        .window(
            "nurst",
            WIDTH as u32 * options.scale,
            HEIGHT as u32 * options.scale,
        )
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().accelerated().build().map_err(|e| e.to_string())?;
    canvas.set_logical_size(WIDTH as u32, HEIGHT as u32).map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::ARGB8888, WIDTH as u32, HEIGHT as u32)
        .map_err(|e| e.to_string())?;

    let audio = if options.mute {
        None
    } else {
        let spec = AudioSpecDesired {
            freq: Some(SAMPLE_RATE as i32),
            channels: Some(1),
            samples: Some(1024),
        };
        let queue = sdl.audio()?.open_queue::<f32, _>(None, &spec)?;
        queue.resume();
        Some(queue)
    };

    let mut events = sdl.event_pump()?;
    let mut paused = false;
    let mut fast_forward = false;
    let mut frames: u64 = 0;
    let mut next_frame = Instant::now();
    let frame_time = Duration::from_nanos(16_639_267); // 60.0988 Hz

    'running: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(key), repeat: false, .. } => match key {
                    Keycode::Escape => break 'running,
                    Keycode::P => paused = !paused,
                    Keycode::R => cpu.reset(),
                    Keycode::Tab => fast_forward = true,
                    Keycode::F12 => {
                        let path = format!("{}.png", options.rom_path);
                        save_screenshot(&cpu.bus.ppu.frame, &path)?;
                    }
                    other => {
                        if let Some(button) = button_for(other) {
                            cpu.bus.controllers[0].set_button(button, true);
                        }
                    }
                },
                Event::KeyUp { keycode: Some(key), .. } => match key {
                    Keycode::Tab => fast_forward = false,
                    other => {
                        if let Some(button) = button_for(other) {
                            cpu.bus.controllers[0].set_button(button, false);
                        }
                    }
                },
                _ => {}
            }
        }

        if !paused {
            apply_input_script(&mut cpu, &options.input_script, frames);
            // Run until the PPU signals the start of vertical blank.
            while !cpu.bus.ppu.frame_complete {
                cpu.step();
            }
            cpu.bus.ppu.frame_complete = false;
            frames += 1;

            let samples = cpu.bus.apu.drain_samples();
            match &audio {
                // Fast forward would flood the queue, so drop its audio.
                Some(queue) if !fast_forward => queue.queue_audio(&samples)?,
                _ => {}
            }
        }

        let frame = &cpu.bus.ppu.frame;
        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..HEIGHT {
                    for x in 0..WIDTH {
                        let pixel = frame[y * WIDTH + x];
                        let offset = y * pitch + x * 4;
                        buffer[offset] = pixel as u8; // B
                        buffer[offset + 1] = (pixel >> 8) as u8; // G
                        buffer[offset + 2] = (pixel >> 16) as u8; // R
                        buffer[offset + 3] = 0xFF;
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();

        if let Some(limit) = options.frames {
            if frames >= limit {
                break 'running;
            }
        }

        if fast_forward {
            next_frame = Instant::now();
            continue;
        }

        // Prefer pacing on the audio queue: it is the clock the user hears.
        match &audio {
            Some(queue) => {
                let target = SAMPLE_RATE * AUDIO_TARGET_MS / 1000;
                while queue.size() / 4 > target {
                    std::thread::sleep(Duration::from_millis(1));
                }
                next_frame = Instant::now();
            }
            None => {
                next_frame += frame_time;
                let now = Instant::now();
                if next_frame > now {
                    std::thread::sleep(next_frame - now);
                } else {
                    next_frame = now;
                }
            }
        }
    }

    if let Some(path) = &options.screenshot {
        save_screenshot(&cpu.bus.ppu.frame, path)?;
    }
    println!("ran {} frames", frames);
    Ok(())
}
