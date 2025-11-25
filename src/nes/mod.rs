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
/*
This file contains the GUI implementation and it is also where we execute the emulator.
*/

use crate::nes::cpu::Opcode;
use cpu::Cpu;
use debugger::Debugger;

struct GUIInstruction {
    addr: u16,
    text: String,
    breakpoint_pc: bool,
    breakpoint_memory: bool,
}

pub struct Nes {
    cpu: cpu::Cpu,
    debugger: debugger::Debugger,
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
}

impl Nes {
    fn default(filename: &String) -> Self {
        let mut cpu = Cpu::new();
        let debugger: Debugger = Debugger::new();
        let step_next_count: u32 = 0;
        cpu.bus.rom.load_rom(filename);
        cpu.reset();

        let disasm: Vec<GUIInstruction> = Vec::new();

        let stack_data: Vec<String> = Vec::new();

        Self {
            cpu,
            debugger,
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
        }
    }

    pub fn new(ctx: &egui::Context, filename: &String) -> Self {
        let app: Nes = Nes::default(filename);

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
        let mut addresses_to_disam: Vec<u16> = Vec::new();
        if self.previous_pc != 0 && self.previous_pc != self.cpu.pc {
            addresses_to_disam.push(self.previous_pc);
        }
        for i in 0..40 {
            addresses_to_disam.push(self.cpu.pc + i);
        }

        let mut disasm: Vec<GUIInstruction> = Vec::new();
        // TODO get instructions before.
        // in the future
        for pc_addr_scan_ahead in addresses_to_disam {
            let current_opcode: &Opcode = self.cpu.get_optcode(pc_addr_scan_ahead);
            let instruction_str =
                current_opcode.get_instruction_decoded(&self.cpu, pc_addr_scan_ahead);

            //let instruction_size = current_opcode.get_opcode_byte_size();

            let memory_accessed =
                current_opcode.get_memory_addr_accessed_u16(&self.cpu, pc_addr_scan_ahead);

            disasm.push(GUIInstruction {
                addr: pc_addr_scan_ahead,
                text: instruction_str,
                breakpoint_pc: self.debugger.hit_breakpoint_pc(pc_addr_scan_ahead),
                breakpoint_memory: (memory_accessed.is_some()
                    && self
                        .debugger
                        .hit_breakpoint_memory_access(memory_accessed.unwrap())),
            });
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

    fn emulator_execution_loop(&mut self) {
        self.ran_instruction = false;
        while self.step_out_mode || self.step_next_count > 0 {
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
                self.step_next_count = 0;
                break;
            }
            let optcode = self.cpu.get_optcode(self.cpu.pc);
            let memory_accessed_u16 = optcode.get_memory_addr_accessed_u16(&self.cpu, self.cpu.pc);
            if memory_accessed_u16.is_some()
                && self.debugger.hit_breakpoint_memory_access(
                    memory_accessed_u16.expect("We check is_some() how is this possible?"),
                )
            {
                // hit breakpoint, stop.
                self.step_next_count = 0;
                break;
            }

            if self.step_out_mode && self.cpu.get_optcode(self.previous_pc).is_rts() {
                // hit RTS in step out mode, stop.
                self.step_out_mode = false;
                self.step_next_count = 0;
                break;
            }
        }
    }
}

impl App for Nes {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.emulator_execution_loop();
        thread::sleep(time::Duration::from_millis(20)); // 50 fps
        ctx.request_repaint();

        if self.ran_instruction {
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
                if ui.button("Run").clicked() {}
                if ui.button("Pause").clicked() {
                    self.step_next_count = 0
                }
                if ui.button("Reset").clicked() {
                    self.cpu.reset();
                    self.previous_pc = 0;
                    self.step_next_count = 0;
                    self.step_out_mode = false;
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
                        for (_, ins) in self.disasm.iter_mut().enumerate() {
                            let is_pc = ins.addr == self.cpu.pc;
                            let text: RichText = if ins.breakpoint_pc {
                                if is_pc {
                                    RichText::new(format!(">{}", ins.text))
                                        .background_color(Color32::LIGHT_BLUE)
                                        .color(Color32::BLACK)
                                } else {
                                    RichText::new(format!(" {}", ins.text))
                                        .background_color(Color32::LIGHT_RED)
                                        .color(Color32::BLACK)
                                }
                            } else if ins.breakpoint_memory {
                                if is_pc {
                                    RichText::new(format!(">{}", ins.text))
                                        .background_color(Color32::LIGHT_BLUE)
                                        .color(Color32::BLACK)
                                } else {
                                    RichText::new(format!(" {}", ins.text))
                                        .background_color(Color32::from_rgb(255, 165, 0)) // Orange
                                        .color(Color32::BLACK)
                                }
                            } else if is_pc {
                                RichText::new(format!(">{}", ins.text))
                                    .background_color(Color32::DEBUG_COLOR)
                                    .color(Color32::BLACK)
                            } else {
                                RichText::new(format!(" {}", ins.text))
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
