use std::fs;
use std::fs::File;
use std::io::Read;
use std::str;
use minifb::MouseButton::Middle;

const PRG_BANK_BANK_SIZE :u32 = (1 << 14); // 16384
const CHR_BANK_BANK_SIZE :u32 = (1 << 13); // 8192

const MAX_PRG_BANK_COUNT :u8 = 16; // 1MB
const MAX_CHR_BANK_COUNT :u8 = 16; // 512KB

const MAX_PRG_BANK_SIZE:u32 = PRG_BANK_BANK_SIZE * (MAX_PRG_BANK_COUNT as u32);
const MAX_CHR_BANK_SIZE:u32 = CHR_BANK_BANK_SIZE * (MAX_CHR_BANK_COUNT as u32);

#[derive(Debug)]
pub enum Mirroring {
    HORIZONTAL,
    VERTICAL,
    FOUR_SCREEN
}

pub struct Rom {
    pub header : [u8; 16],
    pub trainer: [u8; 512],
    pub prg: [u8; MAX_PRG_BANK_SIZE as usize],
    pub chr: [u8; MAX_CHR_BANK_SIZE as usize],
    pub title: [u8; 128],
    pub title_size: u8,

    pub prg_bank_count : u8,
    pub chr_bank_count : u8,
    pub mirroring : Mirroring,
    pub has_battery_ram: bool,
    pub has_trainer: bool,
    pub mapper_number: u8,
}

impl Rom {
    pub fn new() -> Rom {
        Self {
            header : [0; 16],
            trainer: [0; 512],
            prg: [0; MAX_PRG_BANK_SIZE as usize],
            chr: [0; MAX_CHR_BANK_SIZE as usize],
            title: [0; 128],
            title_size: 0,
            prg_bank_count: 0,
            chr_bank_count: 0,
            has_trainer: false,
            has_battery_ram: false,
            mapper_number: 0,
            mirroring: Mirroring::VERTICAL
        }
    }

    pub fn load_rom(&mut self,filename: &String) {
        let mut file_handle = File::open(&filename).expect("no file found");
        let meta_data = fs::metadata(&filename).expect("unable to read metadata");
        let file_length = meta_data.len();

        println!("Loading rom: {}", filename);

        file_handle.read_exact(&mut self.header).expect("header buffer overflow");
        self.parse_header();

        if self.has_trainer {
            println!("Loading trainer");
            file_handle.read_exact(&mut self.trainer).expect("trainer buffer overflow");
        }

        // Load PRG
        {
            let prg_total_bank_size = (PRG_BANK_BANK_SIZE * (self.prg_bank_count as u32)) as u64;
            let mut buf = vec![0u8; prg_total_bank_size as usize];
            file_handle.read_exact(&mut buf).expect("buffer overflow");
            for i in 0..prg_total_bank_size {
                self.prg[i as usize] = buf[i as usize];
            }
            println!("Loaded PRG ROM, size was {:#x}", prg_total_bank_size);
        }

        // Load CHR
        {
            let chr_total_bank_size = (CHR_BANK_BANK_SIZE * (self.chr_bank_count as u32)) as u64;
            let mut buf = vec![0u8; chr_total_bank_size as usize];
            file_handle.read_exact(&mut buf).expect("buffer overflow");
            for i in 0..chr_total_bank_size {
                self.chr[i as usize] = buf[i as usize];
            }
            println!("Loaded CHR ROM, size was {:#x}", chr_total_bank_size);
        }

        // Load Title
        {
            let mut buf = vec![0u8; 128 as usize];
            self.title_size = file_handle.read_to_end(& mut buf).expect("Didn't read enough") as u8;
            for i in 0..self.title_size {
                self.title[i as usize] = buf[i as usize];
            }
            println!("Title bytes loaded: {}", self.title_size);
        }

        // Verify we are EOF
        {
            let mut buf = vec![0u8; 1024 as usize];
            let mut buf_size = file_handle.read_to_end(& mut buf).expect("Didn't read enough");
            if buf_size != 0 {
                panic!("Found extra '{}' bytes after the title, expected EOF!!", buf_size);
            }
        }
    }

    fn parse_header(&mut self){
        // Parse NES marker
        let nes_header = str::from_utf8(&self.header[0..4]).unwrap();
        if nes_header != "NES\x1a"{
            panic!("Nes identifier invalid: {}", nes_header);
        }
        println!("NES header validated");

        // Get Bank Counts
        self.prg_bank_count = self.header[4];
        self.chr_bank_count = self.header[5];

        if self.prg_bank_count > MAX_PRG_BANK_COUNT {
            panic!("PRG bank count exceeds limit: {} > {}", self.prg_bank_count, MAX_PRG_BANK_COUNT);
        }
        if self.chr_bank_count > MAX_CHR_BANK_COUNT {
            panic!("CHR bank count exceeds limit: {} > {}", self.chr_bank_count, MAX_CHR_BANK_COUNT);
        }

        println!("PGR Bank Count: {}", self.prg_bank_count);
        println!("CHR Bank Count: {}", self.chr_bank_count);

        let f6_flags = self.header[6];
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

        // Mirroring
        if f6_flags & (1 << 3) != 0{
            //Bit 3: Ignore mirroring control or above mirroring bit; instead provide four-screen VRAM
            self.mirroring = Mirroring::FOUR_SCREEN;
        }
        else{
            // Bit 0: Mirroring
            if f6_flags & (1 << 0) == 0{
                self.mirroring = Mirroring::HORIZONTAL;
            }
            else{
                self.mirroring = Mirroring::VERTICAL;
            }
        }
        println!("Mirroring: {:?}", self.mirroring);

        // Bit 1: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
        self.has_battery_ram = f6_flags & (1 << 1) != 0;
        println!("Battery Ram: {:?}", self.has_battery_ram);

        // Bit 2: 512-byte trainer at $7000-$71FF (stored before PRG data)
        self.has_trainer = f6_flags & (1 << 2) != 0;
        println!("Trainer: {:?}", self.has_trainer);

        self.mapper_number = (f6_flags >> 4);
        println!("Mapper number: {:?}", self.mapper_number);
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