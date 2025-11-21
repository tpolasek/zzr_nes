use minifb::{Key, ScaleMode, Window, WindowOptions};
use console::style;
use crate::nes::controller::Button;

mod bus;
mod cpu;
mod ram2k;
mod controller;
mod ppu;
mod rom;
mod cpu_flag;
mod debugger;

use cpu::Cpu;
use debugger::Debugger;


pub struct Nes {
    cpu: cpu::Cpu,
    debugger: debugger::Debugger
}

impl Nes {
    pub fn new() -> Self {
        let cpu = Cpu::new();
        let debugger = Debugger::new();
        Self {
            cpu,
            debugger
        }
    }

    #[inline(always)]
    fn execute_cpu_ppu(&mut self){
        if self.cpu.bus.dma_cycles > 0 {
            self.cpu.bus.dma_cycles -= 1;
        }
        else {
            if self.cpu.bus.ppu.get_and_reset_nmi_triggered(){
                self.cpu.trigger_nmi(); // applies nmi instantly, but adds the clock cost
            }
            self.cpu.tick();
        }

        self.cpu.bus.ppu.tick();
        self.cpu.bus.ppu.tick();
        self.cpu.bus.ppu.tick();
    }

    pub fn execute_rom(&mut self, filename: &String){

        use console::Term;
        let term_writer = Term::stdout();
        let term_reader = Term::stdout();
        let mut term_read_buffer: String = String::new();

        let mut button_f6_pressed : bool = false;
        let mut button_f7_pressed : bool = false;
        let mut button_f8_pressed : bool = false;
        let mut button_f9_pressed : bool = false;

        let mut query_break_point : bool = false;

        let mut step_mode : bool = true;
        let mut step_next_count : u16 = 0;

        self.cpu.bus.rom.load_rom(filename);
        self.cpu.reset();

        term_writer.clear_screen().ok();
        term_writer.write_line("--------------------------------------------------------").ok();
        term_writer.write_line("--------------------------------------------------------").ok();
        term_writer.write_line("--------------------------------------------------------").ok();

        let mut gwindow = Window::new(
            "ZZR",
            256,
            240,
            WindowOptions {
                resize: false,
                scale_mode: ScaleMode::UpperLeft,
                ..WindowOptions::default()
            },
        ).expect("Unable to create window");
        gwindow.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
        gwindow.set_position(1920/2 - 256,1080/2 - 240);
        while gwindow.is_open() && !gwindow.is_key_down(Key::Escape) {
            if query_break_point {
                query_break_point = false;

                term_writer.move_cursor_up(2).ok();
                term_writer.write_line("Set breakpoint address in hex format 0000: ").ok();
                term_read_buffer = term_reader.read_line().ok().unwrap();
                term_writer.clear_last_lines(2).ok();


                // Set breakpoint here
                let break_point_addr = (u32::from_str_radix(&term_read_buffer, 16).unwrap() & 0xFFFF) as u16;
                self.debugger.set_breakpoint(break_point_addr,None);

                term_writer.write_line(&format!("Set breakpoint to: 0x{:04x}", break_point_addr)).ok();
                term_writer.move_cursor_down(1).ok();
            }
            if step_mode {
                while step_next_count > 0 {
                    self.execute_cpu_ppu();
                    step_next_count -= 1;

                    while !self.cpu.ready_to_execute_next_instruction() {
                        self.execute_cpu_ppu();
                    }
                }
                let mut pc_addr_scan_ahead = self.cpu.pc;
                for i in 0..16 {
                    let (instruction_str, instruction_size) = self.cpu.get_cpu_opcode_str(pc_addr_scan_ahead);
                    if i == 0 {
                        term_writer.write_line(&format!("{} {}", style(instruction_str).red(), style(self.cpu.get_cpu_state_str()).white())).ok();
                    } else {
                        term_writer.write_line(&format!("{}", style(instruction_str).cyan())).ok();
                    }
                    pc_addr_scan_ahead += instruction_size;
                }
                term_writer.write_line(&format!("Tick Count: {}", style(self.cpu.tick_count).yellow())).ok();
                term_writer.move_cursor_up(16 + 1 ).ok();
            }
            else {
                // Loop mode until Vblank exit
                let mut hit_vblank = false;
                loop {
                    // Hit breakpoint
                    if self.debugger.hit_breakpoint(self.cpu.pc) {
                        term_writer.move_cursor_up(2).ok();
                        term_writer.clear_line().ok();
                        //term_writer.write_line(&format!("{} {:04x}!",style("Hit Breakpoint at").green(), style(break_point_addr).green())).ok();
                        term_writer.move_cursor_down(1).ok();
                        step_mode = true;
                        step_next_count = 0;
                        break;
                    }

                    self.execute_cpu_ppu();

                    if self.cpu.bus.ppu.is_vblank() {
                        hit_vblank = true;
                    } else if hit_vblank {
                        // Just exited the vblank
                        break;
                    }
                }
            }

            gwindow.get_keys().map(|keys| {
                for t in keys {
                    match t {
                        Key::W => self.cpu.bus.controller.pressed(Button::UP),
                        Key::A => self.cpu.bus.controller.pressed(Button::LEFT),
                        Key::S => self.cpu.bus.controller.pressed(Button::DOWN),
                        Key::D => self.cpu.bus.controller.pressed(Button::RIGHT),
                        Key::K => self.cpu.bus.controller.pressed(Button::A),
                        Key::L => self.cpu.bus.controller.pressed(Button::B),
                        Key::F6 => {if !button_f6_pressed {step_mode = true; step_next_count = 400; button_f6_pressed = true;}},
                        Key::F7 => {if !button_f7_pressed {step_mode = true; step_next_count = 1; button_f7_pressed = true;}},
                        Key::F8 => {if !button_f8_pressed {step_mode = false; button_f8_pressed = true;}},
                        Key::F9 => {if !button_f9_pressed {query_break_point = true; button_f9_pressed = true;}},
                        Key::RightShift => self.cpu.bus.controller.pressed(Button::SELECT),
                        Key::Enter => self.cpu.bus.controller.pressed(Button::START),
                        _ => (),
                    }
                }
            });

            gwindow.get_keys_released().map(|keys| {
                for t in keys {
                    match t {
                        Key::W => self.cpu.bus.controller.released(Button::UP),
                        Key::A => self.cpu.bus.controller.released(Button::LEFT),
                        Key::S => self.cpu.bus.controller.released(Button::DOWN),
                        Key::D => self.cpu.bus.controller.released(Button::RIGHT),
                        Key::K => self.cpu.bus.controller.released(Button::A),
                        Key::L => self.cpu.bus.controller.released(Button::B),
                        Key::F6 => {button_f6_pressed = false;},
                        Key::F7 => {button_f7_pressed = false;},
                        Key::F8 => {button_f8_pressed = false;},
                        Key::F9 => {button_f9_pressed = false;},
                        Key::RightShift => self.cpu.bus.controller.released(Button::SELECT),
                        Key::Enter => self.cpu.bus.controller.released(Button::START),
                        _ => (),
                    }
                }
            });

            gwindow.update_with_buffer(&self.cpu.bus.ppu.gbuffer, 256, 240).expect("Failed to update gwindow buffer");
        }
    }

}