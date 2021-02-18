use std::time::Instant;
use minifb::{Key, ScaleMode, Window, WindowOptions};
use std::{thread, time};
use std::io;
use console::style;


mod bus;
mod cpu;
mod ram2k;
mod controller;
mod ppu;
mod rom;

use bus::Bus;
use cpu::Cpu;
use ppu::Ppu;
use rom::Rom;
use controller::*;
use std::io::Write;

pub struct Nes {
    cpu: cpu::Cpu,
}

const WIDTH: usize = 32*16;
const HEIGHT: usize =  32*16;

impl Nes {
    pub fn new() -> Self {
        let cpu = Cpu::new();
        Self {
            cpu
        }
    }

    fn reset_state(& mut self){
        self.cpu.bus.reset_ram();
    }

    fn debugger_mode(& mut self){
        println!("{:?}",self.debugger_read_input());
    }

    fn debugger_read_input(&mut self) -> (String, u32){
        print!("\ncmd: " );
        io::stdout().flush().unwrap();

        let mut line = String::new();
        io::stdin().read_line(&mut line).expect("Failed to read line");

        let mut sects = line.split(" ");

        let mut num: u32 = 0;
        let mut command : String = String::from("");
        let mut index = 0;

        for sect in sects {
            match index {
                0 => {command = sect.trim().parse().expect("Wanted a str")},
                _ => {num = sect.trim().parse().expect("Wanted a number")},

            }
            index += 1;
        }
        return (String::from(command), num);
    }

    pub fn run_donkey(&mut self){

        use console::Term;
        let term_writer = Term::stdout();
        let term_reader = Term::stdout();
        let mut term_read_buffer: String = String::new();

        let mut button_f6_pressed : bool = false;
        let mut button_f7_pressed : bool = false;
        let mut button_f8_pressed : bool = false;
        let mut button_f9_pressed : bool = false;

        let mut query_break_point : bool = false;
        let mut break_point_addr : u16 = 0;

        let mut step_mode : bool = true;
        let mut step_next_count : u16 = 0;

        self.cpu.bus.rom.load_rom(&String::from("/home/thomas/code/rustynes/roms/donkey.nes"));
        self.cpu.reset();

        term_writer.clear_screen();
        term_writer.write_line("--------------------------------------------------------");
        term_writer.write_line("--------------------------------------------------------");
        term_writer.write_line("--------------------------------------------------------");

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

                term_writer.move_cursor_up(2);
                term_writer.write_line("Set breakpoint address in hex format 0000: ");
                term_read_buffer = term_reader.read_line().ok().unwrap();
                term_writer.clear_last_lines(2);

                break_point_addr = (u32::from_str_radix(&term_read_buffer, 16).unwrap() & 0xFFFF) as u16;
                term_writer.write_line(&format!("Set breakpoint to: 0x{:04x}", break_point_addr));
                term_writer.move_cursor_down(1);
            }

            if step_mode {
                while step_next_count > 0 {
                    self.cpu.tick();
                    self.cpu.bus.ppu.tick();
                    self.cpu.bus.ppu.tick();
                    self.cpu.bus.ppu.tick();
                    step_next_count -= 1;

                    while self.cpu.cycles != 0 {
                        self.cpu.tick();
                        self.cpu.bus.ppu.tick();
                        self.cpu.bus.ppu.tick();
                        self.cpu.bus.ppu.tick();
                    }
                }
                let mut pc_addr_scan_ahead = self.cpu.pc;
                for i in 0..16 {
                    let (instruction_str, instruction_size) = self.cpu.get_cpu_opcode_str(pc_addr_scan_ahead);
                    if i == 0 {
                        term_writer.write_line(&format!("{} {}", style(instruction_str).red(), style(self.cpu.get_cpu_state_str()).white()));
                    } else {
                        term_writer.write_line(&format!("{}", style(instruction_str).cyan()));
                    }
                    pc_addr_scan_ahead += instruction_size;
                }
                term_writer.write_line(&format!("Tick Count: {}", style(self.cpu.tick_count).yellow()));
                term_writer.move_cursor_up(16 + 1 );
            }
            else {
                // Loop mode until Vblank exit
                let mut hit_vblank = false;
                loop {
                    // Hit breakpoint
                    if self.cpu.pc == break_point_addr {
                        term_writer.move_cursor_up(2);
                        term_writer.clear_line();
                        term_writer.write_line(&format!("{} {:04x}!",style("Hit Breakpoint at").green(), style(break_point_addr).green()));
                        term_writer.move_cursor_down(1);
                        break_point_addr = 0;
                        step_mode = true;
                        step_next_count = 0;
                        break;
                    }

                    self.cpu.tick();
                    self.cpu.bus.ppu.tick();
                    self.cpu.bus.ppu.tick();
                    self.cpu.bus.ppu.tick();

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

            gwindow.update_with_buffer(&self.cpu.bus.ppu.gbuffer, 256, 240);
        }
    }

    pub fn run_test_suite_a(&mut self){
        //self.debugger_mode();

        // https://github.com/Klaus2m5/6502_65C02_functional_tests
        self.reset_state();
        self.cpu.pc = 0x400;

        let file_path = String::from("roms/6502_functional_test.bin");
        self.cpu.bus.rom.load_bin_file(&file_path);

        let ten_millis = time::Duration::from_millis(10);
        let now = time::Instant::now();


        for index in 0..100_000 {
            //thread::sleep(ten_millis);
            self.cpu.tick();
        }


        for addr in 0..20 {
            self.cpu.tick();
           //thread::sleep(ten_millis);
        }

        self.cpu.bus.print_ram(0x0200, 0xff);
    }

    pub fn run_snake(&mut self){
        self.reset_state();
        self.cpu.pc = 0x0600;
        self.cpu.bus.rom.load_hex_dump(0x0600, "0600: 20 06 06 20 38 06 20 0d 06 20 2a 06 60 a9 02 85
0610: 02 a9 04 85 03 a9 11 85 10 a9 10 85 12 a9 0f 85
0620: 14 a9 04 85 11 85 13 85 15 60 a5 fe 85 00 a5 fe
0630: 29 03 18 69 02 85 01 60 20 4d 06 20 8d 06 20 c3
0640: 06 20 19 07 20 20 07 20 2d 07 4c 38 06 a5 ff c9
0650: 77 f0 0d c9 64 f0 14 c9 73 f0 1b c9 61 f0 22 60
0660: a9 04 24 02 d0 26 a9 01 85 02 60 a9 08 24 02 d0
0670: 1b a9 02 85 02 60 a9 01 24 02 d0 10 a9 04 85 02
0680: 60 a9 02 24 02 d0 05 a9 08 85 02 60 60 20 94 06
0690: 20 a8 06 60 a5 00 c5 10 d0 0d a5 01 c5 11 d0 07
06a0: e6 03 e6 03 20 2a 06 60 a2 02 b5 10 c5 10 d0 06
06b0: b5 11 c5 11 f0 09 e8 e8 e4 03 f0 06 4c aa 06 4c
06c0: 35 07 60 a6 03 ca 8a b5 10 95 12 ca 10 f9 a5 02
06d0: 4a b0 09 4a b0 19 4a b0 1f 4a b0 2f a5 10 38 e9
06e0: 20 85 10 90 01 60 c6 11 a9 01 c5 11 f0 28 60 e6
06f0: 10 a9 1f 24 10 f0 1f 60 a5 10 18 69 20 85 10 b0
0700: 01 60 e6 11 a9 06 c5 11 f0 0c 60 c6 10 a5 10 29
0710: 1f c9 1f f0 01 60 4c 35 07 a0 00 a5 fe 91 00 60
0720: a6 03 a9 00 81 10 a2 00 a9 01 81 10 60 a2 00 ea
0730: ea ca d0 fb 60 " );

        let mut window = Window::new(
            "Snake - Press ESC to exit",
            WIDTH,
            HEIGHT,
            WindowOptions {
                resize: true,
                scale_mode: ScaleMode::UpperLeft,
                ..WindowOptions::default()
            },
        ).expect("Unable to create window");
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        let mut buffer: Vec<u32> = Vec::with_capacity(WIDTH * HEIGHT);
        let mut size = (0, 0);

        while window.is_open() && !window.is_key_down(Key::Escape) {
            let new_size = (window.get_size().0, window.get_size().1);
            if new_size != size {
                size = new_size;
                buffer.resize(size.0 * size.1, 0);
            }

            for addr in 0..400 {
                if self.cpu.pc == 0x00 {
                    break;
                }
                self.cpu.tick();
            }

            let mut index : u32 = 0;
            for i in buffer.iter_mut() {

                let y : u32 = index / (WIDTH as u32) / 16;
                let x : u32 = index % (WIDTH as u32) / 16;

                let dest : u16 = 0x0200 + (x + y*(WIDTH as u32)/16) as u16;

                if self.cpu.bus.read_ram(dest) != 0{
                    *i = (0xFF0000);
                }
                else{
                    *i = (0x00);
                }

                index+=1;
            }

            window.get_keys().map(|keys| {
                for t in keys {
                    match t {
                        Key::W => self.cpu.bus.controller.pressed(Button::UP),
                        Key::A => self.cpu.bus.controller.pressed(Button::LEFT),
                        Key::S => self.cpu.bus.controller.pressed(Button::DOWN),
                        Key::D => self.cpu.bus.controller.pressed(Button::RIGHT),
                        _ => (),
                    }
                }
            });

            window.get_keys_released().map(|keys| {
                for t in keys {
                    match t {
                        Key::W => self.cpu.bus.controller.released(Button::UP),
                        Key::A => self.cpu.bus.controller.released(Button::LEFT),
                        Key::S => self.cpu.bus.controller.released(Button::DOWN),
                        Key::D => self.cpu.bus.controller.released(Button::RIGHT),
                        _ => (),
                    }
                }
            });

            window
                .update_with_buffer(&buffer, new_size.0, new_size.1)
                .unwrap();
        }

    }

    pub fn test_stack(& mut self){
        self.reset_state();
        self.cpu.pc = 0x0600;
        self.cpu.bus.rom.load_hex_dump(0x0600,"0600: a2 00 a0 00 8a 99 00 02 48 e8 c8 c0 10 d0 f5 68 99 00 02 c8 c0 20 d0 f7" );
        self.cpu.run_until_interrupt();

        for i in 0..=0xf{
            assert!(self.cpu.bus.read_ram(0x200 + i) == i as u8);
        }
        for i in 0..0xf{
            assert!(self.cpu.bus.read_ram(0x210 + i) == (0xf-i) as u8);
        }
        //self.cpu.bus.print_ram(0x200, 0xff);
    }

    pub fn test_loop_performance(& mut self, loop_count : u32){
        self.reset_state();
        self.cpu.pc = 0x0600;
        self.cpu.bus.rom.load_hex_dump(0x0600, "0600: a2 00 a0 00 a9 00 e8 c8 69 01 18 90 f9" );
        let start = Instant::now();
        for addr in 0..loop_count {
            self.cpu.tick();
        }

        let elapsed = start.elapsed();
        println!("Ms: {}ms", elapsed.as_millis());
        println!("Clock speed: {}mhz", ((loop_count as f32) / (elapsed.as_millis() as f32) / 1000 as f32));
    }

    pub fn test_cycle_cost_with_page_jump(& mut self){
        /*
        LDA #$02
        STA $0201
        TAX
        ADC $01FF,X
        TAX
        ADC $01FF,X
        LDA #$00
        */
        self.reset_state();
        self.cpu.pc = 0x0600;
        self.cpu.bus.rom.load_hex_dump(0x0600, "0600: a9 02 8d 01 02 aa 7d ff 01 aa 7d ff 01 a9 00 " );

        self.cpu.tick();
        self.cpu.tick();
        assert!(!self.cpu.flag.get_flag_z());
        assert!(!self.cpu.flag.get_flag_n());
        assert!(self.cpu.reg_a == 2);

        assert!(self.cpu.bus.read_ram(0x0201) == 0);

        // STA $0201
        self.cpu.tick();
        self.cpu.tick();
        self.cpu.tick();
        assert!(self.cpu.bus.read_ram(0x0201) == 2);
        assert!(self.cpu.reg_x == 0);

        //TAX
        self.cpu.tick();
        self.cpu.tick();
        assert!(self.cpu.reg_x == 2);

        // ADC $01FF,X  --- 5 cycles because page jump
        self.cpu.tick();
        self.cpu.tick();
        self.cpu.tick();
        self.cpu.tick();
        self.cpu.tick();
        assert!(self.cpu.reg_a == 4);

        //TAX
        self.cpu.tick();
        self.cpu.tick();
        assert!(self.cpu.reg_x == 4);

        // ADC $01FF,X  --- 4 cycles no page jump
        self.cpu.tick();
        self.cpu.tick();
        self.cpu.tick();
        self.cpu.tick();
        self.cpu.tick();
        assert!(self.cpu.reg_a == 4);

        //    LDA #$00
        self.cpu.tick();
        self.cpu.tick();
        assert!(self.cpu.reg_a == 0);
    }
}