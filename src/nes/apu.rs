use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

const CPU_FREQUENCY: f32 = 1_789_773.0; // NES CPU clock frequency in Hz
const SAMPLE_RATE: f32 = 44100.0; // Standard audio sample rate

// Square wave duty cycle patterns (8 steps each)
const DUTY_CYCLES: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0], // 12.5%
    [0, 1, 1, 0, 0, 0, 0, 0], // 25%
    [0, 1, 1, 1, 1, 0, 0, 0], // 50%
    [1, 0, 0, 1, 1, 1, 1, 1], // 25% negated
];

pub struct PulseChannel {
    enabled: bool,

    // $4000/$4004 - Duty, envelope
    duty: u8,
    length_counter_halt: bool, // also acts as envelope loop flag
    constant_volume: bool,
    volume: u8, // also envelope period

    // $4001/$4005 - Sweep
    sweep_enabled: bool,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,

    // $4002/$4006 - Timer low
    // $4003/$4007 - Length counter load, timer high
    timer: u16,
    timer_counter: u16,

    length_counter: u8,

    // Internal state
    sequence_position: u8,
    envelope_counter: u8,
    envelope_divider: u8,
    envelope_volume: u8,

    // Sweep internal state
    sweep_divider: u8,
    sweep_reload: bool,
}

impl PulseChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            duty: 0,
            length_counter_halt: false,
            constant_volume: false,
            volume: 0,
            sweep_enabled: false,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            timer: 0,
            timer_counter: 0,
            length_counter: 0,
            sequence_position: 0,
            envelope_counter: 0,
            envelope_divider: 0,
            envelope_volume: 0,
            sweep_divider: 0,
            sweep_reload: false,
        }
    }

    pub fn write_register(&mut self, addr: u8, value: u8) {
        match addr {
            0 => {
                // $4000/$4004 - DDLC VVVV
                self.duty = (value >> 6) & 0x03;
                self.length_counter_halt = (value & 0x20) != 0;
                self.constant_volume = (value & 0x10) != 0;
                self.volume = value & 0x0F;
            }
            1 => {
                // $4001/$4005 - EPPP NSSS
                self.sweep_enabled = (value & 0x80) != 0;
                self.sweep_period = (value >> 4) & 0x07;
                self.sweep_negate = (value & 0x08) != 0;
                self.sweep_shift = value & 0x07;
                self.sweep_reload = true;
            }
            2 => {
                // $4002/$4006 - Timer low 8 bits
                self.timer = (self.timer & 0xFF00) | (value as u16);
            }
            3 => {
                // $4003/$4007 - LLLL LTTT
                self.timer = (self.timer & 0x00FF) | (((value & 0x07) as u16) << 8);

                // Load length counter if enabled
                if self.enabled {
                    let length_index = (value >> 3) & 0x1F;
                    self.length_counter = LENGTH_TABLE[length_index as usize];
                }

                // Reset sequence and envelope
                self.sequence_position = 0;
                self.envelope_counter = 15;
            }
            _ => {}
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }

    pub fn is_active(&self) -> bool {
        self.enabled && self.length_counter > 0 && self.timer >= 8 && !self.is_sweep_muted()
    }

    pub fn clock(&mut self) {
        if self.timer_counter == 0 {
            self.timer_counter = self.timer;
            self.sequence_position = (self.sequence_position + 1) % 8;
        } else {
            self.timer_counter -= 1;
        }
    }

    pub fn clock_length_counter(&mut self) {
        if !self.length_counter_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn clock_envelope(&mut self) {
        if self.envelope_divider == 0 {
            self.envelope_divider = self.volume;

            if self.envelope_counter > 0 {
                self.envelope_counter -= 1;
            } else if self.length_counter_halt {
                self.envelope_counter = 15;
            }
        } else {
            self.envelope_divider -= 1;
        }

        self.envelope_volume = if self.constant_volume {
            self.volume
        } else {
            self.envelope_counter
        };
    }

    pub fn clock_sweep(&mut self, is_pulse1: bool) {
        // Calculate target period using right shift
        let change_amount = self.timer >> self.sweep_shift;

        let target_period = if self.sweep_negate {
            if is_pulse1 {
                // Pulse 1 uses ones' complement (add the complement)
                self.timer.wrapping_sub(change_amount).wrapping_sub(1)
            } else {
                // Pulse 2 uses two's complement
                self.timer.wrapping_sub(change_amount)
            }
        } else {
            self.timer.wrapping_add(change_amount)
        };

        // Muting conditions
        let is_muted = self.timer < 8 || target_period > 0x7FF;

        // Update period if conditions are met
        if self.sweep_divider == 0 && self.sweep_enabled && self.sweep_shift > 0 && !is_muted {
            self.timer = target_period;
        }

        // Clock divider
        if self.sweep_divider == 0 || self.sweep_reload {
            self.sweep_divider = self.sweep_period;
            self.sweep_reload = false;
        } else {
            self.sweep_divider -= 1;
        }
    }

    fn is_sweep_muted(&self) -> bool {
        if self.sweep_shift == 0 {
            return false;
        }

        let change_amount = self.timer >> self.sweep_shift;
        let target_period = if self.sweep_negate {
            self.timer.wrapping_sub(change_amount)
        } else {
            self.timer.wrapping_add(change_amount)
        };

        self.timer < 8 || target_period > 0x7FF
    }

    pub fn output(&self) -> u8 {
        if !self.is_active() {
            return 0;
        }

        let duty_pattern = DUTY_CYCLES[self.duty as usize];
        if duty_pattern[self.sequence_position as usize] == 1 {
            self.envelope_volume
        } else {
            0
        }
    }
}

// Pre-calculated nonlinear mixing lookup table
// Formula: pulse_out = 95.88 / ((8128 / (pulse1 + pulse2)) + 100)
// Index is pulse1 + pulse2 (range 0-30)
const PULSE_MIXING_TABLE: [f32; 31] = [
    0.0, 0.011609, 0.023010, 0.034207, 0.045207, 0.056016, 0.066639, 0.077080,
    0.087345, 0.097437, 0.107362, 0.117123, 0.126724, 0.136169, 0.145461, 0.154604,
    0.163602, 0.172458, 0.181175, 0.189757, 0.198207, 0.206528, 0.214723, 0.222794,
    0.230745, 0.238578, 0.246296, 0.253901, 0.261396, 0.268784, 0.276066,
];

// NES length counter lookup table
const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

pub struct Apu {
    pub pulse1: PulseChannel,
    pub pulse2: PulseChannel,

    // Frame counter
    frame_counter_mode: bool, // false = 4-step, true = 5-step
    irq_inhibit: bool,

    // Audio output
    pub audio_buffer: Arc<Mutex<VecDeque<f32>>>,

    // Cycle tracking for audio sample generation
    cpu_cycles: u64,
    sample_counter: f32,

    // Frame counter tracking
    apu_cycles: u64,
    frame_counter_reset: bool,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            pulse1: PulseChannel::new(),
            pulse2: PulseChannel::new(),
            frame_counter_mode: false,
            irq_inhibit: false,
            audio_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(2048))),
            cpu_cycles: 0,
            sample_counter: 0.0,
            apu_cycles: 0,
            frame_counter_reset: false,
        }
    }

    pub fn get_audio_buffer(&self) -> Arc<Mutex<VecDeque<f32>>> {
        Arc::clone(&self.audio_buffer)
    }

    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x4000..=0x4003 => {
                self.pulse1.write_register((addr & 0x03) as u8, value);
            }
            0x4004..=0x4007 => {
                self.pulse2.write_register((addr & 0x03) as u8, value);
            }
            0x4015 => {
                // Status register - enable/disable channels
                self.pulse1.set_enabled((value & 0x01) != 0);
                self.pulse2.set_enabled((value & 0x02) != 0);
            }
            0x4017 => {
                // Frame counter
                self.frame_counter_mode = (value & 0x80) != 0;
                self.irq_inhibit = (value & 0x40) != 0;
                self.frame_counter_reset = true;
            }
            _ => {}
        }
    }

    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0x4015 => {
                // Status register
                let mut status = 0;
                if self.pulse1.length_counter > 0 {
                    status |= 0x01;
                }
                if self.pulse2.length_counter > 0 {
                    status |= 0x02;
                }
                status
            }
            _ => 0,
        }
    }

    // Called once per CPU cycle
    pub fn clock(&mut self) {
        self.cpu_cycles += 1;

        // The APU runs at half the CPU speed for the pulse channels
        if self.cpu_cycles % 2 == 0 {
            self.apu_cycles += 1;
            self.pulse1.clock();
            self.pulse2.clock();

            // Clock the frame counter
            self.clock_frame_counter();
        }

        // Generate audio samples
        // We need to generate a sample every (CPU_FREQUENCY / SAMPLE_RATE) CPU cycles
        self.sample_counter += SAMPLE_RATE / CPU_FREQUENCY;

        if self.sample_counter >= 1.0 {
            self.sample_counter -= 1.0;

            // Get raw outputs (0-15 range)
            let pulse1_out = self.pulse1.output() as usize;
            let pulse2_out = self.pulse2.output() as usize;

            // Apply nonlinear mixing using lookup table
            let pulse_sum = pulse1_out + pulse2_out;
            let mixed = PULSE_MIXING_TABLE[pulse_sum];

            // Convert from 0.0-0.276 range to -1.0 to 1.0 for audio output
            let mixed = (mixed - 0.138) * 7.25;

            // Add to buffer
            if let Ok(mut buffer) = self.audio_buffer.lock() {
                buffer.push_back(mixed);

                // Prevent buffer from growing too large
                while buffer.len() > 4096 {
                    buffer.pop_front();
                }
            }
        }
    }

    // Called at 240 Hz for envelope and length counter updates
    pub fn clock_quarter_frame(&mut self) {
        self.pulse1.clock_envelope();
        self.pulse2.clock_envelope();
    }

    pub fn clock_half_frame(&mut self) {
        self.clock_quarter_frame();
        self.pulse1.clock_length_counter();
        self.pulse2.clock_length_counter();
        self.pulse1.clock_sweep(true);   // true = is pulse 1
        self.pulse2.clock_sweep(false);  // false = is pulse 2
    }

    fn clock_frame_counter(&mut self) {
        // Handle frame counter reset (write to $4017)
        if self.frame_counter_reset {
            self.apu_cycles = 0;
            self.frame_counter_reset = false;

            // In 5-step mode, immediately clock half frame
            if self.frame_counter_mode {
                self.clock_half_frame();
            }
            return;
        }

        if !self.frame_counter_mode {
            // 4-step mode: quarter frames at 3728, 7456, 11185; half frames at 7456, 14914
            match self.apu_cycles {
                3728 => {
                    self.clock_quarter_frame();
                }
                7456 => {
                    self.clock_half_frame(); // This also calls quarter frame
                }
                11185 => {
                    self.clock_quarter_frame();
                }
                14914 => {
                    self.clock_half_frame();
                    self.apu_cycles = 0; // Reset cycle counter
                    // TODO: Generate IRQ if !self.irq_inhibit
                }
                _ => {}
            }
        } else {
            // 5-step mode: quarter frames at 3728, 7456, 11185, 18640; half frame at 18640
            match self.apu_cycles {
                3728 | 7456 | 11185 => {
                    self.clock_quarter_frame();
                }
                18640 => {
                    self.clock_half_frame();
                    self.apu_cycles = 0; // Reset cycle counter
                }
                _ => {}
            }
        }
    }
}
