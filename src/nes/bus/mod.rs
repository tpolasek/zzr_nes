use super::controller::Controller;
use super::ram2k::Ram2k;
use super::ram2k::WorkRam;
use super::rom::Rom;
use super::ppu::Ppu;

use rand::Rng;

pub struct Bus {
    pub ram2k: Ram2k,
    pub workram: WorkRam,
    pub controller: Controller,
    pub rom: Rom,
    pub ppu: Ppu,
}

impl Bus {
    pub fn new() -> Self {
        Self {
           ram2k : Ram2k { memory: [0; 0x800] },
           workram  : WorkRam { memory: [0; 0x2000]},
           controller: Controller::new(),
           rom: Rom::new(),
           ppu: Ppu {}
        }
    }

    /*
    This function does not have any side effects
    */
    pub fn read_ram_opcode(&self, location : u16) -> u8 {
        // mapper 0
        match location {
            0x0000..=0x1FFF => {
                return self.ram2k.memory[(location & 0x7FF) as usize];
            },
            0x2000..=0x3FFF => {
                return self.ppu.read((location & 0x7) as u8);
            },
            0x4000..=0x5FFF => {
                return 0;
            },
            0x6000..=0x7FFF => {
                return self.workram.memory[(location & 0x1FFF) as usize];
            },
            0x8000..=0xFFFF => {
                if self.rom.prg_bank_count > 1 {
                    return self.rom.prg[(location & 0x7FFF) as usize]; //32kb mask
                }
                return self.rom.prg[(location & 0x3FFF) as usize]; //16kb mask
            }
        }
        return 0;
    }

    pub fn read_ram(&mut self, location : u16) -> u8 {
        // mapper 0
        match location {
            0x0000..=0x1FFF => {
                return self.ram2k.memory[(location & 0x7FF) as usize];
            },
            0x2000..=0x3FFF => {
                return self.ppu.read((location & 0x7) as u8);
            },
            0x4000..=0x4013 => {
                // APU
            },
            0x4014 => {
                // OAMDMA
            },
            0x4015 => {
                // SND_CHN
            },
            0x4016 => {
                // Controller 1
                return self.controller.read();
            },
            0x4017 => {
                // Controller 2
                return 0x40;
            }
            0x4018..=0x401F => {
                //Unused APU and I/O functionality
            },
            0x4020..=0x5FFF => {
                //empty on mapper 0?
            },
            0x6000..=0x7FFF => {
                return self.workram.memory[(location & 0x1FFF) as usize];
            },
            0x8000..=0xFFFF => {
                if self.rom.prg_bank_count > 1 {
                    return self.rom.prg[(location & 0x7FFF) as usize]; //32kb mask
                }
                return self.rom.prg[(location & 0x3FFF) as usize]; //16kb mask
            }
        }
        return 0;
    }

    pub fn write_ram(&mut self, location : u16, value : u8){
        // Mapper 0
        match location {
            0x0000..=0x1FFF => {
                self.ram2k.memory[(location & 0x7FF) as usize] = value;
            },
            0x2000..=0x3FFF => {
                self.ppu.write((location & 0x7) as u8, value);
            },
            0x4000..=0x4013 => {
                // APU
            },
            0x4014 => {
                // OAMDMA
            },
            0x4015 => {
                // SND_CHN
            },
            0x4016 => {
                // Controller 1
                self.controller.write(value);
            },
            0x4017 => {
                // Controller 2
            }
            0x4018..=0x401F => {
                //Unused APU and I/O functionality
            },
            0x4020..=0x5FFF => {
                //empty on mapper 0?
            },
            0x6000..=0x7FFF => {
                self.workram.memory[(location & 0x1FFF) as usize] = value;
            },
            0x8000..=0xFFFF => {
                if self.rom.prg_bank_count > 1 {
                    self.rom.prg[(location & 0x7FFF) as usize] = value; //32kb mask
                }
                self.rom.prg[(location & 0x3FFF) as usize] = value; //16kb mask
            }
        }
    }

    pub fn reset_ram(&mut self){
        for addr in 0..=0xFFFF {
            self.write_ram(addr, 0x00);
        }
    }

    pub fn print_ram(&mut self, start : u16, length : u16){
        println!("\nMemory: start=0x{:04x} length=0x{:04x}", start, length);
        let mut counter: u32 = 0;
        for addr in start..=(start+length) {
            if counter % 16 == 0 {
                print!("{:04x}: ", addr);
            }
            print!("{:02x} ",self.read_ram(addr));

            if counter % 16 == 15 {
                println!();
            }

            counter+=1;
        }
        println!();
    }


}
