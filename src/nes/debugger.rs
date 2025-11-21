use std::collections::HashMap;

pub struct Debugger {
    breakpoints: HashMap<u16, Option<String>>,
    address_labels: HashMap<u16, Option<String>>,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            address_labels: {
                let mut map = HashMap::new();
                // PPU Registers
                map.insert(0x2000, Some("PPUCTRL".to_string()));
                map.insert(0x2001, Some("PPUMASK".to_string()));
                map.insert(0x2002, Some("PPUSTATUS".to_string()));
                map.insert(0x2003, Some("OAMADDR".to_string()));
                map.insert(0x2004, Some("OAMDATA".to_string()));
                map.insert(0x2005, Some("PPUSCROLL".to_string()));
                map.insert(0x2006, Some("PPUADDR".to_string()));
                map.insert(0x2007, Some("PPUDATA".to_string()));
                // APU and I/O
                map.insert(0x4014, Some("OAMDMA".to_string()));
                map.insert(0x4015, Some("APU_CTRL".to_string()));
                map.insert(0x4016, Some("JOY1".to_string()));
                map.insert(0x4017, Some("JOY2".to_string()));
                // APU Square 1
                map.insert(0x4000, Some("SQ1_VOL".to_string()));
                map.insert(0x4001, Some("SQ1_SWEEP".to_string()));
                map.insert(0x4002, Some("SQ1_LO".to_string()));
                map.insert(0x4003, Some("SQ1_HI".to_string()));
                // APU Square 2
                map.insert(0x4004, Some("SQ2_VOL".to_string()));
                map.insert(0x4005, Some("SQ2_SWEEP".to_string()));
                map.insert(0x4006, Some("SQ2_LO".to_string()));
                map.insert(0x4007, Some("SQ2_HI".to_string()));
                // APU Triangle
                map.insert(0x4008, Some("TRI_LINEAR".to_string()));
                map.insert(0x400A, Some("TRI_LO".to_string()));
                map.insert(0x400B, Some("TRI_HI".to_string()));
                // APU Noise
                map.insert(0x400C, Some("NOISE_VOL".to_string()));
                map.insert(0x400E, Some("NOISE_LO".to_string()));
                map.insert(0x400F, Some("NOISE_HI".to_string()));
                // APU DMC
                map.insert(0x4010, Some("DMC_FREQ".to_string()));
                map.insert(0x4011, Some("DMC_RAW".to_string()));
                map.insert(0x4012, Some("DMC_START".to_string()));
                map.insert(0x4013, Some("DMC_LEN".to_string()));
                // Interrupt Vectors
                map.insert(0xFFFA, Some("NMI_VEC_LO".to_string()));
                map.insert(0xFFFB, Some("NMI_VEC_HI".to_string()));
                map.insert(0xFFFC, Some("RESET_VEC_LO".to_string()));
                map.insert(0xFFFD, Some("RESET_VEC_HI".to_string()));
                map.insert(0xFFFE, Some("IRQ_VEC_LO".to_string()));
                map.insert(0xFFFF, Some("IRQ_VEC_HI".to_string()));
                map
            }
        }
    }

    pub fn hit_breakpoint(&self, pc: u16) -> bool {
        self.breakpoints.contains_key(&pc)
    }

    /// Sets a breakpoint at the given address with optional information text.
    pub fn set_breakpoint(&mut self, addr: u16, info_text: Option<String>) {
        self.breakpoints.insert(addr, info_text);
    }

    /// Removes the breakpoint at the given address.
    pub fn remove_breakpoint(&mut self, addr: u16) {
        self.breakpoints.remove(&addr);
    }
}
