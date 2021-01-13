use std::time::Instant;
use minifb::{Key, ScaleMode, Window, WindowOptions};
use std::{thread, time};

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
        self.cpu.reset();
    }

    /*
    */

    pub fn run_test_suite_a(&mut self){
        // http://visual6502.org/wiki/index.php?title=6502TestPrograms
        self.reset_state();
        self.cpu.pc = 0xF000;
        self.cpu.bus.rom.load_hex_dump(0xF000, "0600: a9 00 8d 10 02 a9 55 8d 00 02 a9 aa 8d 01 02 a9 ff 8d 02 02 a9 6e 8d 03 02 a9 42 8d 04 02 a9 33 8d 05 02 a9 9d 8d 06 02 a9 7f 8d 07 02 a9 a5 8d 08 02 a9 1f 8d 09 02 a9 ce 8d 0a 02 a9 29 8d 0b 02 a9 42 8d 0c 02 a9 0c 8d 0d 02 a9 42 8d 0e 02 a9 55 a2 2a a0 73 85 81 a9 01 85 61 a9 7e a5 81 8d 10 09 a9 7e ad 10 09 95 56 a9 7e b5 56 84 60 91 60 a9 7e B1 60 9d ff 07 a9 7e bd ff 07 99 ff 07 a9 7e b9 ff 07 81 36 a9 7e a1 36 86 50 a6 60 a4 50 8e 13 09 a2 22 ae 13 09 8c 14 09 a0 99 ac 14 09 94 2D 96 77 a0 99 b4 2d a2 22 b6 77 a0 99 bc a0 08 a2 22 be a1 08 9d 00 02 ad 2a 02 cd 00 02 f0 03 4c ca f5 a9 fe 8d 10 02 a9 55 29 53 09 38 49 11 85 99 a9 b9 85 10 a9 e7 85 11 a9 39 85 12 a5 99 25 10 05 11 45 12 A2 10 85 99 a9 bc 85 20 a9 31 85 21 a9 17 85 22 a5 99 35 10 15 11 55 12 85 99 a9 6f 8d 10 01 a9 3c 8d 11 01 a9 27 8d 12 01 a5 99 2d 10 01 0d 11 01 4d 12 01 85 99 a9 8a 8d 20 01 a9 47 8d 21 01 a9 8f 8d 22 01 a5 99 3d 10 01 1d 11 01 5d 12 01 a0 20 85 99 a9 73 8d 30 01 a9 2a 8d 31 01 a9 f1 8d 32 01 a5 99 39 10 01 19 11 01 59 12 01 85 99 a9 70 85 30 a9 01 85 31 a9 71 85 32 a9 01 85 33 a9 72 85 34 a9 01 85 35 a9 c5 8d 70 01 a9 7c 8d 71 01 a9 a1 8d 72 01 a5 99 21 20 01 22 41 24 85 99 a9 60 85 40 a9 01 85 41 a9 61 85 42 a9 01 85 43 a9 62 85 44 a9 01 85 45 a9 37 8d 50 02 a9 23 8d 51 02 a9 9d 8d 52 02 a5 99 a0 f0 31 40 11 42 51 44 85 a9 a5 a9 cd 01 02 f0 08 a9 01 8d 10 02 4c ca f5 a9 ff a2 00 85 90 e6 90 e6 90 a5 90 a6 90 95 90 f6 90 b5 90 a6 91 9d 90 01 ee 92 01 bd 90 01 ae 92 01 9d 90 01 fe 90 01 bd 90 01 ae 93 01 9d 70 01 de 70 01 bd 70 01 ae 74 01 9d 70 01 ce 73 01 bd 70 01 ae 73 01 95 70 d6 70 b5 70 a6 72 95 70 c6 71 c6 71 a5 71 cd 02 02 f0 08 a9 02 8d 10 02 4c ca f5 a9 4b 4a 0a 85 50 06 50 06 50 46 50 a5 50 a6 50 09 c9 85 60 16 4c 56 4c 56 4c b5 4c a6 60 09 41 8d 2e 01 5e 00 01 5e 00 01 1e 00 01 bd 00 01 ae 2e 01 09 81 9d 00 01 4e 36 01 4e 36 01 0e 36 01 bd 00 01 2a 2a 6a 85 70 a6 70 09 03 95 0C 26 c0 66 c0 66 c0 b5 0c a6 c0 85 d0 36 75 36 75 76 75 a5 d0 a6 d0 9d 00 01 2e b7 01 2e b7 01 2e b7 01 6e b7 01 bd 00 01 ae b7 01 8d dd 01 3e 00 01 7e 00 01 7e 00 01 ad dd 01 cd 03 02 f0 08 a9 03 8d 10 02 4c ca f5 a9 e8 85 20 a9 f2 85 21 a9 00 09 03 4c d5 f2 09 ff 09 30 20 e1 f2 09 42 6c 20 00 09 ff 85 30 a6 30 a9 00 60 95 0d a5 40 cd 04 02 f0 08 a9 04 8d 10 02 4c ca f5 a9 35 aa ca ca e8 8a a8 88 88 c8 98 aa a9 20 9a a2 10 ba 8a 85 40 a5 40 cd 05 02 f0 08 a9 05 8d 10 02 4c ca f5 2A a9 6a 85 50 a9 6b 85 51 a9 a1 85 60 a9 a2 85 61 a9 ff 69 ff 69 ff e9 ae 85 40 a6 40 75 00 f5 01 65 60 e5 61 8d 20 01 a9 4d 8d 21 01 a9 23 6d 20 01 ed 21 01 85 f0 a6 f0 a9 64 8d 24 01 a9 62 8d 25 01 a9 26 7d 00 01 fd 01 01 85 f1 a4 f1 a9 e5 8d 28 01 a9 e9 8d 29 01 a9 34 79 00 01 f9 01 01 85 f2 a6 f2 a9 20 85 70 a9 01 85 71 a9 24 85 72 a9 01 85 73 61 41 e1 3f 85 f3 a4 f3 a9 da 85 80 a9 00 85 81 a9 dc 85 82 a9 00 85 83 a9 aa 71 80 f1 82 85 30 a5 30 cd 06 02 f0 08 a9 06 8d 10 02 4c ca f5 a9 00 85 34 a9 ff 8d 30 01 a9 99 8d 9d 01 a9 db 8d 99 01 a9 2f 85 32 a9 32 85 4f a9 30 85 33 a9 70 85 af a9 18 85 30 c9 18 f0 02 29 00 09 01 c5 30 d0 02 29 00 a2 00 cd 30 01 f0 04 85 40 a6 40 d5 27 d0 06 09 84 85 41 a6 41 29 db dd 00 01 f0 02 29 00 85 42 a4 42 29 00 d9 00 01 d0 02 09 0f 85 43 a6 43 09 24 c1 40 f0 02 09 7f 85 44 a4 44 49 0f d1 33 d0 04 a5 44 85 15 a5 15 cd 07 02 f0 08 a9 07 8d 10 02 4c ca f5 a9 a5 85 20 8d 20 01 a9 5a 85 21 a2 a5 e0 a5 f0 02 a2 01 e4 20 f0 02 a2 02 ec 20 01 f0 02 a2 03 86 30 a4 30 c0 a5 f0 02 a0 04 c4 20 f0 02 a0 05 cc 20 01 f0 02 a0 06 84 31 a5 31 24 20 d0 02 a9 07 2c 20 01 d0 02 a9 08 24 21 d0 02 85 42 a5 42 cd 08 02 f0 08 a9 08 8d 10 02 4c ca f5 a9 54 85 32 a9 b3 85 a1 a9 87 85 43 a2 a1 10 02 a2 32 b4 00 10 04 a9 05 a6 a1 30 02 e9 03 30 02 a9 41 49 30 85 32 75 00 50 02 a9 03 85 54 b6 00 75 51 50 02 a9 e5 75 40 70 04 99 01 00 65 55 70 02 a9 00 69 f0 90 04 85 60 65 43 90 02 a9 ff 65 54 b0 04 69 87 a6 60 b0 02 a9 00 95 73 a5 80 cd 09 02 f0 08 a9 09 8d 10 02 4c ca f5 69 00 a9 99 69 87 18 ea 90 04 69 60 69 93 38 ea 90 01 b8 50 02 a9 00 69 ad ea 85 30 a5 30 cd 0a 02 f0 08 a9 0a 8d 10 02 4c ca f5 69 01 a9 27 69 01 38 08 18 28 69 00 48 a9 00 68 85 30 a5 30 cd 0b 02 f0 08 a9 0b 8d 10 02 4c ca f5 18 a9 42 90 04 85 33 b0 0a a9 f5 48 a9 61 48 38 08 18 40 a5 33 cd 0c 02 f0 08 a9 0c 8d 10 02 4c ca f5 69 01 78 f8 08 68 85 20 58 d8 08 68 65 20 85 21 a5 21 cd 0d 02 f0 08 a9 0d 8d 10 02 4c ca f5 4c a9 f5 a9 41 85 60 40 a9 ff 85 60 00 00 e6 60 a5 60 cd 0e 02 f0 08 a9 0e 8d 10 02 4c ca f5 a9 fe cd 10 02 d0 03 ee 10 02 4c ca f5 ");
        self.cpu.bus.write_ram(0xFFFE, 0xa4);
        self.cpu.bus.write_ram(0xFFFF, 0xf5);

        let ten_millis = time::Duration::from_millis(100);
        let now = time::Instant::now();


        while self.cpu.pc != 0xF0BB {
            self.cpu.tick(true);
        }

        self.cpu.bus.print_ram(0xf0bb, 0xff);

        for addr in 0..20 {
            self.cpu.tick(true);
           thread::sleep(ten_millis);
        }

        self.cpu.bus.print_ram(0x0210, 0xff);

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
                self.cpu.tick(false);
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
        self.cpu.run_until_interrupt(false);

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
            self.cpu.tick(false);
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

        self.cpu.tick(false);
        self.cpu.tick(false);
        assert!(!self.cpu.flag.get_flag_z());
        assert!(!self.cpu.flag.get_flag_n());
        assert!(self.cpu.reg_a == 2);

        assert!(self.cpu.bus.read_ram(0x0201) == 0);

        // STA $0201
        self.cpu.tick(false);
        self.cpu.tick(false);
        self.cpu.tick(false);
        assert!(self.cpu.bus.read_ram(0x0201) == 2);
        assert!(self.cpu.reg_x == 0);

        //TAX
        self.cpu.tick(false);
        self.cpu.tick(false);
        assert!(self.cpu.reg_x == 2);

        // ADC $01FF,X  --- 5 cycles because page jump
        self.cpu.tick(false);
        self.cpu.tick(false);
        self.cpu.tick(false);
        self.cpu.tick(false);
        self.cpu.tick(false);
        assert!(self.cpu.reg_a == 4);

        //TAX
        self.cpu.tick(false);
        self.cpu.tick(false);
        assert!(self.cpu.reg_x == 4);

        // ADC $01FF,X  --- 4 cycles no page jump
        self.cpu.tick(false);
        self.cpu.tick(false);
        self.cpu.tick(false);
        self.cpu.tick(false);
        self.cpu.tick(false);
        assert!(self.cpu.reg_a == 4);

        //    LDA #$00
        self.cpu.tick(false);
        self.cpu.tick(false);
        assert!(self.cpu.reg_a == 0);
    }
}