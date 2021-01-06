#![allow(non_snake_case)]
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
    cycles : u8
}


// GOOD
fn address_mode_ACC(cpu : & mut Cpu) -> u8 {
    cpu.fetched = cpu.reg_a;
    return 0;
}

// GOOD
fn address_mode_IMM(cpu : & mut Cpu) -> u8 {
    cpu.abs_addr = cpu.pc;
    cpu.pc += 1;
    return 0;
}

// GOOD
fn address_mode_ZPG(cpu : & mut Cpu) -> u8 {
    cpu.abs_addr = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    return 0;
}

// GOOD
fn address_mode_ZPX(cpu : & mut Cpu) -> u8 {
    cpu.abs_addr = cpu.bus.read_ram(cpu.pc) as u16 + cpu.reg_x as u16;
    cpu.abs_addr &= 0x00FF;
    cpu.pc += 1;
    return 0;
}

// GOOD
fn address_mode_ZPY(cpu : & mut Cpu) -> u8 {
    cpu.abs_addr = cpu.bus.read_ram(cpu.pc) as u16 + cpu.reg_y as u16;
    cpu.abs_addr &= 0x00FF;
    cpu.pc += 1;
    return 0;
}

// GOOD
fn address_mode_ABS(cpu : & mut Cpu) -> u8 {
    let abs_addr_lo = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    let abs_addr_hi = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    cpu.abs_addr = abs_addr_hi << 8 | abs_addr_lo;
    return 0;
}

// GOOD
fn address_mode_ABSX(cpu : & mut Cpu) -> u8 {
    let abs_addr_lo = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    let abs_addr_hi = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;

    let temp : u32 = (abs_addr_hi << 8 | abs_addr_lo) as u32 + cpu.reg_x as u32;
    cpu.abs_addr = (temp & 0xFFFF) as u16; // Assumed this is correct

    // changing page costs extra
    if cpu.abs_addr & 0xFF00 != abs_addr_hi << 8 {
        return 1;
    }
    return 0;
}

// GOOD
fn address_mode_ABSY(cpu : & mut Cpu) -> u8 {
    let abs_addr_lo = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    let abs_addr_hi = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;

    let temp : u32 = (abs_addr_hi << 8 | abs_addr_lo) as u32 + cpu.reg_y as u32;
    cpu.abs_addr = (temp & 0xFFFF) as u16; // Assumed this is correct

    // changing page costs extra
    if cpu.abs_addr & 0xFF00 != abs_addr_hi << 8 {
        return 1;
    }
    return 0;
}

// MAYBE ? kind of complex
fn address_mode_IND(cpu : & mut Cpu) -> u8 {
    let  ptr_addr_lo = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    let ptr_addr_hi = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;

    let ptr : u16 = (ptr_addr_hi << 8 | ptr_addr_lo) as u16;
    let ptr2 : u16 = ((ptr & 0xFF00) | ((ptr + 1) & 0x00FF)) as u16; //replicate 6502 page-boundary wraparound bug

    cpu.abs_addr =  (cpu.bus.read_ram(ptr2) as u16) << 8  | cpu.bus.read_ram(ptr) as u16;

    return 0;
}

// GOOD
fn address_mode_XIND(cpu : & mut Cpu) -> u8 {
    let ptr_addr = (cpu.bus.read_ram(cpu.pc) as u16 + cpu.reg_x as u16) & 0xFF;
    cpu.pc += 1;

    let abs_addr_lo = cpu.bus.read_ram(ptr_addr) as u16;
    let abs_addr_hi = cpu.bus.read_ram(ptr_addr + 1) as u16;

    cpu.abs_addr = (abs_addr_hi << 8 | abs_addr_lo);

    return 0;
}

// GOOD
fn address_mode_INDY(cpu : & mut Cpu) -> u8 {
    let ptr_addr = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;

    let abs_addr_lo = cpu.bus.read_ram(ptr_addr ) as u16;
    let abs_addr_hi = cpu.bus.read_ram(ptr_addr + 1) as u16;

    cpu.abs_addr = (abs_addr_hi << 8 | abs_addr_lo) + cpu.reg_y as u16;

    if cpu.abs_addr & 0xFF00 != abs_addr_hi << 8{
        return 1;
    }
    return 0;
}

fn address_mode_REL(cpu : & mut Cpu) -> u8 {
    cpu.relative_addr_offset = cpu.bus.read_ram(cpu.pc) as i8;
    cpu.pc += 1;
    return 0;
}

///////////////////////////////////////////////

fn set_z_n_flags(cpu : & mut Cpu, val : u8){
    cpu.flag.set_flag_z(val == 0);
    cpu.flag.set_flag_n(val & 0x80 != 0);
}

fn operation_NOP(_cpu : & mut Cpu) -> u8 {
    return 0;
}

fn operation_ADC(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    let sum : u16 = cpu.reg_a as u16 + cpu.fetched as u16 + cpu.flag.flag_c as u16;
    let sum_u8 : u8 = (sum & 0xff) as u8;

    cpu.flag.set_flag_c(sum > 0xff);
    cpu.flag.set_flag_v( (!(cpu.reg_a ^ cpu.fetched) & (cpu.reg_a ^ sum_u8 )) & 0x0080 != 0);
    set_z_n_flags(cpu, sum_u8);

    cpu.reg_a = sum_u8;
    println!("ADC = {:#x}", cpu.reg_a);
    return 0;
}

fn operation_AND(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    cpu.reg_a &= cpu.fetched;

    set_z_n_flags(cpu, cpu.reg_a);
    return 0;
}

fn operation_ASL(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    cpu.flag.set_flag_c(cpu.fetched & 0x80 != 0);

    cpu.reg_a = cpu.fetched << 1;

    set_z_n_flags(cpu, cpu.reg_a);
    return 0;
}

fn operation_BIT(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    cpu.flag.set_flag_z(cpu.reg_a & cpu.fetched == 0x00);
    cpu.flag.set_flag_v(cpu.fetched & (1 << 6) != 0);
    cpu.flag.set_flag_n(cpu.fetched & (1 << 7) != 0);
    return 0;
}


fn operation_LDA(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    cpu.reg_a = cpu.fetched;

    set_z_n_flags(cpu, cpu.reg_a);
    return 0;
}

fn operation_LDX(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    cpu.reg_x = cpu.fetched;

    set_z_n_flags(cpu, cpu.reg_x);

    return 0;
}

fn operation_LDY(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    cpu.reg_y = cpu.fetched;

    set_z_n_flags(cpu, cpu.reg_y);

    return 0;
}

fn operation_STA(cpu : & mut Cpu) -> u8 {
    cpu.bus.write_ram(cpu.abs_addr, cpu.reg_a);
    return 0;
}


fn operation_CLC(cpu : & mut Cpu) -> u8 {
    cpu.flag.set_flag_c(false);
    return 0;
}

fn operation_CLD(cpu : & mut Cpu) -> u8 {
    cpu.flag.set_flag_d(false);
    return 0;
}

fn operation_CLI(cpu : & mut Cpu) -> u8 {
    cpu.flag.set_flag_i(false);
    return 0;
}

fn operation_CLV(cpu : & mut Cpu) -> u8 {
    cpu.flag.set_flag_v(false);
    return 0;
}

fn operation_DEC(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    let mut temp :u8  = 0;
    if cpu.fetched == 0x00{
        temp = 0xFF;
    }
    else{
        temp = cpu.fetched - 1;
    }

    cpu.bus.write_ram(cpu.abs_addr, temp);

    set_z_n_flags(cpu, temp);
    return 0;
}

fn operation_DEX(cpu : & mut Cpu) -> u8 {
    if cpu.reg_x == 0x00{
        cpu.reg_x = 0xFF;
    }
    else{
        cpu.reg_x -= 1;
    }
    set_z_n_flags(cpu, cpu.reg_x);
    return 0;
}

fn operation_DEY(cpu : & mut Cpu) -> u8 {
    if cpu.reg_y == 0x00{
        cpu.reg_y = 0xFF;
    }
    else{
        cpu.reg_y -= 1;
    }
    set_z_n_flags(cpu, cpu.reg_y);
    return 0;
}

fn operation_EOR(cpu : & mut Cpu) -> u8 {
    cpu.fetch();
    cpu.reg_a ^= cpu.fetched;

    set_z_n_flags(cpu, cpu.reg_a);
    return 0;
}

fn operation_INC(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    let mut temp :u8  = 0;
    if cpu.fetched == 0xFF{
        temp = 0x00;
    }
    else{
        temp = cpu.fetched + 1;
    }

    cpu.bus.write_ram(cpu.abs_addr, temp);

    set_z_n_flags(cpu, temp);
    return 0;
}

fn operation_INX(cpu : & mut Cpu) -> u8 {
    if cpu.reg_x == 0xFF{
        cpu.reg_x = 0x00;
    }
    else{
        cpu.reg_x += 1;
    }
    set_z_n_flags(cpu, cpu.reg_x);
    return 0;
}

fn operation_INY(cpu : & mut Cpu) -> u8 {
    if cpu.reg_y == 0xFF{
        cpu.reg_y = 0x00;
    }
    else{
        cpu.reg_y += 1;
    }
    set_z_n_flags(cpu, cpu.reg_y);
    return 0;
}

// Shared function for jumps
fn operation_jump(cpu : &mut Cpu, do_jump_condition : bool) -> u8 {
    let mut cycle_cost : u8 = 0;

    if do_jump_condition {
        cycle_cost +=1;

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

    pub fn new(bus: Bus) -> Self {
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
            opcode : Opcode {name_format: String::from("NULL"), address_mode: address_mode_ACC, operation: operation_NOP, cycles: 2 },
            abs_addr: 0,
            relative_addr_offset: 0
        }
    }

    fn map_to_opcode(&self, value : u8) -> Opcode{
        println!("Optcode byte: {:x}", value);

        return match value {
            // ASL
            0x06 => Opcode { name_format: String::from("ASL zpg"), address_mode: address_mode_ZPG, operation: operation_ASL, cycles: 0 },
            0x0A => Opcode { name_format: String::from("ASL zpg"), address_mode: address_mode_ACC, operation: operation_ASL, cycles: 0 },
            0x0E => Opcode { name_format: String::from("ASL abs"), address_mode: address_mode_ABS, operation: operation_ASL, cycles: 0 },
            0x16 => Opcode { name_format: String::from("ASL zpgx"), address_mode: address_mode_ZPX, operation: operation_ASL, cycles: 0 },
            0x1E => Opcode { name_format: String::from("ASL absx"), address_mode: address_mode_ABSX, operation: operation_ASL, cycles: 0 },

            // AND
            0x20 => Opcode { name_format: String::from("AND xind"), address_mode: address_mode_XIND, operation: operation_AND, cycles: 0 },
            0x25 => Opcode { name_format: String::from("AND zpg"), address_mode: address_mode_ZPG, operation: operation_AND, cycles: 0 },
            0x29 => Opcode { name_format: String::from("AND #"), address_mode: address_mode_IMM, operation: operation_AND, cycles: 0 },
            0x2D => Opcode { name_format: String::from("AND abs"), address_mode: address_mode_ABS, operation: operation_AND, cycles: 0 },
            0x31 => Opcode { name_format: String::from("AND indy"), address_mode: address_mode_INDY, operation: operation_AND, cycles: 0 },
            0x35 => Opcode { name_format: String::from("AND zpgx"), address_mode: address_mode_ZPX, operation: operation_AND, cycles: 0 },
            0x39 => Opcode { name_format: String::from("AND absy"), address_mode: address_mode_ABSY, operation: operation_AND, cycles: 0 },
            0x3D => Opcode { name_format: String::from("AND absx"), address_mode: address_mode_ABSX, operation: operation_AND, cycles: 0 },

            // ADC
            0x61 => Opcode { name_format: String::from("ADC xind"), address_mode: address_mode_XIND, operation: operation_ADC, cycles: 0 },
            0x65 => Opcode { name_format: String::from("ADC zpg"), address_mode: address_mode_ZPG, operation: operation_ADC, cycles: 0 },
            0x69 => Opcode { name_format: String::from("ADC #"), address_mode: address_mode_IMM, operation: operation_ADC, cycles: 0 },
            0x6D => Opcode { name_format: String::from("ADC abs"), address_mode: address_mode_ABS, operation: operation_ADC, cycles: 0 },
            0x71 => Opcode { name_format: String::from("ADC indy"), address_mode: address_mode_INDY, operation: operation_ADC, cycles: 0 },
            0x75 => Opcode { name_format: String::from("ADC zpgx"), address_mode: address_mode_ZPX, operation: operation_ADC, cycles: 0 },
            0x79 => Opcode { name_format: String::from("ADC absy"), address_mode: address_mode_ABSY, operation: operation_ADC, cycles: 0 },
            0x7D => Opcode { name_format: String::from("ADC absx"), address_mode: address_mode_ABSX, operation: operation_ADC, cycles: 0 },

            // STA
            0x81 => Opcode { name_format: String::from("STA xind"), address_mode: address_mode_XIND, operation: operation_STA, cycles: 0 },
            0x85 => Opcode { name_format: String::from("STA zpg"), address_mode: address_mode_ZPG, operation: operation_STA, cycles: 0 },
            0x8D => Opcode { name_format: String::from("STA abs"), address_mode: address_mode_ABS, operation: operation_STA, cycles: 0 },
            0x91 => Opcode { name_format: String::from("STA indy"), address_mode: address_mode_INDY, operation: operation_STA, cycles: 0 },
            0x95 => Opcode { name_format: String::from("STA zpgx"), address_mode: address_mode_ZPX, operation: operation_STA, cycles: 0 },
            0x96 => Opcode { name_format: String::from("STA zpgy"), address_mode: address_mode_ZPY, operation: operation_STA, cycles: 0 },
            0x99 => Opcode { name_format: String::from("STA absy"), address_mode: address_mode_ABSY, operation: operation_STA, cycles: 0 },
            0x9D => Opcode { name_format: String::from("STA absx"), address_mode: address_mode_ABSX, operation: operation_STA, cycles: 0 },

            // LDA
            0xA1 => Opcode { name_format: String::from("LDA xind"), address_mode: address_mode_XIND, operation: operation_LDA, cycles: 0 },
            0xA5 => Opcode { name_format: String::from("LDA zpg"), address_mode: address_mode_ZPG, operation: operation_LDA, cycles: 0 },
            0xA9 => Opcode { name_format: String::from("LDA #"), address_mode: address_mode_IMM, operation: operation_LDA, cycles: 0 },
            0xAD => Opcode { name_format: String::from("LDA abs"), address_mode: address_mode_ABS, operation: operation_LDA, cycles: 0 },
            0xB1 => Opcode { name_format: String::from("LDA indy"), address_mode: address_mode_INDY, operation: operation_LDA, cycles: 0 },
            0xB5 => Opcode { name_format: String::from("LDA zpgx"), address_mode: address_mode_ZPX, operation: operation_LDA, cycles: 0 },
            0xB9 => Opcode { name_format: String::from("LDA absy"), address_mode: address_mode_ABSY, operation: operation_LDA, cycles: 0 },
            0xBD => Opcode { name_format: String::from("LDA absx"), address_mode: address_mode_ABSX, operation: operation_LDA, cycles: 0 },

            // LDX
            0xA2 => Opcode { name_format: String::from("LDX #"), address_mode: address_mode_IMM, operation: operation_LDX, cycles: 0 },
            0xA6 => Opcode { name_format: String::from("LDX zpg"), address_mode: address_mode_ZPG, operation: operation_LDX, cycles: 0 },
            0xAE => Opcode { name_format: String::from("LDX abs"), address_mode: address_mode_ABS, operation: operation_LDX, cycles: 0 },
            0xB6 => Opcode { name_format: String::from("LDX zpgy"), address_mode: address_mode_ZPY, operation: operation_LDX, cycles: 0 },
            0xBE => Opcode { name_format: String::from("LDX absy"), address_mode: address_mode_ABSY, operation: operation_LDX, cycles: 0 },

            // LDY
            0xA0 => Opcode { name_format: String::from("LDY #"), address_mode: address_mode_IMM, operation: operation_LDY, cycles: 0 },
            0xA4 => Opcode { name_format: String::from("LDY zpg"), address_mode: address_mode_ZPG, operation: operation_LDY, cycles: 0 },
            0xAC => Opcode { name_format: String::from("LDY abs"), address_mode: address_mode_ABS, operation: operation_LDY, cycles: 0 },
            0xB4 => Opcode { name_format: String::from("LDY zpgx"), address_mode: address_mode_ZPX, operation: operation_LDY, cycles: 0 },
            0xBC => Opcode { name_format: String::from("LDY absx"), address_mode: address_mode_ABSX, operation: operation_LDY, cycles: 0 },

            // Branching
            0x10 => Opcode { name_format: String::from("BPL rel"), address_mode: address_mode_REL, operation: operation_BPL, cycles: 0 },
            0x30 => Opcode { name_format: String::from("BMI rel"), address_mode: address_mode_REL, operation: operation_BMI, cycles: 0 },
            0x50 => Opcode { name_format: String::from("BVC rel"), address_mode: address_mode_REL, operation: operation_BVC, cycles: 0 },
            0x70 => Opcode { name_format: String::from("BVS rel"), address_mode: address_mode_REL, operation: operation_BVS, cycles: 0 },
            0x90 => Opcode { name_format: String::from("BCC rel"), address_mode: address_mode_REL, operation: operation_BCC, cycles: 0 },
            0xB0 => Opcode { name_format: String::from("BCS rel"), address_mode: address_mode_REL, operation: operation_BCS, cycles: 0 },
            0xD0 => Opcode { name_format: String::from("BNE rel"), address_mode: address_mode_REL, operation: operation_BNE, cycles: 0 },
            0xF0 => Opcode { name_format: String::from("BEQ rel"), address_mode: address_mode_REL, operation: operation_BEQ, cycles: 0 },

            // Bit
            0x24 => Opcode { name_format: String::from("BIT zpg"), address_mode: address_mode_ZPG, operation: operation_BIT, cycles: 0 },
            0x2C => Opcode { name_format: String::from("BIT abs"), address_mode: address_mode_ABS, operation: operation_BIT, cycles: 0 },

            // Clear flags
            0x18 => Opcode { name_format: String::from("CLC impl"), address_mode: address_mode_ACC, operation: operation_CLC, cycles: 0 },
            0x58 => Opcode { name_format: String::from("CLI impl"), address_mode: address_mode_ACC, operation: operation_CLI, cycles: 0 },
            0xB8 => Opcode { name_format: String::from("CLV impl"), address_mode: address_mode_ACC, operation: operation_CLV, cycles: 0 },
            0xD8 => Opcode { name_format: String::from("CLD impl"), address_mode: address_mode_ACC, operation: operation_CLD, cycles: 0 },

            // Decrement Ops
            0xC6 => Opcode { name_format: String::from("DEC zpg"), address_mode: address_mode_ZPG, operation: operation_DEC, cycles: 0 },
            0xCE => Opcode { name_format: String::from("DEC ABS"), address_mode: address_mode_ABS, operation: operation_DEC, cycles: 0 },
            0xD6 => Opcode { name_format: String::from("DEC zpx"), address_mode: address_mode_ZPX, operation: operation_DEC, cycles: 0 },
            0xDE => Opcode { name_format: String::from("DEC absx"), address_mode: address_mode_ABSX, operation: operation_DEC, cycles: 0 },
            0xCA => Opcode { name_format: String::from("DEX imp"), address_mode: address_mode_ACC, operation: operation_DEX, cycles: 0 },
            0x88 => Opcode { name_format: String::from("DEY imp"), address_mode: address_mode_ACC, operation: operation_DEY, cycles: 0 },

            // XOR
            0x41 => Opcode { name_format: String::from("XOR xind"), address_mode: address_mode_XIND, operation: operation_EOR, cycles: 0 },
            0x45 => Opcode { name_format: String::from("XOR zpg"), address_mode: address_mode_ZPG, operation: operation_EOR, cycles: 0 },
            0x49 => Opcode { name_format: String::from("XOR #"), address_mode: address_mode_IMM, operation: operation_EOR, cycles: 0 },
            0x4D => Opcode { name_format: String::from("XOR abs"), address_mode: address_mode_ABS, operation: operation_EOR, cycles: 0 },
            0x51 => Opcode { name_format: String::from("XOR indy"), address_mode: address_mode_INDY, operation: operation_EOR, cycles: 0 },
            0x55 => Opcode { name_format: String::from("XOR zpx"), address_mode: address_mode_ZPX, operation: operation_EOR, cycles: 0 },
            0x59 => Opcode { name_format: String::from("XOR absy"), address_mode: address_mode_ABSY, operation: operation_EOR, cycles: 0 },
            0x5D => Opcode { name_format: String::from("XOR absx"), address_mode: address_mode_ABSX, operation: operation_EOR, cycles: 0 },

            // Increment
            0xE0 => Opcode { name_format: String::from("INC zpg"), address_mode: address_mode_ZPG, operation: operation_INC, cycles: 0 },
            0xEE => Opcode { name_format: String::from("INC abs"), address_mode: address_mode_ABS, operation: operation_INC, cycles: 0 },
            0xF6 => Opcode { name_format: String::from("INC zpgx"), address_mode: address_mode_ZPX, operation: operation_INC, cycles: 0 },
            0xFE => Opcode { name_format: String::from("INC absx"), address_mode: address_mode_ABSX, operation: operation_INC, cycles: 0 },
            0xC8 => Opcode { name_format: String::from("INY impl"), address_mode: address_mode_ACC, operation: operation_INY, cycles: 0 },
            0xE8 => Opcode { name_format: String::from("INX impl"), address_mode: address_mode_ACC, operation: operation_INX, cycles: 0 },

            
            0xEA => Opcode { name_format: String::from("NULL"), address_mode: address_mode_ACC, operation: operation_NOP, cycles: 0 },
            _ => Opcode { name_format: String::from("NULL"), address_mode: address_mode_ACC, operation: operation_NOP, cycles: 0 }, //TODO remove once we implement all instructions
        }
    }

    fn fetch(&mut self){
        // IMP address mode uses a register directly (aka no fetch)
        if self.opcode.address_mode as usize != address_mode_ACC as usize {
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
            let operation_func = self.opcode.operation;

            self.cycles += addr_mode_func(self);
            self.cycles += operation_func(self);
            self.cycles += self.opcode.cycles;

            println!("Opcode = {}", self.opcode.name_format);
            println!("Fetched = {:#x}", self.fetched);
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

    let lines = program.split("\n");

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
