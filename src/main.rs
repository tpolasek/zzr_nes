#![allow(non_snake_case)]
extern crate hex;
mod nes;
use crate::nes::Nes;
use eframe::NativeOptions;
/*
--- TODO LIST ---
- Add ppU register memory (and pre-write memory) to bus (the plan is to have the cpu pre-emptively write to a cached register, that gets written to once all cpu cycles are done for an instruction)
- Setup cpu debugging GUI code (consider using https://crates.io/crates/ncurses)
- Setup classes for PPU and controller input, and main loop
*/
fn main() {
    let opts: NativeOptions = NativeOptions {
        // Configure the viewport (main window) settings
        viewport: egui::ViewportBuilder::default().with_inner_size([850.0, 768.0]), // Set width and height here
        ..Default::default()
    };
    let filename = &String::from("/Users/thomas/code/zzr_nes/roms/all_instrs.nes");
    let filename_owned = filename.clone();
    let _result = eframe::run_native(
        "ZNES",
        opts,
        Box::new(move |cc| Box::new(Nes::new(&cc.egui_ctx, &filename_owned))),
    );

    //nes.test_stack();
    //nes.test_loop_performance(100_000_000);
    //nes.test_cycle_cost_with_page_jump();
    //nes.run_snake();
}
