//NV-BDIZC
//00110000
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

    let mut sum : u16 = cpu.reg_a as u16 + cpu.fetched as u16;
    if cpu.flag.get_flag_c(){
        sum += 1;
    }
    cpu.flag.set_flag_c(sum > 0xff);

    cpu.reg_a = (sum & 0xff) as u8;

    cpu.flag.set_flag_z(cpu.reg_a == 0);
    cpu.flag.set_flag_n(cpu.reg_a & 0b10000000 != 0);

    //TODO set v flag

    println!("ADC = {:#x}", cpu.reg_a);
    return 1;
}

fn operation_LDA(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    cpu.reg_a = cpu.fetched;
    cpu.flag.set_flag_z(cpu.reg_a == 0);
    cpu.flag.set_flag_n(cpu.reg_a & 0b10000000 != 0);
    return 1;
}

fn operation_STA(cpu : & mut Cpu) -> u8 {
    cpu.bus.write_ram(cpu.abs_addr, cpu.reg_a);
    return 1;
}


fn operation_BNE(cpu : & mut Cpu) -> u8 {
    println!("Relative Addr Offset = {}", cpu.relative_addr_offset);

    let mut cycle_cost : u8 = 1;

    if cpu.flag.get_flag_z() == false {
        let updated_pc = (cpu.pc as i32 + cpu.relative_addr_offset as i32) as u16;

        if updated_pc & 0xFF00 != cpu.pc & 0xFF00 {
            cycle_cost += 1;
        }
        cpu.pc = updated_pc;
    }
    return cycle_cost;
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

    fn map_to_opcode(&self, value : u8) -> Opcode{
        println!("Optcode byte: {:x}", value);

        return match value {
            0x69 => Opcode { name_format: String::from("ADC #"), address_mode: address_mode_IMM, operation: operation_ADC },
            0x65 => Opcode { name_format: String::from("ADC zpg"), address_mode: address_mode_ZPG, operation: operation_ADC },
            0x6D => Opcode { name_format: String::from("ADC abs"), address_mode: address_mode_ABS, operation: operation_ADC },
            0xD0 => Opcode { name_format: String::from("BNE rel"), address_mode: address_mode_REL, operation: operation_BNE },
            _ => Opcode { name_format: String::from("NULL"), address_mode: address_mode_NOP, operation: operation_NOP },
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
}


fn loadProgram(bus : & mut Bus, start_address : u16, program: &str){
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


fn main() {

    let mut bus = Bus { ram:  [0; 65536]};

    loadProgram(&mut bus, 0x0600, "0000: 65 FF 69 05 6D 00 FF 69 FF # Some comment" );

    //bus.write_ram(0x0607, 0xD0); // BNE -6?
    //bus.write_ram(0x0608, 0xF8);

    //bus.write_ram(0x00FF, 0x02);
    //bus.write_ram(0xFF00, 0xa);


    let flag = Flag {flag_n: 0, flag_v: 0, flag_b: 0, flag_d: 0, flag_i: 0, flag_z: 0, flag_c: 0};
    let mut cpu = Cpu { bus: bus, pc: 0x0600, cycles: 0, reg_a: 0, reg_x: 0, reg_y: 0, reg_sp: 0, flag: flag, tick_count : 0, fetched : 0, opcode : Opcode {name_format: String::from("NULL"), address_mode: address_mode_IMP, operation: operation_ADC }, abs_addr: 0, relative_addr_offset: 0};



    loop { cpu.tick();  break; } //TODO not done

    for x in 0..40 {
        cpu.tick();
    }



    println!("{:#010b}", cpu.reg_a);
}
