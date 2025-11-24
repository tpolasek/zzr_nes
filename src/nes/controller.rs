pub enum Button {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    A,
    B,
    SELECT,
    START,
}

pub struct Controller {
    up: u8,
    down: u8,
    left: u8,
    right: u8,
    a: u8,
    b: u8,
    select: u8,
    start: u8,

    read_counter: u8,
    strobe_high: bool,
}

impl Controller {
    pub fn new() -> Controller {
        Self {
            up: 0,
            down: 0,
            left: 0,
            right: 0,
            a: 0,
            b: 0,
            select: 0,
            start: 0,
            read_counter: 0,
            strobe_high: true,
        }
    }
    pub fn pressed(&mut self, button: Button) {
        self.button_change(button, true);
    }

    pub fn released(&mut self, button: Button) {
        self.button_change(button, false);
    }

    fn button_change(&mut self, button: Button, pressed: bool) {
        let value: u8 = match pressed {
            true => 1,
            false => 0,
        };
        match button {
            Button::UP => self.up = value,
            Button::DOWN => self.down = value,
            Button::LEFT => self.left = value,
            Button::RIGHT => self.right = value,
            Button::A => self.a = value,
            Button::B => self.b = value,
            Button::SELECT => self.select = value,
            Button::START => self.start = value,
        }
    }

    pub fn read(&mut self) -> u8 {
        let value: u8 = match self.read_counter {
            0 => self.a,
            1 => self.b,
            2 => self.select,
            3 => self.start,
            4 => self.up,
            5 => self.down,
            6 => self.left,
            7 => self.right,
            _ => 1, // default is 1 after we read all buttons
        };

        // only increment the read counter when strobe = low
        if !self.strobe_high && self.read_counter < 8 {
            self.read_counter += 1;
        }

        return 0x40 | value;
    }

    pub fn write(&mut self, value: u8) {
        self.strobe_high = (value & 0x01 != 0);
        if self.strobe_high {
            self.read_counter = 0;
        }
    }
}
