use crate::nes::rom::{Mirroring, Rom};
use egui::{Color32, ColorImage};

#[allow(dead_code)]
pub static PALETTE_LOOKUP: [u32; 64] = [
    0x545454, 0x001E74, 0x081090, 0x300088, 0x440064, 0x5C0030, 0x540400, 0x3C1800, 0x202A00,
    0x083A00, 0x004000, 0x003C00, 0x00323C, 0x000000, 0x000000, 0x000000, 0x989698, 0x084CC4,
    0x3032EC, 0x5C1EE4, 0x8814B0, 0xA01464, 0x982220, 0x783C00, 0x545A00, 0x287200, 0x087C00,
    0x007628, 0x006678, 0x000000, 0x000000, 0x000000, 0xECEEEC, 0x4C9AEC, 0x787CEC, 0xB062EC,
    0xE454EC, 0xEC58B4, 0xEC6A64, 0xD48820, 0xA0AA00, 0x74C400, 0x4CD020, 0x38CC6C, 0x38B4CC,
    0x3C3C3C, 0x000000, 0x000000, 0xECEEEC, 0xA8CCEC, 0xBCBCEC, 0xD4B2EC, 0xECAEEC, 0xECAED4,
    0xECB4B0, 0xE4C490, 0xCCD278, 0xB4DE78, 0xA8E290, 0x98E2B4, 0xA0D6E4, 0xA0A2A0, 0x000000,
    0x000000,
];

const PPU_STATUS_VBLANK_BIT: u8 = 1 << 7;
const PPU_CTRL_NMI_TRIGGER_BIT: u8 = 1 << 7;

pub struct Ppu {
    // TODO once we are done debugging remove any public methods
    reg_ctrl: u8,
    reg_mask: u8,
    reg_status: u8,
    address_latch: bool,
    address: u16,
    data: u8,
    data_buffer: u8,
    pub pixel: u16,
    pub scanline: u16,
    pub gbuffer: ColorImage,
    nmi_triggered: bool,

    // Loopy registers (PPU internal addressing)
    pub v: u16,  // Current VRAM address (15 bits)
    pub t: u16,  // Temporary VRAM address (15 bits)
    pub x: u8,   // Fine X scroll (3 bits)
    pub w: bool, // First/second write toggle

    // Background rendering shift registers
    bg_pattern_shift_low: u16,  // Pattern table low bits
    bg_pattern_shift_high: u16, // Pattern table high bits
    bg_palette_shift_low: u16,  // Palette attribute low bits
    bg_palette_shift_high: u16, // Palette attribute high bits

    // Background latches (for next tile)
    bg_next_tile_id: u8,
    bg_next_tile_attr: u8,
    bg_next_tile_lsb: u8, // Pattern low byte
    bg_next_tile_msb: u8, // Pattern high byte

    // Sprite rendering (8 sprites per scanline)
    sprite_count: usize,
    sprite_pattern_shift_low: [u8; 8],
    sprite_pattern_shift_high: [u8; 8],
    sprite_positions: [u8; 8],
    sprite_priorities: [u8; 8],
    sprite_indices: [u8; 8],

    // Sprite 0 hit detection
    sprite_zero_being_rendered: bool,
    sprite_zero_hit_possible: bool,

    // OAM address
    oam_addr: u8,

    // Mirroring mode
    mirroring: Mirroring,

    // https://wiki.nesdev.com/w/index.php/PPU_memory_map
    // pattern table usually maps to rom CHR
    vram_bank_1: [u8; 0x400], //  Nametable Ram only 2k (room for 2 nametables mirrored, some roms have onboard memory for 4 tables)
    vram_bank_2: [u8; 0x400],
    pub palette_ram: [u8; 0x20],
    pub oam_ram: [u8; 0x100], // 256 bytes for 64 sprites (4 bytes each)
}

impl Ppu {
    pub fn new() -> Ppu {
        Self {
            reg_ctrl: 0,
            reg_mask: 0,
            reg_status: 0,
            address_latch: false,
            address: 0,
            data: 0,
            data_buffer: 0,
            pixel: 0,
            scanline: 0,
            gbuffer: ColorImage::new([256usize, 240usize], Color32::BLACK),
            nmi_triggered: false,
            v: 0,
            t: 0,
            x: 0,
            w: false,
            bg_pattern_shift_low: 0,
            bg_pattern_shift_high: 0,
            bg_palette_shift_low: 0,
            bg_palette_shift_high: 0,
            bg_next_tile_id: 0,
            bg_next_tile_attr: 0,
            bg_next_tile_lsb: 0,
            bg_next_tile_msb: 0,
            sprite_count: 0,
            sprite_pattern_shift_low: [0; 8],
            sprite_pattern_shift_high: [0; 8],
            sprite_positions: [0; 8],
            sprite_priorities: [0; 8],
            sprite_indices: [0; 8],
            sprite_zero_being_rendered: false,
            sprite_zero_hit_possible: false,
            oam_addr: 0,
            mirroring: Mirroring::HORIZONTAL,
            vram_bank_1: [0; 0x400],
            vram_bank_2: [0; 0x400],
            palette_ram: [0; 0x20],
            oam_ram: [0; 0x100],
        }
    }

    #[allow(dead_code)]
    pub fn is_vblank(&self) -> bool {
        return self.reg_status & PPU_STATUS_VBLANK_BIT != 0;
    }

    pub fn set_mirroring(&mut self, mirroring: Mirroring) {
        self.mirroring = mirroring;
    }

    pub fn cpuReadImmutable(&self, _rom: &Rom, register_num: u8) -> u8 {
        return match register_num {
            0 => self.read_PPUCTRL_Immutable(),
            1 => self.read_PPUMASK_Immutable(),
            2 => self.read_PPUSTATUS_Immutable(),
            3 => self.read_OAMADDR(),
            4 => self.read_OAMDATA(),
            5 => self.read_PPUSCROLL(),
            6 => self.read_PPUADDR(),
            7 => self.read_PPUDATA_Immutable(),
            _ => {
                panic!("We should never get here in the PPU addr={}", register_num);
            }
        };
    }

    pub fn cpuRead(&mut self, rom: &Rom, register_num: u8) -> u8 {
        return match register_num {
            0 => self.read_PPUCTRL(),
            1 => self.read_PPUMASK(),
            2 => self.read_PPUSTATUS(),
            3 => self.read_OAMADDR(),
            4 => self.read_OAMDATA(),
            5 => self.read_PPUSCROLL(),
            6 => self.read_PPUADDR(),
            7 => self.read_PPUDATA(rom),
            _ => {
                panic!("We should never get here in the PPU addr={}", register_num);
            }
        };
    }

    pub fn cpuWrite(&mut self, _rom: &mut Rom, register_num: u8, value: u8) {
        match register_num {
            0 => self.write_PPUCTRL(value),
            1 => self.write_PPUMASK(value),
            2 => self.write_PPUSTATUS(value),
            3 => self.write_OAMADDR(value),
            4 => self.write_OAMDATA(value),
            5 => self.write_PPUSCROLL(value),
            6 => self.write_PPUADDR(value),
            7 => self.write_PPUDATA(value),
            _ => {
                panic!("We should never get here in the PPU addr={}", register_num);
            }
        }
    }

    /// Public wrapper for PPU memory reads (for debugger/visualization)
    pub fn read_ppu_memory(&self, rom: &Rom, address: u16) -> u8 {
        self.ppuRead(rom, address)
    }

    /// Start Read Register
    fn ppuRead(&self, rom: &Rom, address: u16) -> u8 {
        return match address {
            0x0000..=0x1FFF => {
                return rom.read_chr(address);
            }
            0x2000..=0x3EFF => {
                let tmp_addr = address & 0xFFF;

                match rom.mirroring {
                    Mirroring::HORIZONTAL => {
                        if tmp_addr <= 0x7FF {
                            return self.vram_bank_1[(tmp_addr & 0x3FF) as usize];
                        } else {
                            //0x800 - 0xFFF
                            return self.vram_bank_2[(tmp_addr & 0x3FF) as usize];
                        }
                    }
                    Mirroring::VERTICAL => {
                        if tmp_addr <= 0x3FF {
                            return self.vram_bank_1[(tmp_addr & 0x3FF) as usize];
                        } else if tmp_addr <= 0x7FF {
                            return self.vram_bank_2[(tmp_addr & 0x3FF) as usize];
                        } else if tmp_addr <= 0xBFF {
                            return self.vram_bank_1[(tmp_addr & 0x3FF) as usize];
                        } else {
                            //0x800 - 0xFFF
                            return self.vram_bank_2[(tmp_addr & 0x3FF) as usize];
                        }
                    }
                    Mirroring::FourScreen => {
                        // TODO implement, also need SINGLE SCREEN
                        return 0;
                    }
                }
            }
            0x3F00..=0x3FFF => {
                let mut tmp_addr = address & 0x1F;

                // Palette mirroring
                tmp_addr = match address {
                    0x10 | 0x14 | 0x18 | 0x1C => tmp_addr & 0xF,
                    _ => tmp_addr,
                };

                if self.reg_mask & 0x1 != 0 {
                    // greyscale
                    return self.palette_ram[tmp_addr as usize] & 0x30;
                } else {
                    return self.palette_ram[tmp_addr as usize] & 0x3F;
                }
            }
            _ => 0, // TODO validate this is correct, do we only go to 0x3FFF?
        };
    }

    fn ppuWrite(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                // CHR ROM - usually read-only for Mapper 0
            }
            0x2000..=0x3EFF => {
                let tmp_addr = address & 0xFFF;

                match self.mirroring {
                    Mirroring::HORIZONTAL => {
                        if tmp_addr <= 0x7FF {
                            self.vram_bank_1[(tmp_addr & 0x3FF) as usize] = value;
                        } else {
                            self.vram_bank_2[(tmp_addr & 0x3FF) as usize] = value;
                        }
                    }
                    Mirroring::VERTICAL => {
                        if tmp_addr <= 0x3FF {
                            self.vram_bank_1[(tmp_addr & 0x3FF) as usize] = value;
                        } else if tmp_addr <= 0x7FF {
                            self.vram_bank_2[(tmp_addr & 0x3FF) as usize] = value;
                        } else if tmp_addr <= 0xBFF {
                            self.vram_bank_1[(tmp_addr & 0x3FF) as usize] = value;
                        } else {
                            self.vram_bank_2[(tmp_addr & 0x3FF) as usize] = value;
                        }
                    }
                    Mirroring::FourScreen => {
                        // TODO: Need 4KB VRAM
                    }
                }
            }
            0x3F00..=0x3FFF => {
                let mut tmp_addr = address & 0x1F;

                // Palette mirroring
                if tmp_addr == 0x10 || tmp_addr == 0x14 || tmp_addr == 0x18 || tmp_addr == 0x1C {
                    tmp_addr &= 0x0F;
                }

                self.palette_ram[tmp_addr as usize] = value;
            }
            _ => {}
        }
    }

    fn read_PPUCTRL(&self) -> u8 {
        return 0; // not readable
    }

    fn read_PPUCTRL_Immutable(&self) -> u8 {
        return self.reg_ctrl; // for debug only
    }

    fn read_PPUMASK(&self) -> u8 {
        return 0; // not readable
    }

    fn read_PPUMASK_Immutable(&self) -> u8 {
        return self.reg_mask; // for debug only
    }

    fn read_PPUSTATUS(&mut self) -> u8 {
        self.w = false; // Reset write latch

        // last 5 bits are the last data bits written -- whack
        let output = (self.reg_status & 0xE0) | (self.data_buffer & 0x1F);

        self.reg_status &= !PPU_STATUS_VBLANK_BIT;
        return output;
    }

    fn read_PPUSTATUS_Immutable(&self) -> u8 {
        return self.reg_status; // for debug only
    }

    fn read_OAMADDR(&self) -> u8 {
        return 0; // Not readable
    }

    fn read_OAMDATA(&self) -> u8 {
        self.oam_ram[self.oam_addr as usize]
    }

    fn read_PPUSCROLL(&self) -> u8 {
        return 0; // Not Readable
    }

    fn read_PPUADDR(&self) -> u8 {
        return 0; // Not Readable
    }

    fn read_PPUDATA(&mut self, rom: &Rom) -> u8 {
        self.data = self.data_buffer;
        self.data_buffer = self.ppuRead(rom, self.v);

        // Palette reads are not buffered
        if self.v >= 0x3F00 {
            self.data = self.data_buffer;
        }

        self.v = self.v.wrapping_add(self.get_vram_increment());
        return self.data;
    }

    fn read_PPUDATA_Immutable(&self) -> u8 {
        return 0; //debug only
    }

    /// End Read Registers

    // PPUCTRL bit extractors
    fn get_nametable_base(&self) -> u16 {
        0x2000 + (((self.reg_ctrl & 0x03) as u16) << 10)
    }

    fn get_vram_increment(&self) -> u16 {
        if self.reg_ctrl & 0x04 != 0 { 32 } else { 1 }
    }

    fn get_sprite_pattern_table(&self) -> u16 {
        if self.reg_ctrl & 0x08 != 0 {
            0x1000
        } else {
            0x0000
        }
    }

    fn get_bg_pattern_table(&self) -> u16 {
        if self.reg_ctrl & 0x10 != 0 {
            0x1000
        } else {
            0x0000
        }
    }

    fn get_sprite_size(&self) -> u8 {
        if self.reg_ctrl & 0x20 != 0 { 16 } else { 8 }
    }

    fn should_trigger_nmi(&self) -> bool {
        self.reg_ctrl & 0x80 != 0
    }

    // PPUMASK bit extractors
    fn is_rendering_enabled(&self) -> bool {
        (self.reg_mask & 0x08) != 0 || (self.reg_mask & 0x10) != 0
    }

    fn show_background(&self) -> bool {
        (self.reg_mask & 0x08) != 0
    }

    fn show_sprites(&self) -> bool {
        (self.reg_mask & 0x10) != 0
    }

    fn show_background_left(&self) -> bool {
        (self.reg_mask & 0x02) != 0
    }

    fn show_sprites_left(&self) -> bool {
        (self.reg_mask & 0x04) != 0
    }

    fn is_greyscale(&self) -> bool {
        (self.reg_mask & 0x01) != 0
    }

    // VRAM address manipulation (Loopy register updates)
    fn increment_scroll_x(&mut self) {
        if !self.is_rendering_enabled() {
            return;
        }

        if (self.v & 0x001F) == 31 {
            // Coarse X = 0
            self.v &= !0x001F;
            // Switch horizontal nametable
            self.v ^= 0x0400;
        } else {
            // Increment coarse X
            self.v += 1;
        }
    }

    fn increment_scroll_y(&mut self) {
        if !self.is_rendering_enabled() {
            return;
        }

        if (self.v & 0x7000) != 0x7000 {
            // Increment fine Y
            self.v += 0x1000;
        } else {
            // Fine Y = 0
            self.v &= !0x7000;
            let mut y = (self.v & 0x03E0) >> 5;

            if y == 29 {
                y = 0;
                // Switch vertical nametable
                self.v ^= 0x0800;
            } else if y == 31 {
                y = 0;
            } else {
                y += 1;
            }

            self.v = (self.v & !0x03E0) | (y << 5);
        }
    }

    fn transfer_address_x(&mut self) {
        if !self.is_rendering_enabled() {
            return;
        }
        // Copy coarse X and horizontal nametable bit from t to v
        self.v = (self.v & 0xFBE0) | (self.t & 0x041F);
    }

    fn transfer_address_y(&mut self) {
        if !self.is_rendering_enabled() {
            return;
        }
        // Copy fine Y, coarse Y, and vertical nametable bit from t to v
        self.v = (self.v & 0x841F) | (self.t & 0x7BE0);
    }

    fn reload_shift_registers(&mut self) {
        // Load the next tile's pattern data into the shift registers
        self.bg_pattern_shift_low =
            (self.bg_pattern_shift_low & 0xFF00) | (self.bg_next_tile_lsb as u16);
        self.bg_pattern_shift_high =
            (self.bg_pattern_shift_high & 0xFF00) | (self.bg_next_tile_msb as u16);

        // Load palette bits (expand 2 bits to 8 bits based on attribute)
        let attr_bits = self.bg_next_tile_attr & 0x03;
        let palette_low = if attr_bits & 0x01 != 0 { 0xFF } else { 0x00 };
        let palette_high = if attr_bits & 0x02 != 0 { 0xFF } else { 0x00 };

        self.bg_palette_shift_low = (self.bg_palette_shift_low & 0xFF00) | palette_low;
        self.bg_palette_shift_high = (self.bg_palette_shift_high & 0xFF00) | palette_high;
    }

    // Background tile fetching methods
    fn fetch_nametable_byte(&self, rom: &Rom) -> u8 {
        let addr = 0x2000 | (self.v & 0x0FFF);
        self.ppuRead(rom, addr)
    }

    fn fetch_attribute_byte(&self, rom: &Rom) -> u8 {
        let addr = 0x23C0 | (self.v & 0x0C00) | ((self.v >> 4) & 0x38) | ((self.v >> 2) & 0x07);
        let attribute = self.ppuRead(rom, addr);

        // Determine which 2x2 tile quadrant (4 quadrants per attribute byte)
        let coarse_x = self.v & 0x1F;
        let coarse_y = (self.v >> 5) & 0x1F;

        let shift = ((coarse_y & 0x02) << 1) | (coarse_x & 0x02);

        (attribute >> shift) & 0x03
    }

    fn fetch_pattern_low(&self, rom: &Rom, tile_id: u8, fine_y: u8) -> u8 {
        let base = self.get_bg_pattern_table();
        let addr = base + ((tile_id as u16) << 4) + (fine_y as u16);
        self.ppuRead(rom, addr)
    }

    fn fetch_pattern_high(&self, rom: &Rom, tile_id: u8, fine_y: u8) -> u8 {
        let base = self.get_bg_pattern_table();
        let addr = base + ((tile_id as u16) << 4) + (fine_y as u16) + 8;
        self.ppuRead(rom, addr)
    }

    // Pixel generation methods
    fn get_background_pixel(&self) -> (u8, u8) {
        if !self.show_background() {
            return (0, 0);
        }

        // Hide left 8 pixels if disabled
        if self.pixel < 8 && !self.show_background_left() {
            return (0, 0);
        }

        // Mux based on fine X scroll
        let bit_mux = 0x8000 >> self.x;

        let p0 = ((self.bg_pattern_shift_low & bit_mux) != 0) as u8;
        let p1 = ((self.bg_pattern_shift_high & bit_mux) != 0) as u8;
        let pixel = (p1 << 1) | p0;

        let pal0 = ((self.bg_palette_shift_low & bit_mux) != 0) as u8;
        let pal1 = ((self.bg_palette_shift_high & bit_mux) != 0) as u8;
        let palette = (pal1 << 1) | pal0;

        (pixel, palette)
    }

    fn get_color_from_palette(&self, palette: u8, pixel: u8) -> u32 {
        let addr = if pixel == 0 {
            0x3F00 // Universal background color
        } else if palette < 4 {
            // Background palette
            0x3F00 + ((palette as u16) << 2) + (pixel as u16)
        } else {
            // Sprite palette
            0x3F10 + (((palette - 4) as u16) << 2) + (pixel as u16)
        };

        let mut palette_index = self.palette_ram[(addr & 0x1F) as usize];

        // Handle palette mirroring
        let mirrored_addr = addr & 0x1F;
        if mirrored_addr == 0x10
            || mirrored_addr == 0x14
            || mirrored_addr == 0x18
            || mirrored_addr == 0x1C
        {
            palette_index = self.palette_ram[(mirrored_addr & 0x0F) as usize];
        }

        if self.is_greyscale() {
            palette_index &= 0x30;
        }

        PALETTE_LOOKUP[(palette_index & 0x3F) as usize]
    }

    // Sprite evaluation (scanline N for scanline N+1)
    fn evaluate_sprites(&mut self, rom: &Rom) {
        self.sprite_count = 0;
        self.sprite_zero_being_rendered = false;

        for i in 0..64 {
            let oam_offset = i * 4;
            let sprite_y = self.oam_ram[oam_offset] as u16;
            let tile_id = self.oam_ram[oam_offset + 1];
            let attributes = self.oam_ram[oam_offset + 2];
            let sprite_x = self.oam_ram[oam_offset + 3];

            // Check if sprite is on next scanline
            let next_scanline = if self.scanline == 261 {
                0
            } else {
                self.scanline + 1
            };
            let sprite_height = if self.get_sprite_size() == 16 { 16 } else { 8 };
            let diff = next_scanline.wrapping_sub(sprite_y);

            if diff < sprite_height {
                if self.sprite_count < 8 {
                    // Sprite 0 detection
                    if i == 0 {
                        self.sprite_zero_being_rendered = true;
                    }

                    // Fetch sprite pattern
                    let mut row = diff;

                    // Vertical flip
                    if attributes & 0x80 != 0 {
                        row = sprite_height - 1 - row;
                    }

                    let pattern_addr = if sprite_height == 8 {
                        self.get_sprite_pattern_table() + ((tile_id as u16) << 4) + row
                    } else {
                        // 8x16 sprites
                        let table = ((tile_id & 0x01) as u16) << 12;
                        let tile = ((tile_id & 0xFE) as u16) << 4;
                        table + tile + (if row >= 8 { row + 8 } else { row })
                    };

                    let mut pattern_low = self.ppuRead(rom, pattern_addr);
                    let mut pattern_high = self.ppuRead(rom, pattern_addr + 8);

                    // Horizontal flip
                    if attributes & 0x40 != 0 {
                        pattern_low = pattern_low.reverse_bits();
                        pattern_high = pattern_high.reverse_bits();
                    }

                    self.sprite_pattern_shift_low[self.sprite_count] = pattern_low;
                    self.sprite_pattern_shift_high[self.sprite_count] = pattern_high;
                    self.sprite_positions[self.sprite_count] = sprite_x;
                    self.sprite_priorities[self.sprite_count] = attributes;
                    self.sprite_indices[self.sprite_count] = i as u8;

                    self.sprite_count += 1;
                } else {
                    // Sprite overflow
                    self.reg_status |= 0x20;
                }
            }
        }
    }

    // Get sprite pixel for current position
    fn get_sprite_pixel(&self) -> (u8, u8, u8, bool) {
        if !self.show_sprites() {
            return (0, 0, 0, false);
        }

        // Hide left 8 pixels if disabled
        if self.pixel < 8 && !self.show_sprites_left() {
            return (0, 0, 0, false);
        }

        for i in 0..self.sprite_count {
            let sprite_x = self.sprite_positions[i];

            // Check if this pixel is within the sprite
            if self.pixel >= sprite_x as u16 && self.pixel < (sprite_x as u16 + 8) {
                let offset = (self.pixel - sprite_x as u16) as u8;
                let bit_mux = 0x80 >> offset;

                let p0 = ((self.sprite_pattern_shift_low[i] & bit_mux) != 0) as u8;
                let p1 = ((self.sprite_pattern_shift_high[i] & bit_mux) != 0) as u8;
                let pixel = (p1 << 1) | p0;

                if pixel != 0 {
                    let palette = (self.sprite_priorities[i] & 0x03) + 4;
                    let priority = (self.sprite_priorities[i] & 0x20) == 0;
                    let sprite_zero =
                        self.sprite_zero_being_rendered && self.sprite_indices[i] == 0;

                    return (pixel, palette, priority as u8, sprite_zero);
                }
            }
        }

        (0, 0, 0, false)
    }

    fn write_PPUCTRL(&mut self, value: u8) {
        self.reg_ctrl = value;
        // Update t with nametable select (bits 0-1 go to bits 10-11 of t)
        self.t = (self.t & 0xF3FF) | (((value as u16) & 0x03) << 10);
        self.data_buffer = value;
    }
    fn write_PPUMASK(&mut self, value: u8) {
        self.reg_mask = value;
    }
    fn write_PPUSTATUS(&mut self, _value: u8) {
        //TODO
    }
    fn write_OAMADDR(&mut self, value: u8) {
        self.oam_addr = value;
    }
    fn write_OAMDATA(&mut self, value: u8) {
        self.oam_ram[self.oam_addr as usize] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }
    fn write_PPUSCROLL(&mut self, value: u8) {
        if !self.w {
            // First write: X scroll
            self.t = (self.t & 0xFFE0) | ((value as u16) >> 3); // Coarse X
            self.x = value & 0x07; // Fine X
            self.w = true;
        } else {
            // Second write: Y scroll
            self.t = (self.t & 0x8FFF) | (((value as u16) & 0x07) << 12); // Fine Y
            self.t = (self.t & 0xFC1F) | (((value as u16) & 0xF8) << 2); // Coarse Y
            self.w = false;
        }
        self.data_buffer = value;
    }
    fn write_PPUADDR(&mut self, value: u8) {
        if !self.w {
            // First write: high byte
            self.t = (self.t & 0x00FF) | (((value as u16) & 0x3F) << 8);
            self.w = true;
        } else {
            // Second write: low byte
            self.t = (self.t & 0xFF00) | (value as u16);
            self.v = self.t; // Copy t to v
            self.w = false;
        }
        self.data_buffer = value;
    }

    fn write_PPUDATA(&mut self, value: u8) {
        self.ppuWrite(self.v, value);
        self.v = self.v.wrapping_add(self.get_vram_increment());
        self.data_buffer = value;
    }

    #[allow(dead_code)]
    fn get_color(&self, rom: &Rom, palette: u8, sprite_color_index: u8) -> u32 {
        // 4 colors per palette, sprite_color_index indexes into the palette
        return PALETTE_LOOKUP[(self.ppuRead(
            rom,
            0x3F00 + ((palette as u16) << 2) + (sprite_color_index as u16),
        ) & 0x3F) as usize];
    }

    pub fn tick(&mut self, rom: &Rom) {
        /*
        if self.scanline == 0 && self.pixel == 1 {
            println!(
                "Scanline 0 start: v={:04X}, t={:04X}, x={}, w={},
        ctrl={:02X}, mask={:02X}",
                self.v, self.t, self.x, self.w, self.reg_ctrl, self.reg_mask
            );
            println!(
                "Shift regs: pattern_low={:04X}, pattern_high={:04X}",
                self.bg_pattern_shift_low, self.bg_pattern_shift_high
            );
        }
        */

        let rendering_enabled = self.is_rendering_enabled();

        // Shift registers and tile fetching (happens on visible AND pre-render scanlines)
        if (self.scanline < 240 || self.scanline == 261) && rendering_enabled {
            // Shift registers during visible pixels (1-256) and prefetch cycles (321-336)
            // IMPORTANT: This must happen BEFORE rendering the pixel
            if (self.pixel >= 1 && self.pixel <= 256) || (self.pixel >= 321 && self.pixel <= 336) {
                self.bg_pattern_shift_low <<= 1;
                self.bg_pattern_shift_high <<= 1;
                self.bg_palette_shift_low <<= 1;
                self.bg_palette_shift_high <<= 1;
            }
        }

        // Visible scanlines (0-239)
        if self.scanline < 240 {
            if self.pixel > 0 && self.pixel <= 256 {
                // RENDER PIXEL (after shifting)
                if rendering_enabled {
                    // Get background pixel
                    let (bg_pixel, bg_palette) = self.get_background_pixel();

                    // Get sprite pixel
                    let (sprite_pixel, sprite_palette, sprite_priority, is_sprite_zero) =
                        self.get_sprite_pixel();

                    // Sprite 0 hit detection
                    if is_sprite_zero && bg_pixel > 0 && sprite_pixel > 0 {
                        // Sprite 0 hit conditions:
                        // - Both bg and sprite pixels are opaque (non-zero)
                        // - Not in leftmost 8 pixels (unless enabled by PPUMASK)
                        // - Not at pixel 255 (hardware quirk)

                        let leftmost_ok = (self.show_background_left() && self.show_sprites_left())
                            || self.pixel >= 9;

                        if leftmost_ok && self.pixel < 255 {
                            self.reg_status |= 0x40; // Set sprite 0 hit flag
                        }
                    }

                    // Determine final pixel using priority rules
                    let (final_pixel, final_palette) = if bg_pixel == 0 && sprite_pixel == 0 {
                        (0, 0)
                    } else if bg_pixel == 0 && sprite_pixel > 0 {
                        (sprite_pixel, sprite_palette)
                    } else if bg_pixel > 0 && sprite_pixel == 0 {
                        (bg_pixel, bg_palette)
                    } else {
                        // Both have pixel, check priority
                        if sprite_priority != 0 {
                            (sprite_pixel, sprite_palette)
                        } else {
                            (bg_pixel, bg_palette)
                        }
                    };

                    let color = self.get_color_from_palette(final_palette, final_pixel);

                    let offset = ((self.pixel - 1) + self.scanline * 256) as usize;
                    let rgb = [
                        ((color >> 16) & 0xFF) as u8,
                        ((color >> 8) & 0xFF) as u8,
                        (color & 0xFF) as u8,
                    ];
                    self.gbuffer.pixels[offset] = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                }
            }
        }

        // Tile fetching (happens on visible AND pre-render scanlines)
        if (self.scanline < 240 || self.scanline == 261) && rendering_enabled {
            // Background tile fetching (8 cycles per tile)
            // Fetches happen during visible area (1-256) and prefetch (321-336)
            if (self.pixel >= 1 && self.pixel <= 256) || (self.pixel >= 321 && self.pixel <= 336) {
                match (self.pixel - 1) % 8 {
                    0 => {
                        self.reload_shift_registers();
                        self.bg_next_tile_id = self.fetch_nametable_byte(rom);
                    }
                    2 => {
                        self.bg_next_tile_attr = self.fetch_attribute_byte(rom);
                    }
                    4 => {
                        let fine_y = ((self.v >> 12) & 0x07) as u8;
                        self.bg_next_tile_lsb =
                            self.fetch_pattern_low(rom, self.bg_next_tile_id, fine_y);
                    }
                    6 => {
                        let fine_y = ((self.v >> 12) & 0x07) as u8;
                        self.bg_next_tile_msb =
                            self.fetch_pattern_high(rom, self.bg_next_tile_id, fine_y);
                    }
                    7 => {
                        self.increment_scroll_x();
                    }
                    _ => {}
                }
            }

            // End of visible scanline
            if self.pixel == 256 {
                self.increment_scroll_y();
            }
            if self.pixel == 257 {
                self.transfer_address_x();
                // Sprite evaluation for next scanline (only on visible scanlines)
                if self.scanline < 240 {
                    self.evaluate_sprites(rom);
                }
            }
        }

        // Pre-render scanline (261)
        if self.scanline == 261 {
            if self.pixel == 1 {
                self.reg_status &= !PPU_STATUS_VBLANK_BIT;
                self.reg_status &= !0x40; // Clear sprite 0 hit
                self.reg_status &= !0x20; // Clear sprite overflow
            }

            if rendering_enabled && self.pixel >= 280 && self.pixel <= 304 {
                self.transfer_address_y();
            }
        }

        // VBlank
        if self.scanline == 241 && self.pixel == 1 {
            if self.should_trigger_nmi() {
                self.nmi_triggered = true;
            }
            self.reg_status |= PPU_STATUS_VBLANK_BIT;
        }

        // Advance counters
        self.pixel += 1;
        if self.pixel > 340 {
            self.pixel = 0;
            self.scanline += 1;
            if self.scanline > 261 {
                self.scanline = 0;
            }
        }
    }

    pub fn get_and_reset_nmi_triggered(&mut self) -> bool {
        if self.nmi_triggered {
            self.nmi_triggered = false;
            return true;
        }
        return false;
    }
}
