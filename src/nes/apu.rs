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
                self.envelope_counter = self.volume;
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
        self.enabled && self.length_counter > 0 && self.timer >= 8
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

    pub fn output(&self) -> f32 {
        if !self.is_active() {
            return 0.0;
        }

        let duty_pattern = DUTY_CYCLES[self.duty as usize];
        let amplitude = if duty_pattern[self.sequence_position as usize] == 1 {
            self.envelope_volume as f32
        } else {
            0.0
        };

        // Normalize to -1.0 to 1.0 range
        (amplitude / 15.0) * 2.0 - 1.0
    }
}

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
    pub audio_buffer: Arc<Mutex<Vec<f32>>>,

    // Cycle tracking for audio sample generation
    cpu_cycles: u64,
    sample_counter: f32,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            pulse1: PulseChannel::new(),
            pulse2: PulseChannel::new(),
            frame_counter_mode: false,
            irq_inhibit: false,
            audio_buffer: Arc::new(Mutex::new(Vec::with_capacity(2048))),
            cpu_cycles: 0,
            sample_counter: 0.0,
        }
    }

    pub fn get_audio_buffer(&self) -> Arc<Mutex<Vec<f32>>> {
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
            self.pulse1.clock();
            self.pulse2.clock();
        }

        // Generate audio samples
        // We need to generate a sample every (CPU_FREQUENCY / SAMPLE_RATE) CPU cycles
        self.sample_counter += SAMPLE_RATE / CPU_FREQUENCY;

        if self.sample_counter >= 1.0 {
            self.sample_counter -= 1.0;

            // Mix the two pulse channels
            let pulse1_out = self.pulse1.output();
            let pulse2_out = self.pulse2.output();

            // Simple mixing (average)
            let mixed = (pulse1_out + pulse2_out) * 0.5;

            // Add to buffer
            if let Ok(mut buffer) = self.audio_buffer.lock() {
                buffer.push(mixed);

                // Prevent buffer from growing too large
                if buffer.len() > 4096 {
                    buffer.drain(0..2048);
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
    }
}
