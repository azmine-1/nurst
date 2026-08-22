/// The eight buttons of a standard NES pad, in the order the controller's
/// shift register reports them.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Button {
    A = 0,
    B = 1,
    Select = 2,
    Start = 3,
    Up = 4,
    Down = 5,
    Left = 6,
    Right = 7,
}

#[derive(Default)]
pub struct Controller {
    /// Live button state, one bit per `Button`.
    buttons: u8,
    /// Snapshot taken while the strobe line is high, shifted out on reads.
    shift: u8,
    strobe: bool,
}

impl Controller {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_button(&mut self, button: Button, pressed: bool) {
        let mask = 1 << button as u8;
        if pressed {
            self.buttons |= mask;
        } else {
            self.buttons &= !mask;
        }
    }

    pub fn write_strobe(&mut self, val: u8) {
        self.strobe = val & 1 != 0;
        if self.strobe {
            self.shift = self.buttons;
        }
    }

    pub fn read(&mut self) -> u8 {
        // While the strobe is high the pad keeps reporting button A.
        if self.strobe {
            self.shift = self.buttons;
        }
        let value = self.shift & 1;
        self.shift = (self.shift >> 1) | 0x80; // reads past the eighth return 1
        value
    }
}
