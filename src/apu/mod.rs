//! The 2A03 audio processing unit: two pulse channels, a triangle, a noise
//! channel and the DMC sample player, mixed down to f32 samples for the host.

mod channels;

use channels::{Dmc, Noise, Pulse, Triangle};

/// NTSC CPU clock, which is also the APU's input clock.
pub const CPU_CLOCK_HZ: f64 = 1_789_773.0;

pub struct APU {
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,

    frame_counter: u32,
    /// 0 = four-step sequence (with IRQ), 1 = five-step sequence.
    five_step_mode: bool,
    irq_inhibit: bool,
    frame_irq: bool,

    /// Fractional accumulator that decides when to emit the next sample.
    sample_accumulator: f64,
    cycles_per_sample: f64,
    samples: Vec<f32>,

    /// Simple one-pole high-pass/low-pass pair, matching the NES's output filters.
    highpass_prev_in: f32,
    highpass_prev_out: f32,
    lowpass_prev_out: f32,
}

impl APU {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            pulse1: Pulse::new(true),
            pulse2: Pulse::new(false),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: Dmc::new(),
            frame_counter: 0,
            five_step_mode: false,
            irq_inhibit: false,
            frame_irq: false,
            sample_accumulator: 0.0,
            cycles_per_sample: CPU_CLOCK_HZ / sample_rate as f64,
            samples: Vec::with_capacity(4096),
            highpass_prev_in: 0.0,
            highpass_prev_out: 0.0,
            lowpass_prev_out: 0.0,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x4000..=0x4003 => self.pulse1.write(addr & 3, val),
            0x4004..=0x4007 => self.pulse2.write(addr & 3, val),
            0x4008..=0x400B => self.triangle.write(addr & 3, val),
            0x400C..=0x400F => self.noise.write(addr & 3, val),
            0x4010..=0x4013 => self.dmc.write(addr & 3, val),
            0x4015 => {
                self.pulse1.set_enabled(val & 0x01 != 0);
                self.pulse2.set_enabled(val & 0x02 != 0);
                self.triangle.set_enabled(val & 0x04 != 0);
                self.noise.set_enabled(val & 0x08 != 0);
                self.dmc.set_enabled(val & 0x10 != 0);
            }
            0x4017 => {
                self.five_step_mode = val & 0x80 != 0;
                self.irq_inhibit = val & 0x40 != 0;
                if self.irq_inhibit {
                    self.frame_irq = false;
                }
                self.frame_counter = 0;
                // Switching to the five-step sequence clocks everything at once.
                if self.five_step_mode {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                }
            }
            _ => {}
        }
    }

    pub fn read_status(&mut self) -> u8 {
        let mut status = 0;
        if self.pulse1.length_counter > 0 {
            status |= 0x01;
        }
        if self.pulse2.length_counter > 0 {
            status |= 0x02;
        }
        if self.triangle.length_counter > 0 {
            status |= 0x04;
        }
        if self.noise.length_counter > 0 {
            status |= 0x08;
        }
        if self.dmc.bytes_remaining > 0 {
            status |= 0x10;
        }
        if self.frame_irq {
            status |= 0x40;
        }
        if self.dmc.irq_flag {
            status |= 0x80;
        }
        self.frame_irq = false;
        status
    }

    pub fn irq_pending(&self) -> bool {
        self.frame_irq || self.dmc.irq_flag
    }

    /// Address the DMC wants to read, if its sample buffer just ran dry.
    pub fn dmc_fetch_address(&self) -> Option<u16> {
        self.dmc.fetch_address()
    }

    pub fn dmc_supply_sample(&mut self, value: u8) {
        self.dmc.supply_sample(value);
    }

    /// Advance one CPU cycle.
    pub fn tick(&mut self) {
        // The triangle's timer runs at the full CPU rate; everything else at half.
        self.triangle.clock_timer();
        if self.frame_counter % 2 == 0 {
            self.pulse1.clock_timer();
            self.pulse2.clock_timer();
            self.noise.clock_timer();
        }
        self.dmc.clock_timer();

        self.clock_frame_counter();

        self.sample_accumulator += 1.0;
        if self.sample_accumulator >= self.cycles_per_sample {
            self.sample_accumulator -= self.cycles_per_sample;
            let sample = self.filter(self.mix());
            self.samples.push(sample);
        }
    }

    /// The frame counter divides the CPU clock into quarter- and half-frames,
    /// which drive the envelopes, sweeps and length counters.
    fn clock_frame_counter(&mut self) {
        self.frame_counter += 1;
        let step = self.frame_counter;

        if self.five_step_mode {
            match step {
                7457 | 22371 => self.clock_quarter_frame(),
                14913 => {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                }
                37281 => {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                }
                37282 => self.frame_counter = 0,
                _ => {}
            }
        } else {
            match step {
                7457 | 22371 => self.clock_quarter_frame(),
                14913 => {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                }
                29829 => {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                    if !self.irq_inhibit {
                        self.frame_irq = true;
                    }
                }
                29830 => self.frame_counter = 0,
                _ => {}
            }
        }
    }

    fn clock_quarter_frame(&mut self) {
        self.pulse1.clock_envelope();
        self.pulse2.clock_envelope();
        self.triangle.clock_linear_counter();
        self.noise.clock_envelope();
    }

    fn clock_half_frame(&mut self) {
        self.pulse1.clock_length_and_sweep();
        self.pulse2.clock_length_and_sweep();
        self.triangle.clock_length();
        self.noise.clock_length();
    }

    /// The hardware's non-linear mixer, using the standard lookup formulas.
    fn mix(&self) -> f32 {
        let pulse_sum = self.pulse1.output() as f32 + self.pulse2.output() as f32;
        let pulse_out =
            if pulse_sum == 0.0 { 0.0 } else { 95.88 / (8128.0 / pulse_sum + 100.0) };

        let tnd = self.triangle.output() as f32 / 8227.0
            + self.noise.output() as f32 / 12241.0
            + self.dmc.output() as f32 / 22638.0;
        let tnd_out = if tnd == 0.0 { 0.0 } else { 159.79 / (1.0 / tnd + 100.0) };

        pulse_out + tnd_out
    }

    fn filter(&mut self, input: f32) -> f32 {
        // High-pass at ~90 Hz removes the DC offset the mixer introduces.
        let hp = 0.996 * (self.highpass_prev_out + input - self.highpass_prev_in);
        self.highpass_prev_in = input;
        self.highpass_prev_out = hp;
        // Low-pass at ~14 kHz softens the square edges.
        let lp = self.lowpass_prev_out + 0.4 * (hp - self.lowpass_prev_out);
        self.lowpass_prev_out = lp;
        lp
    }

    pub fn drain_samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.samples)
    }

    pub fn pending_samples(&self) -> usize {
        self.samples.len()
    }
}

/// Length counter reload values indexed by the 5-bit field in $4003/$4007/etc.
pub const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96,
    22, 192, 24, 72, 26, 16, 28, 32, 30,
];
