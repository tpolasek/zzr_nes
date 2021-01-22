use crate::nes::rom::Rom;

pub struct Ppu {
    reg_ctrl : u8,
    reg_mask : u8,
    reg_status : u8,
    address_latch: bool,
    address : u16,
    data : u8,
    data_buffer: u8,
    vertical_blank : bool,
}

impl Ppu {
    pub fn new() -> Ppu {
        Self {
            reg_ctrl : 0,
            reg_mask : 0,
            reg_status: 0,
            address_latch: false,
            address: 0,
            data : 0,
            data_buffer : 0,
            vertical_blank: false,
        }
    }

    pub fn cpuRead(&mut self, rom: &Rom, register_num : u8) -> u8{
        return match register_num {
            0 => self.read_PPUCTRL(),
            1 => self.read_PPUMASK(),
            2 => self.read_PPUSTATUS(),
            3 => self.read_OAMADDR(),
            4 => self.read_OAMDATA(),
            5 => self.read_PPUSCROLL(),
            6 => self.read_PPUADDR(),
            7 => self.read_PPUDATA(),
            _  => { panic!("We should never get here in the PPU addr={}", register_num);}
        }
    }

    pub fn cpuWrite(&mut self, rom : &mut Rom, register_num : u8, value : u8){
        match register_num {
            0 => self.write_PPUCTRL(value),
            1 => self.write_PPUMASK(value),
            2 => self.write_PPUSTATUS(value),
            3 => self.write_OAMADDR(value),
            4 => self.write_OAMDATA(value),
            5 => self.write_PPUSCROLL(value),
            6 => self.write_PPUADDR(value),
            7 => self.write_PPUDATA(value),
            _  => { panic!("We should never get here in the PPU addr={}", register_num);}
        }
    }

    /// Start Read Register
    fn ppuRead(&self, address: u16) -> u8{
        return 0; //TODO this is incomplete
    }

    fn ppuWrite(&self, address: u16, value : u8){
        return ; //TODO this is incomplete
    }

    fn read_PPUCTRL(&self) -> u8{
       return self.reg_ctrl;
    }

    fn read_PPUMASK(&self) -> u8{
        return self.reg_mask;
    }

    fn read_PPUSTATUS(&mut self) -> u8{
        // Wack behaviour on a NES, data_buffer gets here somehow
        self.vertical_blank = false;
        self.address_latch = false;
        return (self.reg_status & 0xE0) | (self.data_buffer & 0x1F);
    }

    fn read_OAMADDR(&self) -> u8{
        return 0; //TODO
    }

    fn read_OAMDATA(&self) -> u8{
        return 0;//TODO
    }

    fn read_PPUSCROLL(&self) -> u8{
        return 0;//TODO
    }

    fn read_PPUADDR(&self) -> u8{
        return 0; //TODO

    }

    fn read_PPUDATA(&mut self) -> u8{
        self.data = self.data_buffer;
        self.data_buffer = self.ppuRead(self.address);

        if self.address >= 0x3F00 {
            //TODO from YT tutorial
            self.data = self.data_buffer
        }
        self.address += 1; // Auto incrementing
        return self.data;
    }

    /// End Read Registers

    fn write_PPUCTRL(&mut self, value : u8){
        self.reg_ctrl = value;
    }
    fn write_PPUMASK(&mut self, value : u8){
        self.reg_mask = value;
    }
    fn write_PPUSTATUS(&mut self, value : u8){
        //TODO
    }
    fn write_OAMADDR(&mut self, value : u8){
        //TODO
    }
    fn write_OAMDATA(&mut self, value : u8){
        //TODO
    }
    fn write_PPUSCROLL(&mut self, value : u8){
        //TODO
    }
    fn write_PPUADDR(&mut self, value : u8){
        if !self.address_latch {
            self.address = (self.address & 0x00FF) | ((value as u16) << 8);
            self.address_latch = true;
        }
        else{
            self.address = (self.address & 0xFF00) | (value as u16);
            self.address_latch = false;
        }
        self.address += 1; // Auto incrementing TODO left off here
    }

    fn write_PPUDATA(&mut self, value : u8){
        // do nothing?
    }
}