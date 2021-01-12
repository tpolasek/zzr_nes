pub struct Rom {
    //TODO add all possible PRG roms here
    pub prg1: [u8; 0x4000] //16KB
}

impl Rom {
    fn clear(&mut self){
        for address in 0..0x4000 {
            self.prg1[address as usize] = 0;
        }
    }

    pub fn load_hex_dump(&mut self, program: &str) {
        self.clear();

        let lines = program.split("\n");

        let mut hexchars = String::with_capacity(60);

        for line in lines {
            let char_vec: Vec<char> = line.chars().collect();
            let mut foundColon: bool = false;
            for c in char_vec {
                if c == ' ' {
                    continue;
                }
                if c == '#' {
                    break;
                }

                if c == '\n' {
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

        let mut address: u16 = 0;
        for hex in hex_bytes {
            self.prg1[address as usize] = hex;
            address += 1;
        }
    }
}