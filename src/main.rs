#![allow(non_snake_case)]
extern crate hex;
mod nes;
use nes::Nes;

/*
use std::time::Instant;
use minifb::{Key, ScaleMode, Window, WindowOptions};

fn test_cycle_cost_with_page_jump(){
    let mut bus = bus::Bus { ram:  [0; 65536]};
    /*
    LDA #$02
    STA $0201
    TAX
    ADC $01FF,X
    TAX
    ADC $01FF,X
    LDA #$00
    */
    bus.loadProgram(0x0600, "0600: a9 02 8d 01 02 aa 7d ff 01 aa 7d ff 01 a9 00 " );
    let mut cpu = cpu::Cpu::new(bus);

    cpu.tick(true);
    cpu.tick(true);
    assert!(!cpu.flag.get_flag_z());
    assert!(!cpu.flag.get_flag_n());
    assert!(cpu.reg_a == 2);

    assert!(cpu.bus.read_ram(0x0201) == 0);

    // STA $0201
    cpu.tick(true);
    cpu.tick(true);
    cpu.tick(true);
    assert!(cpu.bus.read_ram(0x0201) == 2);
    assert!(cpu.reg_x == 0);

    //TAX
    cpu.tick(true);
    cpu.tick(true);
    assert!(cpu.reg_x == 2);

    // ADC $01FF,X  --- 5 cycles because page jump
    cpu.tick(true);
    cpu.tick(true);
    cpu.tick(true);
    cpu.tick(true);
    cpu.tick(true);
    assert!(cpu.reg_a == 4);

    //TAX
    cpu.tick(true);
    cpu.tick(true);
    assert!(cpu.reg_x == 4);

    // ADC $01FF,X  --- 4 cycles no page jump
    cpu.tick(true);
    cpu.tick(true);
    cpu.tick(true);
    cpu.tick(true);
    cpu.tick(true);
    assert!(cpu.reg_a == 4);

    //    LDA #$00
    cpu.tick(true);
    cpu.tick(true);
    assert!(cpu.reg_a == 0);
}


fn test_Snake(){
    let mut bus = bus::Bus { ram:  [0; 65536]};

    bus.loadProgram( 0x0600, "0600: 20 06 06 20 38 06 20 0d 06 20 2a 06 60 a9 02 85
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

    let mut cpu = cpu::Cpu::new(bus);

    let mut window = Window::new(
        "Snake - Press ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            scale_mode: ScaleMode::UpperLeft,
            ..WindowOptions::default()
        },
    )
        .expect("Unable to create window");

    // Limit to max ~60 fps update rate
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
            if cpu.pc == 0x00 {
                break;
            }
            cpu.tick(false);
        }

        let mut index : u32 = 0;
        for i in buffer.iter_mut() {

            let y : u32 = index / (WIDTH as u32) / 16;
            let x : u32 = index % (WIDTH as u32) / 16;

            let dest : u16 = 0x0200 + (x + y*(WIDTH as u32)/16) as u16;

            if cpu.bus.read_ram(dest) != 0{
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
                    Key::W => cpu.bus.write_ram(0x00FF, 0x77),
                    Key::A => cpu.bus.write_ram(0x00FF, 0x61),
                    Key::S => cpu.bus.write_ram(0x00FF, 0x73),
                    Key::D => cpu.bus.write_ram(0x00FF, 0x64),
                    _ => (),
                }
            }
        });

        window.get_keys_released().map(|keys| {
            for t in keys {
                match t {
                    _ => (),
                }
            }
        });

        window
            .update_with_buffer(&buffer, new_size.0, new_size.1)
            .unwrap();
    }
}
const WIDTH: usize = 32*16;
const HEIGHT: usize =  32*16;

/*
--- TODO LIST ---
- Add ppU register memory (and pre-write memory) to bus (the plan is to have the cpu pre-emptively write to a cached register, that gets written to once all cpu cycles are done for an instruction)
- Setup cpu debugging GUI code (consider using https://crates.io/crates/ncurses)
- Setup classes for PPU and controller input, and main loop
*/

fn main() {
    //test_cycle_cost_with_page_jump();
    //test_Stack();
    //test_Snake();
    test_loop_performance(100_000_000);
}

*/
fn main() {
    let mut nes = Nes::new();
    nes.test_stack();
    nes.test_loop_performance(100_000_000);
}