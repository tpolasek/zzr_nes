extern crate hex;


struct Flag {
    flag_n: u8,
    flag_v: u8,
    flag_b: u8,
    flag_d: u8,
    flag_i: u8,
    flag_z: u8,
    flag_c: u8
}
impl Flag {
    fn get_sr(&self) -> u8{
        return (self.flag_c << 0) | (self.flag_z << 1) | (self.flag_i << 2) | (self.flag_d << 3) | (self.flag_b << 4) | (1 << 5) | (self.flag_v << 6) | (self.flag_n << 7);
    }

    fn bool_to_u8(&self, set : bool) -> u8 {
        return match set {
            true => 1,
            false => 0
        };
    }

    fn set_flag_n(&mut self, set : bool) { self.flag_n = self.bool_to_u8(set); }
    fn set_flag_v(&mut self, set : bool) { self.flag_v = self.bool_to_u8(set); }
    fn set_flag_b(&mut self, set : bool) { self.flag_b = self.bool_to_u8(set); }
    fn set_flag_d(&mut self, set : bool) { self.flag_d = self.bool_to_u8(set); }
    fn set_flag_i(&mut self, set : bool) { self.flag_i = self.bool_to_u8(set); }
    fn set_flag_z(&mut self, set : bool) { self.flag_z = self.bool_to_u8(set); }
    fn set_flag_c(&mut self, set : bool) { self.flag_c = self.bool_to_u8(set); }

    fn get_flag_n(&self) -> bool { assert!(self.flag_n <= 1); return self.flag_n == 1; }
    fn get_flag_v(&self) -> bool { assert!(self.flag_v <= 1); return self.flag_v == 1; }
    fn get_flag_b(&self) -> bool { assert!(self.flag_b <= 1); return self.flag_b == 1; }
    fn get_flag_d(&self) -> bool { assert!(self.flag_d <= 1); return self.flag_d == 1; }
    fn get_flag_i(&self) -> bool { assert!(self.flag_i <= 1); return self.flag_i == 1; }
    fn get_flag_z(&self) -> bool { assert!(self.flag_z <= 1); return self.flag_z == 1; }
    fn get_flag_c(&self) -> bool { assert!(self.flag_n <= 1); return self.flag_c == 1; }

    // NV-BDIZC
    fn print(&self){
        println!("FLAGS: N/{} V/{} B/{} D/{} I/{} Z/{} C/{}",self.flag_n, self.flag_v, self.flag_b,  self.flag_d, self.flag_i, self.flag_z, self.flag_c);
    }
}

struct Bus {
    ram: [u8; 65536]
}

impl Bus {

    fn read_ram(&self, location : u16) -> u8 {
        return self.ram[location as usize];
    }

    fn write_ram(&mut self, location : u16, value : u8){
        self.ram[location as usize ] = value;
    }

    fn reset_ram(&mut self){
        for addr in 0..65535 {
            self.write_ram(addr, 0x00);
        }
    }
}

struct Opcode {
    name_format : String,
    address_mode :  fn(cpu : & mut Cpu) -> u8,
    operation : fn(cpu : & mut Cpu) -> u8,
}

fn address_mode_NOP(cpu : & mut Cpu) -> u8 {
    return 0;
}

fn address_mode_IMP(cpu : & mut Cpu) -> u8 {
    cpu.fetched = cpu.reg_a;
    return 1;
}

fn address_mode_IMM(cpu : & mut Cpu) -> u8 {
    cpu.abs_addr = cpu.pc;
    cpu.pc += 1;
    return 1;
}

fn address_mode_ZPG(cpu : & mut Cpu) -> u8 {
    cpu.abs_addr = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    return 2;
}

fn address_mode_ZPX(cpu : & mut Cpu) -> u8 {
    cpu.abs_addr = cpu.bus.read_ram(cpu.pc) as u16 + cpu.reg_x as u16;
    cpu.pc += 1;
    return 3;
}

fn address_mode_ZPY(cpu : & mut Cpu) -> u8 {
    cpu.abs_addr = cpu.bus.read_ram(cpu.pc) as u16 + cpu.reg_y as u16;
    cpu.pc += 1;
    return 3;
}

fn address_mode_ABS(cpu : & mut Cpu) -> u8 {
    let abs_addr_lo = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    let abs_addr_hi = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    cpu.abs_addr = abs_addr_hi << 8 | abs_addr_lo;
    return 3;
}

fn address_mode_ABSX(cpu : & mut Cpu) -> u8 {
    let abs_addr_lo = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    let abs_addr_hi = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;

    cpu.abs_addr = abs_addr_hi << 8 | abs_addr_lo + cpu.reg_x as u16;

    // changing page costs extra
    if cpu.abs_addr & 0xFF00 != abs_addr_hi << 8 {
        return 4;
    }
    return 3;
}

fn address_mode_ABSY(cpu : & mut Cpu) -> u8 {
    let abs_addr_lo = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    let abs_addr_hi = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;

    let addr_abs = abs_addr_hi << 8 | abs_addr_lo + cpu.reg_y as u16;
    cpu.abs_addr = addr_abs;

    // changing page costs extra
    if cpu.abs_addr & 0xFF00 != abs_addr_hi << 8 {
        return 4;
    }
    return 3;
}

fn address_mode_IND(cpu : & mut Cpu) -> u8 {
    let  ptr_addr_lo = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    let ptr_addr_hi = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;

    let ptr = ptr_addr_hi << 8 | ptr_addr_lo;
    cpu.abs_addr =  (cpu.bus.read_ram(ptr + 1) as u16) << 8  | cpu.bus.read_ram(ptr) as u16;

    return 5;
}

fn address_mode_XIND(cpu : & mut Cpu) -> u8 {
    let ptr_addr = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;

    let abs_addr_lo = cpu.bus.read_ram(ptr_addr + cpu.reg_x as u16) as u16;
    let abs_addr_hi = cpu.bus.read_ram(ptr_addr + cpu.reg_x as u16 + 1) as u16;

    cpu.abs_addr = abs_addr_hi << 8 | abs_addr_lo;

    return 5;
}

fn address_mode_INDY(cpu : & mut Cpu) -> u8 {
    let ptr_addr = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;

    let abs_addr_lo = cpu.bus.read_ram(ptr_addr ) as u16;
    let abs_addr_hi = cpu.bus.read_ram(ptr_addr + 1) as u16;

    cpu.abs_addr = (abs_addr_hi << 8 | abs_addr_lo) + cpu.reg_y as u16;

    if cpu.abs_addr & 0xFF00 != abs_addr_hi << 8{
        return 6;
    }
    return 5;
}

fn address_mode_REL(cpu : & mut Cpu) -> u8 {
    cpu.relative_addr_offset = cpu.bus.read_ram(cpu.pc) as i8;
    cpu.pc += 1;
    return 1;
}

///////////////////////////////////////////////

fn operation_NOP(cpu : & mut Cpu) -> u8 {
    return 1;
}

fn operation_ADC(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    let mut sum : u16 = cpu.reg_a as u16 + cpu.fetched as u16 + cpu.flag.flag_c as u16;
    let sum_u8 : u8 = (sum & 0xff) as u8;

    cpu.flag.set_flag_c(sum > 0xff);
    cpu.flag.set_flag_v( (!(cpu.reg_a ^ cpu.fetched) & (cpu.reg_a ^ sum_u8 )) & 0x0080 != 0);
    cpu.flag.set_flag_z(sum_u8 == 0);
    cpu.flag.set_flag_n(sum_u8 & 0b10000000 != 0);

    cpu.reg_a = sum_u8;
    println!("ADC = {:#x}", cpu.reg_a);
    return 1;
}

fn operation_LDA(cpu : & mut Cpu) -> u8 {
    cpu.fetch();
    cpu.flag.set_flag_z(cpu.fetched == 0);
    cpu.flag.set_flag_n(cpu.fetched & 0b10000000 != 0);

    cpu.reg_a = cpu.fetched;

    return 1;
}

fn operation_LDX(cpu : & mut Cpu) -> u8 {
    cpu.fetch();
    cpu.flag.set_flag_z(cpu.fetched == 0);
    cpu.flag.set_flag_n(cpu.fetched & 0b10000000 != 0);

    cpu.reg_x = cpu.fetched;

    return 1;
}

fn operation_LDY(cpu : & mut Cpu) -> u8 {
    cpu.fetch();
    cpu.flag.set_flag_z(cpu.fetched == 0);
    cpu.flag.set_flag_n(cpu.fetched & 0b10000000 != 0);

    cpu.reg_y = cpu.fetched;

    return 1;
}

fn operation_STA(cpu : & mut Cpu) -> u8 {
    cpu.bus.write_ram(cpu.abs_addr, cpu.reg_a);
    return 1;
}


// Shared function for jumps
fn operation_jump(cpu : &mut Cpu, do_jump_condition : bool) -> u8 {
    let mut cycle_cost : u8 = 1;

    if do_jump_condition {
        let updated_pc = (cpu.pc as i32 + cpu.relative_addr_offset as i32) as u16;

        if updated_pc & 0xFF00 != cpu.pc & 0xFF00 {
            cycle_cost += 1;
        }
        cpu.pc = updated_pc;
    }
    return cycle_cost;
}

// carry flag
fn operation_BCS(cpu : & mut Cpu) -> u8 {
    return operation_jump( cpu, cpu.flag.get_flag_c() );
}

fn operation_BCC(cpu : & mut Cpu) -> u8 {
    return operation_jump( cpu, !cpu.flag.get_flag_c() );
}

// zero
fn operation_BEQ(cpu : & mut Cpu) -> u8 {
    return operation_jump( cpu, cpu.flag.get_flag_z() );
}
fn operation_BNE(cpu : & mut Cpu) -> u8 {
    return operation_jump( cpu, !cpu.flag.get_flag_z() );
}

// negative
fn operation_BMI(cpu : & mut Cpu) -> u8 {
    return operation_jump( cpu, cpu.flag.get_flag_n() );
}
fn operation_BPL(cpu : & mut Cpu) -> u8 {
    return operation_jump( cpu, !cpu.flag.get_flag_n());
}

// overflow
fn operation_BVS(cpu : & mut Cpu) -> u8 {
    return operation_jump( cpu, cpu.flag.get_flag_v());
}
fn operation_BVC(cpu : & mut Cpu) -> u8 {
    return operation_jump( cpu, !cpu.flag.get_flag_v());
}



struct Cpu {
    bus: Bus,
    pc: u16,
    cycles: u8,
    reg_a: u8,
    reg_x: u8,
    reg_y: u8,
    reg_sp: u8,
    flag: Flag,
    // other
    tick_count: u64,
    opcode : Opcode,
    fetched: u8,
    abs_addr: u16,
    relative_addr_offset: i8,
}

impl Cpu {

    pub fn new(mut bus: Bus) -> Self {
        let flag = Flag {flag_n: 0, flag_v: 0, flag_b: 0, flag_d: 0, flag_i: 0, flag_z: 0, flag_c: 0};
        Self {
            bus,
            pc: 0x0600,
            cycles: 0,
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            reg_sp: 0,
            flag,
            tick_count : 0,
            fetched : 0,
            opcode : Opcode {name_format: String::from("NULL"), address_mode: address_mode_NOP, operation: operation_NOP },
            abs_addr: 0,
            relative_addr_offset: 0
        }
    }

    fn map_to_opcode(&self, value : u8) -> Opcode{
        println!("Optcode byte: {:x}", value);

        return match value {
            // ADC
            0x61 => Opcode { name_format: String::from("ADC xind"), address_mode: address_mode_XIND, operation: operation_ADC },
            0x65 => Opcode { name_format: String::from("ADC zpg"), address_mode: address_mode_ZPG, operation: operation_ADC },
            0x69 => Opcode { name_format: String::from("ADC #"), address_mode: address_mode_IMM, operation: operation_ADC },
            0x6D => Opcode { name_format: String::from("ADC abs"), address_mode: address_mode_ABS, operation: operation_ADC },
            0x71 => Opcode { name_format: String::from("ADC indy"), address_mode: address_mode_INDY, operation: operation_ADC },
            0x75 => Opcode { name_format: String::from("ADC zpgx"), address_mode: address_mode_ZPX, operation: operation_ADC },
            0x79 => Opcode { name_format: String::from("ADC absy"), address_mode: address_mode_ABSY, operation: operation_ADC },
            0x7D => Opcode { name_format: String::from("ADC absx"), address_mode: address_mode_ABSX, operation: operation_ADC },

            // STA
            0x81 => Opcode { name_format: String::from("STA xind"), address_mode: address_mode_XIND, operation: operation_STA },
            0x85 => Opcode { name_format: String::from("STA zpg"), address_mode: address_mode_ZPG, operation: operation_STA },
            0x8D => Opcode { name_format: String::from("STA abs"), address_mode: address_mode_ABS, operation: operation_STA },
            0x91 => Opcode { name_format: String::from("STA indy"), address_mode: address_mode_INDY, operation: operation_STA },
            0x95 => Opcode { name_format: String::from("STA zpgx"), address_mode: address_mode_ZPX, operation: operation_STA },
            0x96 => Opcode { name_format: String::from("STA zpgy"), address_mode: address_mode_ZPY, operation: operation_STA },
            0x99 => Opcode { name_format: String::from("STA absy"), address_mode: address_mode_ABSY, operation: operation_STA },
            0x9D => Opcode { name_format: String::from("STA absx"), address_mode: address_mode_ABSX, operation: operation_STA },

            // LDA
            0xA1 => Opcode { name_format: String::from("LDA xind"), address_mode: address_mode_XIND, operation: operation_LDA },
            0xA5 => Opcode { name_format: String::from("LDA zpg"), address_mode: address_mode_ZPG, operation: operation_LDA },
            0xA9 => Opcode { name_format: String::from("LDA #"), address_mode: address_mode_IMM, operation: operation_LDA },
            0xAD => Opcode { name_format: String::from("LDA abs"), address_mode: address_mode_ABS, operation: operation_LDA },
            0xB1 => Opcode { name_format: String::from("LDA indy"), address_mode: address_mode_INDY, operation: operation_LDA },
            0xB5 => Opcode { name_format: String::from("LDA zpgx"), address_mode: address_mode_ZPX, operation: operation_LDA },
            0xB9 => Opcode { name_format: String::from("LDA absy"), address_mode: address_mode_ABSY, operation: operation_LDA },
            0xBD => Opcode { name_format: String::from("LDA absx"), address_mode: address_mode_ABSX, operation: operation_LDA },

            // LDX
            0xA2 => Opcode { name_format: String::from("LDX #"), address_mode: address_mode_IMM, operation: operation_LDX },
            0xA6 => Opcode { name_format: String::from("LDX zpg"), address_mode: address_mode_ZPG, operation: operation_LDX },
            0xAE => Opcode { name_format: String::from("LDX abs"), address_mode: address_mode_ABS, operation: operation_LDX },
            0xB6 => Opcode { name_format: String::from("LDX zpgy"), address_mode: address_mode_ZPY, operation: operation_LDX },
            0xBE => Opcode { name_format: String::from("LDX absy"), address_mode: address_mode_ABSY, operation: operation_LDX },

            // LDY
            0xA0 => Opcode { name_format: String::from("LDY #"), address_mode: address_mode_IMM, operation: operation_LDY },
            0xA4 => Opcode { name_format: String::from("LDY zpg"), address_mode: address_mode_ZPG, operation: operation_LDY },
            0xAC => Opcode { name_format: String::from("LDY abs"), address_mode: address_mode_ABS, operation: operation_LDY },
            0xB4 => Opcode { name_format: String::from("LDY zpgx"), address_mode: address_mode_ZPX, operation: operation_LDY },
            0xBC => Opcode { name_format: String::from("LDY absx"), address_mode: address_mode_ABSX, operation: operation_LDY },

            // Branching
            0x10 => Opcode { name_format: String::from("BPL rel"), address_mode: address_mode_REL, operation: operation_BPL },
            0x30 => Opcode { name_format: String::from("BMI rel"), address_mode: address_mode_REL, operation: operation_BMI },
            0x50 => Opcode { name_format: String::from("BVC rel"), address_mode: address_mode_REL, operation: operation_BVC },
            0x70 => Opcode { name_format: String::from("BVS rel"), address_mode: address_mode_REL, operation: operation_BVS },
            0x90 => Opcode { name_format: String::from("BCC rel"), address_mode: address_mode_REL, operation: operation_BCC },
            0xB0 => Opcode { name_format: String::from("BCS rel"), address_mode: address_mode_REL, operation: operation_BCS },
            0xD0 => Opcode { name_format: String::from("BNE rel"), address_mode: address_mode_REL, operation: operation_BNE },
            0xF0 => Opcode { name_format: String::from("BEQ rel"), address_mode: address_mode_REL, operation: operation_BEQ },

            0xEA => Opcode { name_format: String::from("NULL"), address_mode: address_mode_NOP, operation: operation_NOP },
            _ => Opcode { name_format: String::from("NULL"), address_mode: address_mode_NOP, operation: operation_NOP }, //TODO remove once we implement all instructions
        }
    }

    fn fetch(&mut self){
        // IMP address mode uses a register directly (aka no fetch)
        if self.opcode.address_mode as usize != address_mode_IMP as usize {
            self.fetched = self.bus.read_ram(self.abs_addr);
        }
    }

    fn tick(&mut self) {
        self.tick_count += 1;

        if self.cycles > 0 {
            self.cycles -= 1;
        }

        if self.cycles == 0 {
            self.print();

            self.opcode = self.map_to_opcode(self.bus.read_ram(self.pc));
            self.pc += 1;

            let addr_mode_func =  self.opcode.address_mode;
            self.cycles += addr_mode_func(self);

            println!("Opcode = {}", self.opcode.name_format);
            println!("Fetched = {:#x}", self.fetched);

            let operation_func = self.opcode.operation;

            self.cycles += operation_func(self);
        }
    }

    fn print(&self) {
        self.flag.print();
        println!("Registers A/{:#x} X/{:#x} Y/{:#x} SP/{:#x} PC/{:#x}", self.reg_a, self.reg_x, self.reg_y, self.reg_sp, self.pc);
    }

    fn get_reg_sr(&self) -> u8 {
        return self.flag.get_sr();
    }

    fn run_until_halt(&mut self){
        loop {
            self.tick();
            // TODO hack to stop the program when we hit a 00 opcode
            if self.bus.read_ram(self.pc) == 0x00 {
                break;
            }
        }
    }
}


/*
Program format:
XXXX: XX XX XX XX XX XX #somecomment \n
XXXX: XX XX XX #somecomment \n
*/
fn loadProgram(bus : & mut Bus, start_address : u16, program: &str){
    bus.reset_ram(); // resets the ram

    let mut lines = program.split("\n");

    let mut hexchars = String::with_capacity(60);

    for line in lines {
        let char_vec: Vec<char> = line.chars().collect();
        let mut foundColon : bool = false;
        for c in char_vec {
            if c == ' '{
                continue;
            }
            if c == '#' {
                break;
            }

            if c == '\n'{
                break;
            }
            if c == ':' {
                foundColon = true;
                continue;
            }
            if !foundColon {
                continue;
            }

            hexchars.push(c);

        }
    }

    let hex_bytes = hex::decode(hexchars).expect("Decoding failed");

    let mut address : u16 = start_address;
    for hex in hex_bytes {
        bus.write_ram(address, hex);
        address += 1;
    }
}

fn test_LDA(){
    let mut bus = Bus { ram:  [0; 65536]};
    /*
    LDA #$00
    LDA #$ff
    */
    loadProgram(&mut bus, 0x0600, "0600: a9 00 a9 80" );
    let mut cpu = Cpu::new(bus);
    cpu.tick();
    assert!(cpu.flag.get_flag_z());
    assert!(!cpu.flag.get_flag_n());
    assert!(cpu.reg_a == 0);
    cpu.tick();
    cpu.tick();
    assert!(!cpu.flag.get_flag_z());
    assert!(cpu.flag.get_flag_n());
    assert!(cpu.reg_a == 0x80);
}


fn main() {
    test_LDA();
    /*
    let mut bus = Bus { ram:  [0; 65536]};

    loadProgram(&mut bus, 0x0600, "0000: 65 FF 69 05 6D 00 FF 69 FF # Some comment" );

    let mut cpu = Cpu::new(bus);
    cpu.run_until_halt();


    println!("{:#010b}", cpu.reg_a);

    */
}
