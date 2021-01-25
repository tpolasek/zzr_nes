use crate::nes::rom::{Rom, Mirroring};

static PALETTE_LOOKUP: [u32; 64] = [
    0x545454, 0x001E74, 0x081090, 0x300088, 0x440064, 0x5C0030, 0x540400, 0x3C1800, 0x202A00, 0x083A00, 0x004000, 0x003C00, 0x00323C, 0x000000, 0x000000, 0x000000,
    0x989698, 0x084CC4, 0x3032EC, 0x5C1EE4, 0x8814B0, 0xA01464, 0x982220, 0x783C00, 0x545A00, 0x287200, 0x087C00, 0x007628, 0x006678, 0x000000, 0x000000, 0x000000,
    0xECEEEC, 0x4C9AEC, 0x787CEC, 0xB062EC, 0xE454EC, 0xEC58B4, 0xEC6A64, 0xD48820, 0xA0AA00, 0x74C400, 0x4CD020, 0x38CC6C, 0x38B4CC, 0x3C3C3C, 0x000000, 0x000000,
    0xECEEEC, 0xA8CCEC, 0xBCBCEC, 0xD4B2EC, 0xECAEEC, 0xECAED4, 0xECB4B0, 0xE4C490, 0xCCD278, 0xB4DE78, 0xA8E290, 0x98E2B4, 0xA0D6E4, 0xA0A2A0, 0x000000, 0x000000
];

const VBLANK_BIT :u8 = (1 << 7);


pub struct Ppu { // TODO once we are done debugging remove any public methods
    reg_ctrl : u8,
    reg_mask : u8,
    reg_status : u8,
    address_latch: bool,
    address : u16,
    data : u8,
    data_buffer: u8,
    pub pixel: u16,
    pub scanline: u16,

    // https://wiki.nesdev.com/w/index.php/PPU_memory_map
    // pattern table usually maps to rom CHR
    vram_bank_1: [u8; 0x400], //  Nametable Ram only 2k (room for 2 nametables mirrored, some roms have onboard memory for 4 tables)
    vram_bank_2: [u8; 0x400],
    palette_ram: [u8; 0x20],
    //TODO OAM memory 256 bytes (64-4 byte groups)
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
            pixel: 0,
            scanline: 0,
            vram_bank_1: [0; 0x400],
            vram_bank_2: [0; 0x400],
            palette_ram: [0; 0x20]
        }
    }

    pub fn cpuReadImmutable(&self, rom: &Rom, register_num : u8) -> u8{
        return match register_num {
            0 => self.read_PPUCTRL_Immutable(),
            1 => self.read_PPUMASK_Immutable(),
            2 => self.read_PPUSTATUS_Immutable(),
            3 => self.read_OAMADDR(),
            4 => self.read_OAMDATA(),
            5 => self.read_PPUSCROLL(),
            6 => self.read_PPUADDR(),
            7 => self.read_PPUDATA_Immutable(),
            _  => { panic!("We should never get here in the PPU addr={}", register_num);}
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
            7 => self.read_PPUDATA(rom),
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
    fn ppuRead(&self, rom: &Rom, address: u16) -> u8{
        return match address {
            0x0000..=0x1FFF => {
                return rom.read_chr(address);
            },
            0x2000..=0x3EFF => {
                let tmp_addr = address & 0xFFF;

                return match rom.mirroring {
                    Mirroring::HORIZONTAL => {
                        if tmp_addr <= 0x7FF {
                            return self.vram_bank_1[(tmp_addr & 0x3FF) as usize];
                        }
                        else{ //0x800 - 0xFFF
                            return self.vram_bank_2[(tmp_addr & 0x3FF) as usize];
                        }
                    },
                    Mirroring::VERTICAL => {
                        if tmp_addr <= 0x3FF {
                            return self.vram_bank_1[(tmp_addr & 0x3FF) as usize];
                        }
                        else if tmp_addr <= 0x7FF {
                            return self.vram_bank_2[(tmp_addr & 0x3FF) as usize];
                        }
                        else if tmp_addr <= 0xBFF {
                            return self.vram_bank_1[(tmp_addr & 0x3FF) as usize];
                        }
                        else{ //0x800 - 0xFFF
                            return self.vram_bank_2[(tmp_addr & 0x3FF) as usize];
                        }
                    },
                    Mirroring::FOUR_SCREEN => {
                        // TODO implement, also need SINGLE SCREEN
                        return 0;
                    }
                }
            },
            0x3F00..=0x3FFF => {
                let mut tmp_addr = address & 0x1F;

                // Palette mirroring
                tmp_addr = match address {
                    0x10 | 0x14 | 0x18 | 0x1C => tmp_addr & 0xF,
                    _ => tmp_addr
                };

                if self.reg_mask & 0x1 != 0 {
                    // greyscale
                    return self.palette_ram[tmp_addr as usize] & 0x30;
                }
                else{
                    return self.palette_ram[tmp_addr as usize] & 0x3F;
                }
            }
            _ => 0 // TODO validate this is correct, do we only go to 0x3FFF?
        };

    }

    fn ppuWrite(&self, address: u16, value : u8){
        return ; //TODO this is incomplete
    }


    fn read_PPUCTRL(&self) -> u8{
       return 0; // not readable
    }

    fn read_PPUCTRL_Immutable(&self) -> u8{
        return self.reg_ctrl; // for debug only
    }

    fn read_PPUMASK(&self) -> u8{
        return 0; // not readable
    }

    fn read_PPUMASK_Immutable(&self) -> u8{
        return self.reg_mask; // for debug only
    }


    fn read_PPUSTATUS(&mut self) -> u8{
        self.address_latch = false;

        // last 5 bits are the last data bits written -- whack
        let output = (self.reg_status & 0xE0) | (self.data_buffer & 0x1F);

        self.reg_status &= !VBLANK_BIT;
        return output;
    }

    fn read_PPUSTATUS_Immutable(&self) -> u8{
        return self.reg_status; // for debug only
    }

    fn read_OAMADDR(&self) -> u8{
        return 0; //TODO
    }

    fn read_OAMDATA(&self) -> u8{
        return 0;//TODO
    }

    fn read_PPUSCROLL(&self) -> u8{
        return 0; // Not Readable
    }

    fn read_PPUADDR(&self) -> u8{
        return 0; // Not Readable
    }

    fn read_PPUDATA(&mut self, rom: &Rom) -> u8{
        self.data = self.data_buffer;
        self.data_buffer = self.ppuRead(rom, self.address);

        if self.address >= 0x3F00 {
            //TODO from YT tutorial
            self.data = self.data_buffer
        }
        self.address += 1; // Auto incrementing
        return self.data;
    }

    fn read_PPUDATA_Immutable(& self) -> u8{
        return 0; //debug only
    }

    /// End Read Registers

    fn write_PPUCTRL(&mut self, value : u8){
        self.reg_ctrl = value;
        self.data_buffer = value;
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
        if self.reg_ctrl & 0b100 != 0 {
            // increment mode set
            self.address += 32;
        }
        else {
            self.address += 1;
        }
        self.data_buffer = value; // sets LSB 5 bits in the STATUS Register
    }

    fn write_PPUDATA(&mut self, value : u8){
        // do nothing?
    }


    fn get_color(& self,rom: &Rom, palette: u8, sprite_color_index: u8) -> u32{
        // 4 colors per palette, sprite_color_index indexes into the palette
        return PALETTE_LOOKUP[(self.ppuRead(rom,0x3F00 + ((palette as u16) << 2) + (sprite_color_index as u16)) & 0x3F) as usize];
    }

    pub fn tick(&mut self){
        self.pixel +=1;

        if self.pixel > 340 {
            self.pixel = 0;
            self.scanline += 1;
            if self.scanline > 261 {
                self.scanline = 0;
            }
        }

        if self.scanline == 1 {
            self.reg_status &= !VBLANK_BIT;
        }

        if self.scanline == 241 {
            self.reg_status |= VBLANK_BIT;
        }

    }
}