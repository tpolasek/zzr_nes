#![allow(non_snake_case)]
extern crate hex;
mod nes;
use nes::Nes;
/*
--- TODO LIST ---
- Add ppU register memory (and pre-write memory) to bus (the plan is to have the cpu pre-emptively write to a cached register, that gets written to once all cpu cycles are done for an instruction)
- Setup cpu debugging GUI code (consider using https://crates.io/crates/ncurses)
- Setup classes for PPU and controller input, and main loop
*/

use std::time::Instant;


pub struct ArrayBoy {
    pub mem :  [u8; 0x100000]
}

pub struct Vecboy {
    pub mem :  Vec<u8>
}

fn read_static_mem(array: &ArrayBoy, bank: u32, location : u32) -> u8{
    return array.mem[((1 << 16)*bank + location) as usize];
}

fn read_vec_mem(array: &Vecboy, bank: u32, location : u32) -> u8{
    return array.mem[((1 << 16)*bank + location) as usize];
}


fn main() {
    let mut array = ArrayBoy{ mem : [0; 0x100000]};
    let mut vec = Vecboy { mem : Vec::with_capacity(0x100000)};

    for i in 0..0x100000{
        vec.mem.push(0x00);
    }
    let start2 = Instant::now();
    for i in 0..10 {
        for bank in 0..16 {
            for location in 0..0x10000 {
                read_vec_mem(&vec, bank, location);
            }
        }
    }
    let elapsed2 = start2.elapsed();
    println!("vec ms: {}ms", elapsed2.as_micros());

    let start = Instant::now();
    for i in 0..10 {
        for bank in 0..16 {
            for location in 0..0x10000 {
                read_static_mem(&array, bank, location);
            }
        }
    }
    let elapsed = start.elapsed();
    println!("array ms: {}ms", elapsed.as_micros());




    let mut nes = Nes::new();
    nes.run_test_suite_a();
    //nes.test_stack();
    //nes.test_loop_performance(100_000_000);
    //nes.test_cycle_cost_with_page_jump();
    //nes.run_snake();
}