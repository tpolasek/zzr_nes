pub enum Button {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    A,
    B,
    SELECT,
    START
}

#[derive(Debug)]
pub struct Controller {
    up: u8,
    down: u8,
    left: u8,
    right: u8,
    a : u8,
    b : u8,
    select : u8,
    start : u8,

    value : u8, // TODO remove this its specifically for snake
}

impl Controller {
    pub fn new() -> Controller {
        Self {
            up : 0,
            down : 0,
            left : 0,
            right : 0,
            a : 0,
            b : 0,
            select : 0,
            start : 0,
            value : 0
        }
    }
    pub fn pressed(&mut self, button: Button){
        self.button_change(button, true);
    }

    pub fn released(&mut self, button: Button){
        self.button_change(button, false);
    }

    fn button_change(&mut self, button: Button, pressed : bool){
        let value : u8 = match pressed{
            true => 1,
            false => 0
        };
        match button {
            Button::UP => self.up = value,
            Button::DOWN => self.down = value,
            Button::LEFT=> self.left = value,
            Button::RIGHT => self.right = value,
            Button::A => self.a = value,
            Button::B => self.b = value,
            Button::SELECT => self.select = value,
            Button::START => self.start = value
        }
        let new_value = self.generate_value();
        if new_value != 0 {
            self.value = new_value;
        }
    }
    
    pub fn read(&self) -> u8 {
        return self.value;
    }

    fn generate_value(&self) -> u8{
        if self.up != 0 {
            return 0x77;
        }

        if self.left != 0 {
            return 0x61;
        }

        if self.down != 0 {
            return 0x73;
        }

        if self.right != 0 {
            return 0x64;
        }

        return 0x00;
    }


}