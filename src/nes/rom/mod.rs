use std::fs;
use std::fs::File;
use std::io::Read;

pub struct Rom {
    //TODO add all possible PRG roms here
    pub prg1: [u8; 0x10000] //16KB
}

impl Rom {
    fn clear(&mut self){
        for address in 0..0x10000 {
            self.prg1[address as usize] = 0;
        }
    }

    pub fn load_bin_file(&mut self,filename: &String) {
        self.clear();

        let mut f = File::open(&filename).expect("no file found");
        let metadata = fs::metadata(&filename).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");
        for address in 0..0x10000 {
            self.prg1[address as usize] = buffer[address as usize];
        }
    }

    pub fn load_hex_dump(&mut self, offset : u16, program: &str) {
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

        let mut address: u16 = offset;
        for hex in hex_bytes {
            self.prg1[address as usize] = hex;
            address += 1;
        }
    }
}