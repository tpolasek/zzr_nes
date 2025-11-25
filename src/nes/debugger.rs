use std::collections::HashMap;

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
}
