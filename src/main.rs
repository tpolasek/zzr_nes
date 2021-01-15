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
fn main() {


    let mut nes = Nes::new();
    nes.read_nes_rom_test();
    //nes.test_stack();
    //nes.test_loop_performance(100_000_000);
    //nes.test_cycle_cost_with_page_jump();
    //nes.run_snake();
}