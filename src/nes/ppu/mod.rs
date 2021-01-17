pub struct Ppu {

}

impl Ppu {
    pub fn read(&self, register_num : u8) -> u8{
        return 0; //TODO this is incomplete?
    }

    pub fn write(&mut self, register_num : u8, value : u8){
        match register_num {
            0 => self.PPUCTRL(value),
            1 => self.PPUMASK(value),
            2 => self.PPUSTATUS(value),
            3 => self.OAMADDR(value),
            4 => self.OAMDATA(value),
            5 => self.PPUSCROLL(value),
            6 => self.PPUADDR(value),
            7 => self.PPUDATA(value),
            _  => { panic!("We should never get here in the PPU addr={}", register_num);}
        }
    }

    fn PPUCTRL(&mut self, value : u8){
        //TODO
    }
    fn PPUMASK(&mut self, value : u8){
        //TODO
    }
    fn PPUSTATUS(&mut self, value : u8){
        //TODO
    }
    fn OAMADDR(&mut self, value : u8){
        //TODO
    }
    fn OAMDATA(&mut self, value : u8){
        //TODO
    }
    fn PPUSCROLL(&mut self, value : u8){
        //TODO
    }
    fn PPUADDR(&mut self, value : u8){
        //TODO
    }
    fn PPUDATA(&mut self, value : u8){
        //TODO
    }
}