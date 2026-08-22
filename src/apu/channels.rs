use super::LENGTH_TABLE;

const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0], // 12.5%
    [0, 1, 1, 0, 0, 0, 0, 0], // 25%
    [0, 1, 1, 1, 1, 0, 0, 0], // 50%
    [1, 0, 0, 1, 1, 1, 1, 1], // 25% inverted
];

const TRIANGLE_STEPS: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
    12, 13, 14, 15,
];

const NOISE_PERIODS: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

const DMC_RATES: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

/// The volume envelope shared by the pulse and noise channels.
#[derive(Default)]
struct Envelope {
    start: bool,
    divider: u8,
    decay: u8,
    period: u8,
    loop_flag: bool,
    constant: bool,
}

impl Envelope {
    fn clock(&mut self) {
        if self.start {
            self.start = false;
            self.decay = 15;
            self.divider = self.period;
        } else if self.divider > 0 {
            self.divider -= 1;
        } else {
            self.divider = self.period;
            if self.decay > 0 {
                self.decay -= 1;
            } else if self.loop_flag {
                self.decay = 15;
            }
        }
    }

    fn volume(&self) -> u8 {
        if self.constant { self.period } else { self.decay }
    }
}

pub struct Pulse {
    enabled: bool,
    duty: u8,
    sequence_step: u8,
    timer: u16,
    timer_period: u16,
    envelope: Envelope,
    pub length_counter: u8,
    length_halt: bool,

    sweep_enabled: bool,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_divider: u8,
    sweep_reload: bool,
    /// Pulse 1 negates with an extra -1; pulse 2 does not.
    ones_complement_sweep: bool,
}

impl Pulse {
    pub fn new(is_pulse1: bool) -> Self {
        Self {
            enabled: false,
            duty: 0,
            sequence_step: 0,
            timer: 0,
            timer_period: 0,
            envelope: Envelope::default(),
            length_counter: 0,
            length_halt: false,
            sweep_enabled: false,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            sweep_divider: 0,
            sweep_reload: false,
            ones_complement_sweep: is_pulse1,
        }
    }

    pub fn write(&mut self, register: u16, val: u8) {
        match register {
            0 => {
                self.duty = val >> 6;
                self.length_halt = val & 0x20 != 0;
                self.envelope.loop_flag = self.length_halt;
                self.envelope.constant = val & 0x10 != 0;
                self.envelope.period = val & 0x0F;
            }
            1 => {
                self.sweep_enabled = val & 0x80 != 0;
                self.sweep_period = (val >> 4) & 0x07;
                self.sweep_negate = val & 0x08 != 0;
                self.sweep_shift = val & 0x07;
                self.sweep_reload = true;
            }
            2 => self.timer_period = (self.timer_period & 0xFF00) | val as u16,
            _ => {
                self.timer_period = (self.timer_period & 0x00FF) | ((val as u16 & 0x07) << 8);
                if self.enabled {
                    self.length_counter = LENGTH_TABLE[(val >> 3) as usize];
                }
                self.sequence_step = 0;
                self.envelope.start = true;
            }
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }

    pub fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            self.sequence_step = (self.sequence_step + 1) & 0x07;
        } else {
            self.timer -= 1;
        }
    }

    pub fn clock_envelope(&mut self) {
        self.envelope.clock();
    }

    pub fn clock_length_and_sweep(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }

        if self.sweep_divider == 0 && self.sweep_enabled && self.sweep_shift > 0 {
            let target = self.sweep_target();
            if target <= 0x7FF && self.timer_period >= 8 {
                self.timer_period = target;
            }
        }
        if self.sweep_divider == 0 || self.sweep_reload {
            self.sweep_divider = self.sweep_period;
            self.sweep_reload = false;
        } else {
            self.sweep_divider -= 1;
        }
    }

    fn sweep_target(&self) -> u16 {
        let delta = self.timer_period >> self.sweep_shift;
        if self.sweep_negate {
            let extra = if self.ones_complement_sweep { 1 } else { 0 };
            self.timer_period.saturating_sub(delta + extra)
        } else {
            self.timer_period + delta
        }
    }

    pub fn output(&self) -> u8 {
        // Periods below 8, a muted sweep target and an expired length counter
        // all silence the channel outright.
        if !self.enabled
            || self.length_counter == 0
            || self.timer_period < 8
            || self.sweep_target() > 0x7FF
            || DUTY_TABLE[self.duty as usize][self.sequence_step as usize] == 0
        {
            0
        } else {
            self.envelope.volume()
        }
    }
}

pub struct Triangle {
    enabled: bool,
    timer: u16,
    timer_period: u16,
    sequence_step: u8,
    pub length_counter: u8,
    length_halt: bool,
    linear_counter: u8,
    linear_reload_value: u8,
    linear_reload: bool,
}

impl Triangle {
    pub fn new() -> Self {
        Self {
            enabled: false,
            timer: 0,
            timer_period: 0,
            sequence_step: 0,
            length_counter: 0,
            length_halt: false,
            linear_counter: 0,
            linear_reload_value: 0,
            linear_reload: false,
        }
    }

    pub fn write(&mut self, register: u16, val: u8) {
        match register {
            0 => {
                self.length_halt = val & 0x80 != 0;
                self.linear_reload_value = val & 0x7F;
            }
            2 => self.timer_period = (self.timer_period & 0xFF00) | val as u16,
            3 => {
                self.timer_period = (self.timer_period & 0x00FF) | ((val as u16 & 0x07) << 8);
                if self.enabled {
                    self.length_counter = LENGTH_TABLE[(val >> 3) as usize];
                }
                self.linear_reload = true;
            }
            _ => {}
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }

    pub fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            if self.length_counter > 0 && self.linear_counter > 0 {
                self.sequence_step = (self.sequence_step + 1) & 0x1F;
            }
        } else {
            self.timer -= 1;
        }
    }

    pub fn clock_linear_counter(&mut self) {
        if self.linear_reload {
            self.linear_counter = self.linear_reload_value;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }
        if !self.length_halt {
            self.linear_reload = false;
        }
    }

    pub fn clock_length(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn output(&self) -> u8 {
        // Very short periods produce an ultrasonic buzz on hardware; mute them.
        if !self.enabled || self.timer_period < 2 {
            0
        } else {
            TRIANGLE_STEPS[self.sequence_step as usize]
        }
    }
}

pub struct Noise {
    enabled: bool,
    timer: u16,
    timer_period: u16,
    shift_register: u16,
    mode: bool,
    envelope: Envelope,
    pub length_counter: u8,
    length_halt: bool,
}

impl Noise {
    pub fn new() -> Self {
        Self {
            enabled: false,
            timer: 0,
            timer_period: NOISE_PERIODS[0],
            shift_register: 1, // must start non-zero or the LFSR never moves
            mode: false,
            envelope: Envelope::default(),
            length_counter: 0,
            length_halt: false,
        }
    }

    pub fn write(&mut self, register: u16, val: u8) {
        match register {
            0 => {
                self.length_halt = val & 0x20 != 0;
                self.envelope.loop_flag = self.length_halt;
                self.envelope.constant = val & 0x10 != 0;
                self.envelope.period = val & 0x0F;
            }
            2 => {
                self.mode = val & 0x80 != 0;
                self.timer_period = NOISE_PERIODS[(val & 0x0F) as usize];
            }
            3 => {
                if self.enabled {
                    self.length_counter = LENGTH_TABLE[(val >> 3) as usize];
                }
                self.envelope.start = true;
            }
            _ => {}
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }

    pub fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            // Mode selects whether the feedback tap is bit 6 (short, metallic)
            // or bit 1 (long, hiss).
            let tap = if self.mode { 6 } else { 1 };
            let feedback = (self.shift_register & 1) ^ ((self.shift_register >> tap) & 1);
            self.shift_register = (self.shift_register >> 1) | (feedback << 14);
        } else {
            self.timer -= 1;
        }
    }

    pub fn clock_envelope(&mut self) {
        self.envelope.clock();
    }

    pub fn clock_length(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn output(&self) -> u8 {
        if !self.enabled || self.length_counter == 0 || self.shift_register & 1 != 0 {
            0
        } else {
            self.envelope.volume()
        }
    }
}

pub struct Dmc {
    enabled: bool,
    pub irq_flag: bool,
    irq_enabled: bool,
    loop_flag: bool,
    timer: u16,
    timer_period: u16,

    sample_address: u16,
    sample_length: u16,
    current_address: u16,
    pub bytes_remaining: u16,

    sample_buffer: Option<u8>,
    /// Set when the buffer is empty and a fetch from CPU memory is owed.
    fetch_pending: bool,

    shift_register: u8,
    bits_remaining: u8,
    silence: bool,
    level: u8,
}

impl Dmc {
    pub fn new() -> Self {
        Self {
            enabled: false,
            irq_flag: false,
            irq_enabled: false,
            loop_flag: false,
            timer: 0,
            timer_period: DMC_RATES[0],
            sample_address: 0xC000,
            sample_length: 1,
            current_address: 0xC000,
            bytes_remaining: 0,
            sample_buffer: None,
            fetch_pending: false,
            shift_register: 0,
            bits_remaining: 8,
            silence: true,
            level: 0,
        }
    }

    pub fn write(&mut self, register: u16, val: u8) {
        match register {
            0 => {
                self.irq_enabled = val & 0x80 != 0;
                if !self.irq_enabled {
                    self.irq_flag = false;
                }
                self.loop_flag = val & 0x40 != 0;
                self.timer_period = DMC_RATES[(val & 0x0F) as usize];
            }
            1 => self.level = val & 0x7F,
            2 => self.sample_address = 0xC000 + (val as u16) * 64,
            _ => self.sample_length = (val as u16) * 16 + 1,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.irq_flag = false;
        if !enabled {
            self.bytes_remaining = 0;
        } else if self.bytes_remaining == 0 {
            self.restart();
        }
    }

    fn restart(&mut self) {
        self.current_address = self.sample_address;
        self.bytes_remaining = self.sample_length;
        if self.sample_buffer.is_none() {
            self.fetch_pending = true;
        }
    }

    pub fn fetch_address(&self) -> Option<u16> {
        if self.fetch_pending && self.bytes_remaining > 0 {
            Some(self.current_address)
        } else {
            None
        }
    }

    pub fn supply_sample(&mut self, value: u8) {
        self.sample_buffer = Some(value);
        self.fetch_pending = false;
        // The sample pointer wraps to $8000 rather than leaving the ROM window.
        self.current_address = self.current_address.checked_add(1).unwrap_or(0x8000);
        if self.current_address == 0 {
            self.current_address = 0x8000;
        }
        self.bytes_remaining -= 1;
        if self.bytes_remaining == 0 {
            if self.loop_flag {
                self.restart();
            } else if self.irq_enabled {
                self.irq_flag = true;
            }
        }
    }

    pub fn clock_timer(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
            return;
        }
        self.timer = self.timer_period;

        if !self.silence {
            // Each bit nudges the 7-bit output level by two, with clamping.
            if self.shift_register & 1 != 0 {
                if self.level <= 125 {
                    self.level += 2;
                }
            } else if self.level >= 2 {
                self.level -= 2;
            }
        }
        self.shift_register >>= 1;

        self.bits_remaining -= 1;
        if self.bits_remaining == 0 {
            self.bits_remaining = 8;
            match self.sample_buffer.take() {
                Some(byte) => {
                    self.silence = false;
                    self.shift_register = byte;
                    if self.bytes_remaining > 0 {
                        self.fetch_pending = true;
                    }
                }
                None => self.silence = true,
            }
        }
    }

    pub fn output(&self) -> u8 {
        self.level
    }
}
