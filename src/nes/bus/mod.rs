use super::controller::Controller;
use super::ram2k::Ram2k;
use super::rom::Rom;
use super::ppu::Ppu;

use rand::Rng;

pub struct Bus {
    pub ram2k: Ram2k,
    pub controller: Controller,
    pub rom: Rom,
    pub ppu: Ppu,
}

impl Bus {
    pub fn new() -> Self {
        Self {
           ram2k : Ram2k { memory: [0; 0x10000] }, // todo make this actually 2k
           controller: Controller::new(),
           rom: Rom { prg1: [0; 0x10000] },
           ppu: Ppu {}
        }
    }

    pub fn read_ram(&self, location : u16) -> u8 {
        return match location {
            _ => self.rom.prg1[location as usize]
        };


    }

    pub fn write_ram(&mut self, location : u16, value : u8){
        match location {
            _ =>  self.rom.prg1[location as usize] = value
        }
    }

    pub fn reset_ram(&mut self){
        for addr in 0..=0xFFFF {
            self.write_ram(addr, 0x00);
        }
    }

    pub fn print_ram(&self, start : u16, length : u16){
        println!("\nMemory: start=0x{:04x} length=0x{:04x}", start, length);
        let mut counter: u32 = 0;
        for addr in start..start+length+1 {
            if counter % 16 == 0 {
                print!("{:04x}: ", addr);
            }
            print!("{:02x} ",self.read_ram(addr));

            if counter % 16 == 15 {
                println!();
            }

            counter+=1;
        }
    }


}