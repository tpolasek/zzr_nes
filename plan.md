# NES Emulator + GUI Debugger Integration Plan

## Current Todo List
- [x] Gather context from GUI code and emulator files
- [x] Analyze GUI data requirements: disassembly, registers, hex dump, PPU image, mem writes, PC, tick count, breakpoints, controls (step, run, pause, reset)
- [-] Define shared data structures in nes/mod.rs (e.g., pub struct Instruction matching GUI)
- [ ] Expose emulator state via public fields/methods in Nes:
  - Make cpu: Cpu public
  - Add pub breakpoints: std::collections::HashSet<u16>
  - Add breakpoint management methods (toggle_breakpoint(addr: u16), has_breakpoint(addr: u16), clear_all_breakpoints())
- [ ] Add disassembly support:
  - Modify cpu.rs get_cpu_opcode_str to &self using read_ram_opcode_decoding
  - Add Nes::get_disasm(&self, start_addr: u16, count: usize) -> Vec<Instruction>
- [ ] Add state getters:
  - Nes::get_registers(&self) -> Vec<(&'static str, String)>
  - Nes::get_memory_dump(&self, start: u16, len: usize) -> Vec<u8>
  - Nes::get_ppu_framebuffer(&self) -> &[u32]
  - Nes::get_tick_count(&self) -> u64
- [ ] Add memory write logging:
  - Modify bus.rs to add pub recent_writes: VecDeque<String, capacity 100>
  - In bus.write_ram, log writes (e.g., format!("WR{:04X}:{:02X} {}", location, value)) if not PPU/APU
  - Add Nes::get_mem_writes(&self) -> &VecDeque<String>
- [ ] Add control methods to Nes:
  - pub fn reset(&mut self)
  - pub fn step_instruction(&mut self)  // advance to next instr boundary
  - pub fn tick(&mut self)  // single cpu_ppu cycle
  - pub fn execute_frame(&mut self) -> bool  // run to next vblank, return true if breakpoint hit
- [ ] Refactor execute_rom in nes/mod.rs:
  - Separate load_rom(&mut self, filename: &str)
  - Remove blocking loop, prepare for external loop (e.g., GUI)
- [ ] Update GUI integration plan:
  - In new main.rs or nes_debugger, create Nes, load ROM, pass to DebuggerApp, replace mocks with nes getters
  - Handle buttons to call nes methods, use nes.tick() or execute_frame() in update loop if running
- [ ] Test compatibility:
  - Verify all GUI panels use real data
  - Ensure breakpoints, stepping work

## Nes Struct and Methods - Class Diagram

```mermaid
classDiagram
    class Nes {
        <<emulator>>
        +cpu Cpu
        +breakpoints HashSet~u16~
        +mem_writes VecDeque~String~
        +tick_count u64
        +new() Nes
        +load_rom(filename~String~) void
        +reset() void
        +tick() void
        +step_instruction() void
        +execute_frame() bool
        +get_disasm(start~u16~, count~usize~) Vec~Instruction~
        +get_registers() Vec~(&str, String)~
        +get_memory_dump(start~u16~, len~usize~) Vec~u8~
        +get_ppu_framebuffer() ~[u32]~
        +get_tick_count() u64
        +get_mem_writes() ~VecDeque~String~
        +toggle_breakpoint(addr~u16~) void
        +has_breakpoint(addr~u16~) bool
        +clear_all_breakpoints() void
    }
    class Instruction {
        +addr u16
        +bytes Vec~u8~
        +text String
        +breakpoint bool
    }
    class Cpu {
        <<6502>>
        +bus Bus
        +pc u16
        +reg_a u8
        +reg_x u8
        +reg_y u8
        +reg_sp u8
        +flag Flag
        +tick_count u64
        +get_cpu_opcode_str(addr~u16~) (String, u16)
        +get_cpu_state_str() String
        +tick() void
        +ready_to_execute_next_instruction() bool
    }
    class Bus {
        <<memory>>
        +ppu Ppu
        +rom Rom
        +recent_writes VecDeque~String~
        +read_ram(addr~u16~) u8
        +write_ram(addr~u16~, val~u8~) void
    }
    class Ppu {
        <<video>>
        +gbuffer Vec~u32~
        +tick() void
        +is_vblank() bool
    }
    Nes *-- Cpu : contains
    Cpu *-- Bus : contains
    Bus *-- Ppu : contains
    Nes ||--o{ Instruction : generates
    Nes ||--o{ mem_writes : logs
```

## Next Steps
Review this plan and diagram. If approved, proceed to implementation in code mode.