use std::fs;
use std::fs::File;
use std::io::Read;
use std::str;

/*
F6 flags:

76543210
||||||||
|||||||+- Mirroring: 0: horizontal (vertical arrangement) (CIRAM A10 = PPU A11)
|||||||              1: vertical (horizontal arrangement) (CIRAM A10 = PPU A10)
||||||+-- 1: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
|||||+--- 1: 512-byte trainer at $7000-$71FF (stored before PRG data)
||||+---- 1: Ignore mirroring control or above mirroring bit; instead provide four-screen VRAM
++++----- Lower nybble of mapper number
*/

pub struct Rom {
    pub header : [u8; 16],
    pub trainer: [u8; 512],
    pub prg: [u8; 0x80000], // 512KB
    pub chr: [u8; 0x80000], // 512KB
    pub title: [u8; 128],
    pub bool

}

impl Rom {
    pub fn new() -> Rom {
        Self {
            header : [0; 16],
            trainer: [0; 512],
            prg: [0; 0x80000],
            chr: [0; 0x80000],
            title: [0; 128]
        }
    }

    pub fn load_rom(&mut self,filename: &String) {
        let mut file_handle = File::open(&filename).expect("no file found");
        let meta_data = fs::metadata(&filename).expect("unable to read metadata");
        let file_length = meta_data.len();
        file_handle.read_exact(&mut self.header).expect("buffer overflow");

        let nes_header = str::from_utf8(&self.header[0..4]).unwrap();
        if nes_header != "NES\x1a"{
            panic!("Nes identifier invalid: {}", nes_header);
        }

        let pgr_bank_count = self.header[4];
        let chr_bank_count = self.header[5];
        let f6 = self.header[6];
        let f7 = self.header[7];

        println!("PGR Bank Count = {}", pgr_bank_count);
        println!("CHR Bank Count = {}", chr_bank_count);

    }

    pub fn load_bin_file(&mut self,filename: &String) {

        let mut f = File::open(&filename).expect("no file found");
        let metadata = fs::metadata(&filename).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");
        for address in 0..0x10000 {
            self.prg[address as usize] = buffer[address as usize];
        }
    }

    pub fn load_hex_dump(&mut self, offset : u16, program: &str) {
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
            self.prg[address as usize] = hex;
            address += 1;
        }
    }
}