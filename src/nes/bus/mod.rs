use rand::Rng;

pub struct Bus {
    pub ram: [u8; 65536]
}

impl Bus {
    pub fn read_ram(&self, location : u16) -> u8 {
        if location == 0x00FE { //TODO remove
            return rand::thread_rng().gen_range(0..256) as u8;
        }
        return self.ram[location as usize];
    }

    pub fn write_ram(&mut self, location : u16, value : u8){
        self.ram[location as usize ] = value;
    }

    pub fn reset_ram(&mut self){
        for addr in 0..65535 {
            self.write_ram(addr, 0x00);
        }
    }

    pub fn print_ram(&self, start : u16, length : u16){
        println!("\nMemory: start=0x{:04x} length=0x{:04x}", start, length);
        let mut counter: u32 = 0;
        for addr in start..start+length+1 {
            if counter % 16 == 0 {
                print!("{:04x}: ", addr);
            }
            print!("{:02x} ",self.read_ram(addr));

            if counter % 16 == 15 {
                println!();
            }

            counter+=1;
        }
    }

    pub fn loadProgram(& mut self, start_address : u16, program: &str){
        self.reset_ram(); // resets the ram

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
            self.write_ram(address, hex);
            address += 1;
        }
    }
}
