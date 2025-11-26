use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct Debugger {
    breakpoints_pc: HashMap<u16, Option<String>>,
    breakpoints_memory_access: HashMap<u16, Option<String>>,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            breakpoints_pc: HashMap::new(),
            breakpoints_memory_access: {
                let map = HashMap::new();
                //map.insert(0xEC10, None);
                map
            },
        }
    }
    // Memory Accessed Breakpoint

    pub fn hit_breakpoint_memory_access(&self, pc: u16) -> bool {
        self.breakpoints_memory_access.contains_key(&pc)
    }

    pub fn set_breakpoint_memory_access(&mut self, addr: u16, info_text: Option<String>) {
        self.breakpoints_memory_access.insert(addr, info_text);
    }

    pub fn toggle_breakpoint_memory_access(&mut self, addr: u16, info_text: Option<String>) {
        if self.hit_breakpoint_memory_access(addr) {
            self.remove_breakpoint_memory_access(addr);
        } else {
            self.set_breakpoint_memory_access(addr, info_text);
        }
    }
    pub fn remove_breakpoint_memory_access(&mut self, addr: u16) {
        self.breakpoints_memory_access.remove(&addr);
    }
    // Breakpoints on hitting Program Counter (PC) locations

    pub fn hit_breakpoint_pc(&self, pc: u16) -> bool {
        self.breakpoints_pc.contains_key(&pc)
    }

    pub fn set_breakpoint_pc(&mut self, addr: u16, info_text: Option<String>) {
        self.breakpoints_pc.insert(addr, info_text);
    }

    pub fn toggle_breakpoint_pc(&mut self, addr: u16, info_text: Option<String>) {
        if self.hit_breakpoint_pc(addr) {
            self.remove_breakpoint_pc(addr);
        } else {
            self.set_breakpoint_pc(addr, info_text);
        }
    }
    pub fn remove_breakpoint_pc(&mut self, addr: u16) {
        self.breakpoints_pc.remove(&addr);
    }

    pub fn load_debug_symbol<P: AsRef<Path>>(debug_file_path: P) -> HashMap<u16, String> {
        let mut symbols = HashMap::new();

        // Try to read the debug file
        let contents = match fs::read_to_string(debug_file_path) {
            Ok(content) => content,
            Err(_) => return symbols, // Return empty map if file can't be read
        };

        // Parse each line
        for line in contents.lines() {
            // Only process lines starting with "sym"
            if !line.starts_with("sym") {
                continue;
            }

            // Extract name and val from the line
            let mut name: Option<String> = None;
            let mut val: Option<u16> = None;

            // Split by comma to get key=value pairs
            for part in line.split(',') {
                let part = part.trim();

                if part.starts_with("name=") {
                    // Extract name between quotes
                    if let Some(start) = part.find('"') {
                        if let Some(end) = part.rfind('"') {
                            if start < end {
                                name = Some(part[start + 1..end].to_string());
                            }
                        }
                    }
                } else if part.starts_with("val=") {
                    // Extract hex value
                    let val_str = &part[4..]; // Skip "val="
                    // Parse hex string (e.g., "0x850C")
                    if let Ok(parsed_val) = u16::from_str_radix(val_str.trim_start_matches("0x"), 16) {
                        val = Some(parsed_val);
                    }
                }
            }

            // If both name and val were found, add to hashmap
            if let (Some(symbol_name), Some(addr)) = (name, val) {
                symbols.insert(addr, symbol_name);
            }
        }

        symbols
    }
}
