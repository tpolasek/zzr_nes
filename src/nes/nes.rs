use eframe::{App, Frame, egui};
use egui::{Color32, FontFamily, FontId, RichText, Style, TextStyle, TextureHandle, Visuals};
use std::{thread, time};
/*
This file contains the GUI implementation and it is also where we execute the emulator.
*/

use crate::nes::cpu::Cpu;
use crate::nes::cpu::Opcode;
use crate::nes::debugger::Debugger;

struct GUIInstruction {
    addr: u16,
    text: String,
    symbol: String,
    breakpoint_pc: bool,
    breakpoint_memory: bool,
}

pub struct Nes {
    cpu: Cpu,
    debugger: Debugger,
    running: bool,
    step_next_count: u32,
    step_out_mode: bool,
    memory_dump: String,
    disasm: Vec<GUIInstruction>,
    stack_data: Vec<String>,
    previous_pc: u16,
    image: Option<TextureHandle>,
    ran_instruction: bool,
    show_breakpoint_window: bool,
    breakpoint_addr_input: String,
    show_ppu_debug_window: bool,
    selected_pattern_palette: u8,
    pattern_table_0_texture: Option<TextureHandle>,
    pattern_table_1_texture: Option<TextureHandle>,
    nametable_texture: Option<TextureHandle>,
}

impl Nes {
    fn default(filename: &String, debug_file: Option<&String>) -> Self {
        let mut cpu = Cpu::new();
        let debugger: Debugger = Debugger::new(debug_file);
        let step_next_count: u32 = 0;
        cpu.bus.rom.load_rom(filename);
        // Set PPU mirroring from ROM
        cpu.bus.ppu.set_mirroring(cpu.bus.rom.mirroring);
        cpu.reset();

        let disasm: Vec<GUIInstruction> = Vec::new();

        let stack_data: Vec<String> = Vec::new();

        Self {
            cpu,
            debugger,
            running: false,
            step_next_count,
            step_out_mode: false,
            memory_dump: "".to_string(),
            disasm,
            stack_data,
            previous_pc: 0,
            image: None,
            ran_instruction: false,
            show_breakpoint_window: false,
            breakpoint_addr_input: String::new(),
            show_ppu_debug_window: false,
            selected_pattern_palette: 0,
            pattern_table_0_texture: None,
            pattern_table_1_texture: None,
            nametable_texture: None,
        }
    }

    pub fn new(ctx: &egui::Context, filename: &String, debug_file: Option<&String>) -> Self {
        let app: Nes = Nes::default(filename, debug_file);

        ctx.set_visuals(Visuals::light());

        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "pixel".to_owned(),
            egui::FontData::from_static(include_bytes!("../ProggyClean.ttf")),
        );

        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "pixel".to_owned());

        ctx.set_fonts(fonts);

        let mut style: Style = Style::default();

        style
            .text_styles
            .insert(TextStyle::Body, FontId::new(14.0, FontFamily::Monospace));
        style.visuals.panel_fill = Color32::from_rgb(0, 0, 0); // Classic gray
        style.visuals.override_text_color = Some(Color32::from_rgb(0, 255, 0));
        style
            .text_styles
            .get_mut(&egui::TextStyle::Body)
            .unwrap()
            .size = 16.0;
        style
            .text_styles
            .get_mut(&egui::TextStyle::Button)
            .unwrap()
            .size = 16.0;
        style
            .text_styles
            .get_mut(&egui::TextStyle::Monospace)
            .unwrap()
            .size = 16.0;
        style
            .text_styles
            .get_mut(&egui::TextStyle::Heading)
            .unwrap()
            .size = 16.0;
        style
            .text_styles
            .get_mut(&egui::TextStyle::Small)
            .unwrap()
            .size = 16.0;
        ctx.set_style(style);

        app
    }

    fn populate_stack_data(&mut self) {
        // Update stack data
        let mut stack_data: Vec<String> = Vec::new();
        let stack_pointer = self.cpu.reg_sp;

        // Read stack data from memory (stack is at 0x0100-0x01FF)
        // Stack grows downward, so we display from current SP position to bottom of stack
        for stack_addr in (0x0100 + stack_pointer as u16 + 1)..=0x01FF {
            let value = self.cpu.bus.read_ram_immutable_debug(stack_addr);

            // Format: "SP+offset: address value"
            let offset = (stack_addr - 0x0100) as u8;

            stack_data.push(format!("{:02X}: {:02X}", offset, value));
        }

        self.stack_data = stack_data;
    }

    fn generate_memory_dump(&self) -> String {
        let memory: Vec<u8> = (0..0xFFFF)
            .map(|i: u16| self.cpu.bus.read_ram_immutable_debug(i))
            .collect();

        let cols: usize = 16;
        let rows: usize = (memory.len() + cols - 1) / cols;
        let mut output = String::new();

        for r in 0..rows {
            let base = r * cols;
            output.push_str(&format!("{:04X}: ", base));

            // Hex bytes
            for c in 0..cols {
                let idx = base + c;
                if idx < memory.len() {
                    output.push_str(&format!("{:02X} ", memory[idx]));
                } else {
                    output.push_str("   ");
                }
            }

            output.push_str(" | ");

            // ASCII representation
            for c in 0..cols {
                let idx = base + c;
                if idx < memory.len() {
                    let b = memory[idx];
                    if b.is_ascii_graphic() {
                        output.push(b as char);
                    } else {
                        output.push('.');
                    }
                } else {
                    output.push(' ');
                }
            }

            output.push('\n');
        }

        output
    }
    fn generate_opcode_diassembly(&mut self) {
        /*
        let mut addresses_to_disam: Vec<u16> = Vec::new();
        if self.previous_pc != 0 && self.previous_pc != self.cpu.pc {
            addresses_to_disam.push(self.previous_pc);
        }
        */

        let mut disasm: Vec<GUIInstruction> = Vec::new();
        // TODO get instructions before.
        // in the future
        let mut pc_addr_scan_ahead = self.cpu.pc;
        for _index in 0..40 {
            let current_opcode: &Opcode = self.cpu.get_optcode(pc_addr_scan_ahead);
            let instruction_str = current_opcode.get_instruction_decoded(
                &self.cpu,
                &self.debugger,
                pc_addr_scan_ahead,
            );

            let memory_accessed =
                current_opcode.get_memory_addr_accessed(&self.cpu, pc_addr_scan_ahead);

            let symbol = self
                .debugger
                .get_symbol_at_memory_access(pc_addr_scan_ahead);

            disasm.push(GUIInstruction {
                addr: pc_addr_scan_ahead,
                text: instruction_str,
                symbol,
                breakpoint_pc: self.debugger.hit_breakpoint_pc(pc_addr_scan_ahead),
                breakpoint_memory: (memory_accessed.is_some()
                    && self
                        .debugger
                        .hit_breakpoint_memory_access(memory_accessed.unwrap())),
            });
            pc_addr_scan_ahead += current_opcode.get_opcode_byte_size();
        }
        self.disasm = disasm;
    }

    fn ui_action_step(&mut self) {
        self.step_next_count = 1;
    }
    fn ui_action_big_step(&mut self) {
        self.step_next_count = 1000;
    }
    fn ui_action_step_out(&mut self) {
        self.step_out_mode = true;
    }

    fn emulator_stop(&mut self) {
        self.step_out_mode = false;
        self.step_next_count = 0;
        self.running = false;
    }

    fn nes_color_to_rgb(nes_color: u8) -> Color32 {
        let color_u32 = crate::nes::ppu::PALETTE_LOOKUP[(nes_color & 0x3F) as usize];
        Color32::from_rgb(
            ((color_u32 >> 16) & 0xFF) as u8,
            ((color_u32 >> 8) & 0xFF) as u8,
            (color_u32 & 0xFF) as u8,
        )
    }

    fn decode_tile_row(
        rom: &crate::nes::rom::Rom,
        tile_id: u8,
        row: u8,
        pattern_table_base: u16,
    ) -> [u8; 8] {
        let tile_addr = pattern_table_base + ((tile_id as u16) << 4) + (row as u16);
        let low_byte = rom.read_chr(tile_addr);
        let high_byte = rom.read_chr(tile_addr + 8);

        let mut pixels = [0u8; 8];
        for bit in 0..8 {
            let mask = 0x80 >> bit;
            let low_bit = if low_byte & mask != 0 { 1 } else { 0 };
            let high_bit = if high_byte & mask != 0 { 2 } else { 0 };
            pixels[bit] = low_bit | high_bit;
        }
        pixels
    }

    fn get_palette_color(
        ppu: &crate::nes::ppu::Ppu,
        palette_index: u8,
        pixel_value: u8,
    ) -> Color32 {
        if pixel_value == 0 {
            let nes_color = ppu.palette_ram[0];
            return Self::nes_color_to_rgb(nes_color);
        }

        let palette_addr = if palette_index < 4 {
            (palette_index * 4 + pixel_value) as usize
        } else {
            (0x10 + (palette_index - 4) * 4 + pixel_value) as usize
        };

        let nes_color = ppu.palette_ram[palette_addr & 0x1F];
        Self::nes_color_to_rgb(nes_color)
    }

    fn render_pattern_table(&self, table_index: u8, palette_index: u8) -> egui::ColorImage {
        let pattern_table_base = if table_index == 0 { 0x0000 } else { 0x1000 };
        let size = [128, 128];
        let mut image = egui::ColorImage::new(size, Color32::BLACK);

        for tile_y in 0..16 {
            for tile_x in 0..16 {
                let tile_id = (tile_y * 16 + tile_x) as u8;

                for row in 0..8 {
                    let pixels =
                        Self::decode_tile_row(&self.cpu.bus.rom, tile_id, row, pattern_table_base);

                    for col in 0..8 {
                        let pixel_value = pixels[col];
                        let color =
                            Self::get_palette_color(&self.cpu.bus.ppu, palette_index, pixel_value);

                        let x = tile_x * 8 + col;
                        let y = tile_y * 8 + row as usize;
                        let offset = y * 128 + x;

                        image.pixels[offset] = color;
                    }
                }
            }
        }

        image
    }

    fn render_nametable(&self) -> egui::ColorImage {
        let size = [256, 240];
        let mut image = egui::ColorImage::new(size, Color32::BLACK);

        let ctrl = self.cpu.bus.ppu.cpuReadImmutable(&self.cpu.bus.rom, 0);
        let nametable_base = 0x2000u16 + (((ctrl & 0x03) as u16) << 10);
        let bg_pattern_table = if ctrl & 0x10 != 0 { 0x1000 } else { 0x0000 };

        for tile_y in 0..30 {
            for tile_x in 0..32 {
                let nametable_addr = nametable_base + (tile_y * 32 + tile_x);
                let tile_id = self
                    .cpu
                    .bus
                    .ppu
                    .read_ppu_memory(&self.cpu.bus.rom, nametable_addr);

                // Get attribute palette
                let attr_addr = nametable_base + 0x3C0 + (tile_y / 4) * 8 + (tile_x / 4);
                let attr_byte = self
                    .cpu
                    .bus
                    .ppu
                    .read_ppu_memory(&self.cpu.bus.rom, attr_addr);
                let shift = ((tile_y % 4) / 2) * 4 + ((tile_x % 4) / 2) * 2;
                let palette_index = (attr_byte >> shift) & 0x03;

                for row in 0..8 {
                    let pixels =
                        Self::decode_tile_row(&self.cpu.bus.rom, tile_id, row, bg_pattern_table);

                    for col in 0..8 {
                        let pixel_value = pixels[col];
                        let color =
                            Self::get_palette_color(&self.cpu.bus.ppu, palette_index, pixel_value);

                        let x = tile_x as usize * 8 + col;
                        let y = tile_y as usize * 8 + row as usize;
                        let offset = y * 256 + x;

                        image.pixels[offset] = color;
                    }
                }
            }
        }

        image
    }

    fn render_ppu_debug_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("PPU Debug")
            .collapsible(true)
            .resizable(true)
            .max_width(700.0)
            .default_height(800.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Pattern Tables Section
                    ui.heading("Pattern Tables");
                    ui.separator();

                    // Palette selector
                    ui.horizontal(|ui| {
                        ui.heading("Palette:");
                        for i in 0..8 {
                            if ui
                                .selectable_label(
                                    self.selected_pattern_palette == i,
                                    format!("{}", i),
                                )
                                .clicked()
                            {
                                self.selected_pattern_palette = i;
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("Pattern Table 0");
                            if let Some(tex) = &self.pattern_table_0_texture {
                                ui.add(
                                    egui::Image::from_texture(tex)
                                        .fit_to_exact_size(egui::vec2(256.0, 256.0)),
                                );
                            }
                        });

                        ui.vertical(|ui| {
                            ui.heading("Pattern Table 1");
                            if let Some(tex) = &self.pattern_table_1_texture {
                                ui.add(
                                    egui::Image::from_texture(tex)
                                        .fit_to_exact_size(egui::vec2(256.0, 256.0)),
                                );
                            }
                        });
                    });

                    ui.add_space(10.0);

                    // OAM Sprites
                    ui.heading("OAM Sprites");
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.heading(RichText::new("#   Y  X  Tile Pal Pri H V").monospace());
                            });

                            for i in 0..64 {
                                let oam_offset = i * 4;
                                let y = self.cpu.bus.ppu.oam_ram[oam_offset];
                                let tile = self.cpu.bus.ppu.oam_ram[oam_offset + 1];
                                let attr = self.cpu.bus.ppu.oam_ram[oam_offset + 2];
                                let x = self.cpu.bus.ppu.oam_ram[oam_offset + 3];

                                let palette = (attr & 0x03) + 4;
                                let priority = if attr & 0x20 != 0 { "BG" } else { "FG" };
                                let flip_h = if attr & 0x40 != 0 { "Y" } else { "N" };
                                let flip_v = if attr & 0x80 != 0 { "Y" } else { "N" };

                                ui.heading(
                                    RichText::new(format!(
                                        "{:02} {:02X} {:02X} {:02X}  {}  {}  {} {}",
                                        i, y, x, tile, palette, priority, flip_h, flip_v
                                    ))
                                    .monospace(),
                                );
                            }
                        });

                    ui.add_space(10.0);

                    // PPU Registers and Internal State
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("PPU Registers");
                            ui.separator();

                            let ctrl = self.cpu.bus.ppu.cpuReadImmutable(&self.cpu.bus.rom, 0);
                            let mask = self.cpu.bus.ppu.cpuReadImmutable(&self.cpu.bus.rom, 1);
                            let status = self.cpu.bus.ppu.cpuReadImmutable(&self.cpu.bus.rom, 2);

                            ui.heading(
                                RichText::new(format!("PPUCTRL: ${:02X}", ctrl)).monospace(),
                            );
                            ui.heading(
                                RichText::new(format!(
                                    "  NMI: {}",
                                    if ctrl & 0x80 != 0 { "ON" } else { "OFF" }
                                ))
                                .monospace(),
                            );
                            ui.heading(
                                RichText::new(format!(
                                    "  Sprite size: {}",
                                    if ctrl & 0x20 != 0 { "8x16" } else { "8x8" }
                                ))
                                .monospace(),
                            );
                            ui.heading(
                                RichText::new(format!(
                                    "  BG table: ${:04X}",
                                    if ctrl & 0x10 != 0 { 0x1000 } else { 0x0000 }
                                ))
                                .monospace(),
                            );
                            ui.heading(
                                RichText::new(format!(
                                    "  Sprite table: ${:04X}",
                                    if ctrl & 0x08 != 0 { 0x1000 } else { 0x0000 }
                                ))
                                .monospace(),
                            );

                            ui.add_space(4.0);
                            ui.heading(
                                RichText::new(format!("PPUMASK: ${:02X}", mask)).monospace(),
                            );
                            ui.heading(
                                RichText::new(format!(
                                    "  Show BG: {}",
                                    if mask & 0x08 != 0 { "ON" } else { "OFF" }
                                ))
                                .monospace(),
                            );
                            ui.heading(
                                RichText::new(format!(
                                    "  Show Sprites: {}",
                                    if mask & 0x10 != 0 { "ON" } else { "OFF" }
                                ))
                                .monospace(),
                            );

                            ui.add_space(4.0);
                            ui.heading(
                                RichText::new(format!("PPUSTATUS: ${:02X}", status)).monospace(),
                            );
                            ui.heading(
                                RichText::new(format!(
                                    "  VBlank: {}",
                                    if status & 0x80 != 0 { "YES" } else { "NO" }
                                ))
                                .monospace(),
                            );
                            ui.heading(
                                RichText::new(format!(
                                    "  Sprite 0 hit: {}",
                                    if status & 0x40 != 0 { "YES" } else { "NO" }
                                ))
                                .monospace(),
                            );
                            ui.heading(
                                RichText::new(format!(
                                    "  Sprite overflow: {}",
                                    if status & 0x20 != 0 { "YES" } else { "NO" }
                                ))
                                .monospace(),
                            );
                            ui.separator();

                            ui.heading("Internal State");
                            ui.separator();
                            ui.heading(
                                RichText::new(format!("Scanline: {}", self.cpu.bus.ppu.scanline))
                                    .monospace(),
                            );
                            ui.heading(
                                RichText::new(format!("Pixel: {}", self.cpu.bus.ppu.pixel))
                                    .monospace(),
                            );
                            ui.heading(
                                RichText::new(format!("v: ${:04X}", self.cpu.bus.ppu.v))
                                    .monospace(),
                            );
                            ui.heading(
                                RichText::new(format!("t: ${:04X}", self.cpu.bus.ppu.t))
                                    .monospace(),
                            );
                            ui.heading(
                                RichText::new(format!("x: ${:02X}", self.cpu.bus.ppu.x))
                                    .monospace(),
                            );
                            ui.heading(
                                RichText::new(format!(
                                    "w: {}",
                                    if self.cpu.bus.ppu.w { "2nd" } else { "1st" }
                                ))
                                .monospace(),
                            );
                        });
                    });

                    ui.add_space(10.0);

                    // Palette RAM
                    ui.heading("Palette RAM");
                    ui.separator();
                    ui.heading("Background Palettes:");
                    for pal in 0..4 {
                        ui.horizontal(|ui| {
                            ui.heading(format!("{}:", pal));
                            for col in 0..4 {
                                let addr = pal * 4 + col;
                                let nes_color = self.cpu.bus.ppu.palette_ram[addr];
                                let rgb_color = Self::nes_color_to_rgb(nes_color);

                                let (rect, _response) = ui.allocate_exact_size(
                                    egui::vec2(16.0, 16.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(rect, 0.0, rgb_color);
                                ui.heading(
                                    RichText::new(format!("${:02X}", nes_color))
                                        .monospace()
                                        .small(),
                                );
                            }
                        });
                    }

                    ui.add_space(4.0);
                    ui.heading("Sprite Palettes:");
                    for pal in 0..4 {
                        ui.horizontal(|ui| {
                            ui.heading(format!("{}:", pal + 4));
                            for col in 0..4 {
                                let addr = 0x10 + pal * 4 + col;
                                let nes_color = self.cpu.bus.ppu.palette_ram[addr & 0x1F];
                                let rgb_color = Self::nes_color_to_rgb(nes_color);

                                let (rect, _response) = ui.allocate_exact_size(
                                    egui::vec2(16.0, 16.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(rect, 0.0, rgb_color);
                                ui.heading(
                                    RichText::new(format!("${:02X}", nes_color))
                                        .monospace()
                                        .small(),
                                );
                            }
                        });
                    }

                    ui.add_space(10.0);

                    // Nametable Preview
                    ui.heading("Nametable Preview");
                    ui.separator();
                    if let Some(tex) = &self.nametable_texture {
                        ui.add(
                            egui::Image::from_texture(tex)
                                .fit_to_exact_size(egui::vec2(512.0, 480.0)),
                        );
                    }
                });
            });
    }

    fn emulator_execution_loop(&mut self) {
        self.ran_instruction = false;
        let start_time = std::time::Instant::now();
        let frame_duration = std::time::Duration::from_millis(16); // 16.6ms for 60Hz, using 16ms for safety

        while self.step_out_mode || self.step_next_count > 0 || self.running {
            // Check if we've exceeded frame time
            if start_time.elapsed() >= frame_duration {
                break;
            }

            self.step_next_count -= 1;

            self.previous_pc = self.cpu.pc;

            // Start Core emulation here
            self.cpu.execute_cpu_ppu();
            while !self.cpu.ready_to_execute_next_instruction() {
                self.cpu.execute_cpu_ppu();
            }
            self.ran_instruction = true;

            // End Core emulation here

            // Debugger control flow section
            if self.debugger.hit_breakpoint_pc(self.cpu.pc) {
                // hit breakpoint, stop.
                self.emulator_stop();
                break;
            }
            let optcode = self.cpu.get_optcode(self.cpu.pc);
            let memory_accessed_u16 = optcode.get_memory_addr_accessed(&self.cpu, self.cpu.pc);
            if memory_accessed_u16.is_some()
                && self.debugger.hit_breakpoint_memory_access(
                    memory_accessed_u16.expect("We check is_some() how is this possible?"),
                )
            {
                // hit breakpoint, stop.
                self.emulator_stop();
                break;
            }

            if self.step_out_mode && self.cpu.get_optcode(self.previous_pc).is_rts() {
                // hit RTS in step out mode, stop.
                self.emulator_stop();
                break;
            }
        }
    }
}

impl App for Nes {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        let start_time = std::time::Instant::now();
        self.emulator_execution_loop();
        let elapsed_time: time::Duration = start_time.elapsed();
        if elapsed_time < std::time::Duration::from_millis(16) {
            // Refresh the UI at 60fps
            thread::sleep(std::time::Duration::from_millis(16) - elapsed_time);
        }

        ctx.request_repaint();

        if self.ran_instruction && !self.running {
            // SUPER SUPER EXPENSIVE, this scans the entire memory map
            self.memory_dump = self.generate_memory_dump();
        }
        self.generate_opcode_diassembly();
        self.populate_stack_data();

        // Start Keyboard Hooks
        if ctx.input(|i| i.key_pressed(egui::Key::S)) {
            self.ui_action_step()
        }
        // End Keyboard Hooks

        // Render image
        self.image = Some(ctx.load_texture(
            "ppu_preview",
            self.cpu.bus.ppu.gbuffer.clone(), // TODO look into replacing this clone with arc?
            egui::TextureOptions::default(),
        ));

        // Regenerate PPU debug images if window is open and instruction ran
        if self.show_ppu_debug_window && self.ran_instruction {
            let pt0_image = self.render_pattern_table(0, self.selected_pattern_palette);
            self.pattern_table_0_texture =
                Some(ctx.load_texture("pattern_table_0", pt0_image, egui::TextureOptions::NEAREST));

            let pt1_image = self.render_pattern_table(1, self.selected_pattern_palette);
            self.pattern_table_1_texture =
                Some(ctx.load_texture("pattern_table_1", pt1_image, egui::TextureOptions::NEAREST));

            let nt_image = self.render_nametable();
            self.nametable_texture =
                Some(ctx.load_texture("nametable", nt_image, egui::TextureOptions::NEAREST));
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui: &mut egui::Ui| {
            ui.horizontal(|ui: &mut egui::Ui| {
                if ui.button("Step").clicked() {
                    self.ui_action_step()
                }
                if ui.button("Big Step").clicked() {
                    self.ui_action_big_step()
                }
                if ui.button("Step Out").clicked() {
                    self.ui_action_step_out()
                }
                if ui.button("Run").clicked() {
                    self.running = true;
                }
                if ui.button("Pause").clicked() {
                    self.emulator_stop();
                }
                if ui.button("Reset").clicked() {
                    self.cpu.reset();
                    self.previous_pc = 0;
                    self.step_next_count = 0;
                    self.step_out_mode = false;
                }
                if ui.button("PPU").clicked() {
                    self.show_ppu_debug_window = !self.show_ppu_debug_window;
                }
                if ui.button("Breakpoint").clicked() {
                    self.show_breakpoint_window = !self.show_breakpoint_window;
                }
            });
        });

        // Side panel first - spans full vertical space between top and bottom of window
        egui::SidePanel::right("sidebar")
            .default_width(250.0)
            .show(ctx, |ui| {
                // Game Rendering Screen is here:
                if let Some(tex) = &self.image {
                    ui.add(egui::Image::from_texture(tex));
                }
                ui.add_space(8.0);

                ui.heading("Registers");
                ui.separator();
                ui.heading(RichText::new(format!(" A={:02X}", self.cpu.reg_a)));
                ui.heading(RichText::new(format!(" X={:02X}", self.cpu.reg_x)));
                ui.heading(RichText::new(format!(" Y={:02X}", self.cpu.reg_y)));
                ui.heading(RichText::new(format!("SP={:02X}", self.cpu.reg_sp)));
                ui.heading(RichText::new(format!("PC={:04x}", self.cpu.pc)));
                ui.heading(RichText::new(format!(
                    "{}",
                    self.cpu.flag.get_formatted_str()
                )));

                ui.add_space(8.0);
                ui.heading("CPU Info");
                ui.separator();
                ui.heading(format!("Cycle: {}", self.cpu.tick_count as usize));
                ui.add_space(8.0);
                ui.heading("CPU Stack ($0100-$01FF)");
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    for stack_item in &self.stack_data {
                        ui.heading(stack_item);
                    }
                });
            });

        // Central panel - takes remaining space after sidebar
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            ui.push_id(1, |ui| {
                // Use a vertical layout to split the central panel
                // Disassembly section with its own scroll area
                ui.heading("Disassembly");
                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(400.0) // Set a max height for disassembly section
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        for (_, ins) in self.disasm.iter_mut().enumerate() {
                            let is_pc = ins.addr == self.cpu.pc;
                            let text: RichText = if ins.breakpoint_pc {
                                if is_pc {
                                    RichText::new(format!(">{}\t\t{}", ins.text, ins.symbol))
                                        .background_color(Color32::LIGHT_BLUE)
                                        .color(Color32::BLACK)
                                } else {
                                    RichText::new(format!(" {}\t\t{}", ins.text, ins.symbol))
                                        .background_color(Color32::LIGHT_RED)
                                        .color(Color32::BLACK)
                                }
                            } else if ins.breakpoint_memory {
                                if is_pc {
                                    RichText::new(format!(">{}\t\t{}", ins.text, ins.symbol))
                                        .background_color(Color32::LIGHT_BLUE)
                                        .color(Color32::BLACK)
                                } else {
                                    RichText::new(format!(" {}\t\t{}", ins.text, ins.symbol))
                                        .background_color(Color32::from_rgb(255, 165, 0)) // Orange
                                        .color(Color32::BLACK)
                                }
                            } else if is_pc {
                                RichText::new(format!(">{}\t\t{}", ins.text, ins.symbol))
                                    .background_color(Color32::DEBUG_COLOR)
                                    .color(Color32::BLACK)
                            } else {
                                RichText::new(format!(" {}\t\t{}", ins.text, ins.symbol))
                            };
                            let response = ui.selectable_label(ins.breakpoint_pc, text);
                            if response.clicked() {
                                self.debugger.toggle_breakpoint_pc(ins.addr, None);
                            }
                            if is_pc && self.ran_instruction {
                                response.scroll_to_me(Some(egui::Align::Center));
                            }
                        }
                    });
            });
            ui.add_space(14.0);
            ui.push_id(2, |ui| {
                // Memory Hex Dump section with its own scroll area
                ui.heading("Memory Hex Dump");
                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(300.0) // Set a max height for memory section
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.add(
                            egui::TextEdit::multiline(&mut self.memory_dump.as_str())
                                .desired_width(ui.available_width())
                                .font(egui::TextStyle::Small),
                        );
                    });
            });
        });

        // Breakpoint Selection Window
        if self.show_breakpoint_window {
            egui::Window::new("Breakpoint Selection")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Address (hex):");
                        ui.text_edit_singleline(&mut self.breakpoint_addr_input);
                    });
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.button("Add PC Breakpoint").clicked() {
                            // Parse the hex address and add PC breakpoint
                            if let Ok(addr) = u16::from_str_radix(
                                &self.breakpoint_addr_input.trim_start_matches("0x"),
                                16,
                            ) {
                                self.debugger.toggle_breakpoint_pc(addr, None);
                                self.breakpoint_addr_input.clear();
                            }
                        }

                        if ui.button("Add Memory Breakpoint").clicked() {
                            // Parse the hex address and add memory breakpoint
                            if let Ok(addr) = u16::from_str_radix(
                                &self.breakpoint_addr_input.trim_start_matches("0x"),
                                16,
                            ) {
                                self.debugger.toggle_breakpoint_memory_access(addr, None);
                                self.breakpoint_addr_input.clear();
                            }
                        }
                    });

                    ui.add_space(8.0);

                    if ui.button("Close").clicked() {
                        self.show_breakpoint_window = false;
                    }
                });
        }

        // PPU Debug Window
        if self.show_ppu_debug_window {
            self.render_ppu_debug_window(ctx);
        }
    }
}

/*
Support Key Presses
Key::W => self.cpu.bus.controller.pressed(Button::UP),
Key::A => self.cpu.bus.controller.pressed(Button::LEFT),
Key::S => self.cpu.bus.controller.pressed(Button::DOWN),
Key::D => self.cpu.bus.controller.pressed(Button::RIGHT),
Key::K => self.cpu.bus.controller.pressed(Button::A),
Key::L => self.cpu.bus.controller.pressed(Button::B),
*/
