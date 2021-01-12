use std::time::Instant;

mod bus;
mod cpu;

pub struct Nes {
    cpu: cpu::Cpu,
}

impl Nes {
    pub fn new() -> Self {
        let mut bus = bus::Bus { ram:  [0; 65536]};
        let cpu = cpu::Cpu::new(bus);

        Self {
            cpu
        }
    }

    fn reset_state(& mut self){
        self.cpu.bus.reset_ram();
        self.cpu.reset();
    }

    pub fn test_stack(& mut self){
        self.reset_state();
        self.cpu.pc = 0x0600;
        self.cpu.bus.loadProgram(0x0600, "0600: a2 00 a0 00 8a 99 00 02 48 e8 c8 c0 10 d0 f5 68 99 00 02 c8 c0 20 d0 f7" );
        self.cpu.run_until_interrupt(true);
        self.cpu.bus.print_ram(0x200, 0xff);
    }

    pub fn test_loop_performance(& mut self, loop_count : u32){
        self.reset_state();
        self.cpu.pc = 0x0600;
        self.cpu.bus.loadProgram( 0x0600, "0600: a2 00 a0 00 a9 00 e8 c8 69 01 18 90 f9" );
        let start = Instant::now();
        for addr in 0..loop_count {
            self.cpu.tick(false);
        }

        let elapsed = start.elapsed();
        println!("Ms: {}ms", elapsed.as_millis());
        println!("Clock speed: {}mhz", ((loop_count as f32) / (elapsed.as_millis() as f32) / 1000 as f32));
    }
}