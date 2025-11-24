use eframe::{App, Frame, egui};
use egui::{Color32, FontFamily, FontId, RichText, Style, TextStyle, TextureHandle, Visuals};
use std::{thread, time};
mod bus;
mod controller;
mod cpu;
mod cpu_flag;
mod debugger;
mod ppu;
mod ram2k;
mod rom;

use cpu::Cpu;
use debugger::Debugger;

struct GUIInstruction {
    addr: u16,
    text: String,
    breakpoint: bool,
}

pub struct Nes {
    cpu: cpu::Cpu,
    debugger: debugger::Debugger,
    step_next_count: u16,
    memory_dump: String,
    disasm: Vec<GUIInstruction>,
    stack_data: Vec<String>,
    pc: usize,
    image: Option<TextureHandle>,
    ran_instruction: bool,
}

impl Nes {
    fn default(filename: &String) -> Self {
        let mut cpu = Cpu::new();
        let debugger: Debugger = Debugger::new();
        let step_next_count: u16 = 0;
        cpu.bus.rom.load_rom(filename);
        cpu.reset();

        let disasm: Vec<GUIInstruction> = Vec::new();

        let stack_data: Vec<String> = Vec::new();

        Self {
            cpu,
            debugger,
            step_next_count,
            memory_dump: "".to_string(),
            disasm,
            stack_data,
            pc: 0,
            image: None,
            ran_instruction: false,
        }
    }

    pub fn new(ctx: &egui::Context, filename: &String) -> Self {
        let mut app: Nes = Nes::default(filename);

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
            let value = self.cpu.bus.read_ram_opcode_decoding(stack_addr);

            // Format: "SP+offset: address value"
            let offset = (stack_addr - 0x0100) as u8;

            stack_data.push(format!("{:02X}: {:02X}", offset, value));
        }

        self.stack_data = stack_data;
    }

    fn generate_memory_dump(&self) -> String {
        let memory: Vec<u8> = (0..0xFFFF)
            .map(|i: u16| self.cpu.bus.read_ram_opcode_decoding(i))
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

    fn ui_action_step(&mut self) {
        self.step_next_count = 1;
    }
    fn ui_action_big_step(&mut self) {
        self.step_next_count = 1000;
    }
}

impl App for Nes {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Step
        if ctx.input(|i| i.key_pressed(egui::Key::S)) {
            self.ui_action_step()
        }

        self.ran_instruction = false;
        while self.step_next_count > 0 {
            self.ran_instruction = true;
            self.cpu.execute_cpu_ppu();
            self.step_next_count -= 1;

            if self.debugger.hit_breakpoint(self.cpu.pc) {
                // hit breakpoint, stop.
                self.step_next_count = 0;
                break;
            }

            while !self.cpu.ready_to_execute_next_instruction() {
                self.cpu.execute_cpu_ppu();
            }
        }
        if (self.ran_instruction) {
            // SUPER SUPER EXPENSIVE.
            self.memory_dump = self.generate_memory_dump();
            ctx.request_repaint();
        }
        thread::sleep(time::Duration::from_millis(20)); // 50 fps

        let mut disasm: Vec<GUIInstruction> = Vec::new();
        let mut pc_addr_scan_ahead = self.cpu.pc;

        // TODO in the past

        // in the future
        for _i in 0..16 {
            let (instruction_str, instruction_size) =
                self.cpu.get_cpu_opcode_str(pc_addr_scan_ahead);
            disasm.push(GUIInstruction {
                addr: pc_addr_scan_ahead,
                text: instruction_str,
                breakpoint: self.debugger.hit_breakpoint(pc_addr_scan_ahead),
            });
            pc_addr_scan_ahead += instruction_size;
        }
        self.disasm = disasm;

        self.populate_stack_data();

        // Render image
        self.image = Some(ctx.load_texture(
            "ppu_preview",
            self.cpu.bus.ppu.gbuffer.clone(), // TODO look into replacing this clone with arc?
            egui::TextureOptions::default(),
        ));

        egui::TopBottomPanel::top("top").show(ctx, |ui: &mut egui::Ui| {
            ui.horizontal(|ui: &mut egui::Ui| {
                if ui.button("Step").clicked() {
                    self.ui_action_step()
                }
                if ui.button("Big Step").clicked() {
                    self.ui_action_big_step()
                }
                // if ui.button("Step Out").clicked() {} TODO step until a branch is true?
                if ui.button("Run").clicked() {}
                if ui.button("Pause").clicked() {
                    self.step_next_count = 0
                }
                if ui.button("Reset").clicked() {}
                if ui.button("Breakpoint").clicked() {}
            });
        });

        // Side panel first - spans full vertical space between top and bottom of window
        egui::SidePanel::right("sidebar")
            .default_width(250.0)
            .show(ctx, |ui| {
                if let Some(tex) = &self.image {
                    ui.add(
                        egui::Image::from_texture(tex)
                            .fit_to_exact_size(egui::vec2(ui.available_width(), 150.0)),
                    );
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
                        for (intruction_addr, ins) in self.disasm.iter_mut().enumerate() {
                            let is_pc = intruction_addr == self.pc;
                            let text: RichText = if ins.breakpoint {
                                if is_pc {
                                    RichText::new(format!(">{}", ins.text))
                                        .background_color(Color32::LIGHT_BLUE)
                                        .color(Color32::BLACK)
                                } else {
                                    RichText::new(format!(" {}", ins.text))
                                        .background_color(Color32::LIGHT_RED)
                                        .color(Color32::BLACK)
                                }
                            } else if is_pc {
                                RichText::new(format!(">{}", ins.text))
                                    .background_color(Color32::DEBUG_COLOR)
                                    .color(Color32::BLACK)
                            } else {
                                RichText::new(format!(" {}", ins.text))
                            };
                            let response = ui.selectable_label(ins.breakpoint, text);
                            if response.clicked() {
                                self.debugger.toggle_breakpoint(ins.addr, None);
                            }
                            if is_pc {
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
