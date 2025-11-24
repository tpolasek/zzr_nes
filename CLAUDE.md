# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a NES (Nintendo Entertainment System) emulator written in Rust, focused on cycle-accurate CPU and PPU emulation with a minimalistic design. The emulator includes a built-in debugger with GUI support using egui/eframe.

## Build and Run Commands

```bash
# Build the project (optimized debug build)
cargo build

# Build release version
cargo build --release

# Run the emulator
cargo run

# Run tests
cargo test

# Run a specific test
cargo test <test_name>

# Format code
cargo fmt
```

Note: The project uses `opt-level = 3` in debug mode for performance, with overflow checks disabled.

## Architecture Overview

### Core Components

**Main Entry (src/main.rs)**
- Initializes the egui application with window dimensions (850x768)
- ROM path is currently hardcoded to `/Users/thomas/code/zzr_nes/roms/all_instrs.nes`
- Creates and runs the Nes struct which implements the eframe::App trait

**Nes Module (src/nes/mod.rs)**
- Central GUI and emulation coordinator
- Implements the egui App trait for the main render loop
- Manages the debugger interface with multiple panels:
  - Disassembly view with breakpoint support (PC breakpoints and memory access breakpoints)
  - Memory hex dump
  - Register display (A, X, Y, SP, PC, flags)
  - CPU stack visualization ($0100-$01FF)
  - PPU framebuffer preview
- Keyboard controls: 'S' key steps through one instruction
- Runs at ~50 FPS (20ms sleep per frame)

**CPU (src/nes/cpu.rs)**
- 6502 CPU emulation with full instruction set
- Uses a 256-entry OPCODE_LOOKUP table mapping opcodes to:
  - Instruction name
  - Addressing mode function
  - Operation function
  - Base cycle count
- Supports various addressing modes (ACC, IDX, ZPG, IMM, ABS, etc.)
- Tracks cycle-accurate execution via `tick_count`

**Bus (src/nes/bus.rs)**
- Memory mapper (currently Mapper 0 only)
- Routes CPU memory accesses to appropriate components:
  - $0000-$1FFF: 2KB internal RAM (mirrored)
  - $2000-$3FFF: PPU registers (mirrored every 8 bytes)
  - $4000-$4017: APU and I/O registers (including controller at $4016)
  - $6000-$7FFF: Work RAM (8KB)
  - $8000-$FFFF: PRG ROM
- Provides `read_ram_opcode_decoding()` for side-effect-free reads (used by debugger)
- Handles DMA cycles for PPU sprite memory transfers

**PPU (src/nes/ppu.rs)**
- Picture Processing Unit emulation
- Manages 256x240 framebuffer (ColorImage)
- Tracks scanline and pixel position
- Implements PPU registers:
  - PPUCTRL, PPUMASK, PPUSTATUS
  - OAMADDR, OAMDATA
  - PPUSCROLL, PPUADDR, PPUDATA
- Contains internal VRAM (2 nametable banks of 1KB each)
- Palette RAM (32 bytes)
- OAM RAM (sprite memory, 256 bytes)
- 64-entry palette lookup table for NES colors
- VBlank flag management and NMI triggering

**Debugger (src/nes/debugger.rs)**
- Manages two types of breakpoints:
  - PC breakpoints: Break when program counter reaches specific address
  - Memory access breakpoints: Break when instruction accesses specific memory location
- Uses HashMap for O(1) breakpoint lookups
- Supports optional info text for each breakpoint
- Breakpoints visualized in GUI with color coding:
  - Light red: PC breakpoint
  - Orange: Memory access breakpoint
  - Light blue: Current PC position

**ROM (src/nes/rom.rs)**
- Loads .nes ROM files (iNES format)
- Handles PRG ROM (program code) and CHR ROM (graphics data)
- Supports mirroring modes (Horizontal, Vertical, FourScreen)

**Controller (src/nes/controller.rs)**
- Handles NES controller input via memory-mapped I/O at $4016
- Planned key mappings (commented in code):
  - WASD: D-pad
  - K: A button
  - L: B button

**Other Components**
- `cpu_flag.rs`: CPU status flag register implementation using bitflags
- `ram2k.rs`: 2KB internal RAM and 8KB work RAM structures

### Memory Access Patterns

When modifying memory access code:
- Use `read_ram()` for normal CPU execution (may have side effects like clearing status flags)
- Use `read_ram_opcode_decoding()` for debugger/disassembly (no side effects)
- PPU register reads may clear flags or update internal state
- Memory writes to $4014 trigger OAM DMA transfers

### Cycle Timing

The emulator aims for cycle accuracy:
- CPU instructions execute over multiple cycles
- Use `ready_to_execute_next_instruction()` to check if CPU is ready for next instruction
- PPU runs in lockstep with CPU via `execute_cpu_ppu()`
- DMA transfers add cycles via `bus.dma_cycles`

### Debugger Integration

When adding new debugging features:
- PC breakpoints are toggled by clicking on instructions in the disassembly view
- Memory breakpoints can be set via `debugger.toggle_breakpoint_memory_access()`
- The debugger stops execution when breakpoints are hit during the step loop
- Use `get_cpu_opcode_metadata()` to get instruction information for disassembly
- Use `get_cpu_optcode_memory_access_u16()` to determine which memory address an instruction accesses

### GUI Rendering Notes

- Memory dump generation is EXTREMELY expensive (noted in code comments)
- Only regenerate memory dump when instructions execute (`ran_instruction` flag)
- The disassembly view shows 16 instructions ahead of current PC
- Stack display shows values from current SP+1 to $01FF
- All panels use monospace font with green-on-black terminal aesthetic
