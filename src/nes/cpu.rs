use crate::nes::bus::Bus;
use crate::nes::cpu_flag;
use crate::nes::debugger::Debugger;

fn decrement_u8(value: u8) -> u8 {
    if value == 0 {
        return 0xFF;
    }
    return value - 1;
}

fn increment_u8(value: u8) -> u8 {
    if value == 0xFF {
        return 0;
    }
    return value + 1;
}

///////////////////////////////////////////////
static OPCODE_LOOKUP: [Opcode; 256] = [
    Opcode {
        name: "BRK",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_BRK,
        cycles: 7,
    },
    Opcode {
        name: "ORA",
        addr_t: Cpu::addr_IDX,
        operation: Cpu::op_ORA,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 8,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 3,
    },
    Opcode {
        name: "ORA",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_ORA,
        cycles: 3,
    },
    Opcode {
        name: "ASL",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_ASL,
        cycles: 5,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 5,
    },
    Opcode {
        name: "PHP",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_PHP,
        cycles: 3,
    },
    Opcode {
        name: "ORA",
        addr_t: Cpu::addr_IMM,
        operation: Cpu::op_ORA,
        cycles: 2,
    },
    Opcode {
        name: "ASL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_ASL,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "ORA",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_ORA,
        cycles: 4,
    },
    Opcode {
        name: "ASL",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_ASL,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 6,
    },
    Opcode {
        name: "BPL",
        addr_t: Cpu::addr_REL,
        operation: Cpu::op_BPL,
        cycles: 2,
    },
    Opcode {
        name: "ORA",
        addr_t: Cpu::addr_IDY,
        operation: Cpu::op_ORA,
        cycles: 5,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 8,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "ORA",
        addr_t: Cpu::addr_ZPX,
        operation: Cpu::op_ORA,
        cycles: 4,
    },
    Opcode {
        name: "ASL",
        addr_t: Cpu::addr_ZPX,
        operation: Cpu::op_ASL,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 6,
    },
    Opcode {
        name: "CLC",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_CLC,
        cycles: 2,
    },
    Opcode {
        name: "ORA",
        addr_t: Cpu::addr_ABY,
        operation: Cpu::op_ORA,
        cycles: 4,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 7,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "ORA",
        addr_t: Cpu::addr_ABX,
        operation: Cpu::op_ORA,
        cycles: 4,
    },
    Opcode {
        name: "ASL",
        addr_t: Cpu::addr_ABX,
        operation: Cpu::op_ASL,
        cycles: 7,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 7,
    },
    Opcode {
        name: "JSR",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_JSR,
        cycles: 6,
    },
    Opcode {
        name: "AND",
        addr_t: Cpu::addr_IDX,
        operation: Cpu::op_AND,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 8,
    },
    Opcode {
        name: "BIT",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_BIT,
        cycles: 3,
    },
    Opcode {
        name: "AND",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_AND,
        cycles: 3,
    },
    Opcode {
        name: "ROL",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_ROL,
        cycles: 5,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 5,
    },
    Opcode {
        name: "PLP",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_PLP,
        cycles: 4,
    },
    Opcode {
        name: "AND",
        addr_t: Cpu::addr_IMM,
        operation: Cpu::op_AND,
        cycles: 2,
    },
    Opcode {
        name: "ROL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_ROL,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "BIT",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_BIT,
        cycles: 4,
    },
    Opcode {
        name: "AND",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_AND,
        cycles: 4,
    },
    Opcode {
        name: "ROL",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_ROL,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 6,
    },
    Opcode {
        name: "BMI",
        addr_t: Cpu::addr_REL,
        operation: Cpu::op_BMI,
        cycles: 2,
    },
    Opcode {
        name: "AND",
        addr_t: Cpu::addr_IDY,
        operation: Cpu::op_AND,
        cycles: 5,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 8,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "AND",
        addr_t: Cpu::addr_ZPX,
        operation: Cpu::op_AND,
        cycles: 4,
    },
    Opcode {
        name: "ROL",
        addr_t: Cpu::addr_ZPX,
        operation: Cpu::op_ROL,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 6,
    },
    Opcode {
        name: "SEC",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_SEC,
        cycles: 2,
    },
    Opcode {
        name: "AND",
        addr_t: Cpu::addr_ABY,
        operation: Cpu::op_AND,
        cycles: 4,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 7,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "AND",
        addr_t: Cpu::addr_ABX,
        operation: Cpu::op_AND,
        cycles: 4,
    },
    Opcode {
        name: "ROL",
        addr_t: Cpu::addr_ABX,
        operation: Cpu::op_ROL,
        cycles: 7,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 7,
    },
    Opcode {
        name: "RTI",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_RTI,
        cycles: 6,
    },
    Opcode {
        name: "XOR",
        addr_t: Cpu::addr_IDX,
        operation: Cpu::op_EOR,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 8,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 3,
    },
    Opcode {
        name: "XOR",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_EOR,
        cycles: 3,
    },
    Opcode {
        name: "LSR",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_LSR,
        cycles: 5,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 5,
    },
    Opcode {
        name: "PHA",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_PHA,
        cycles: 3,
    },
    Opcode {
        name: "XOR",
        addr_t: Cpu::addr_IMM,
        operation: Cpu::op_EOR,
        cycles: 2,
    },
    Opcode {
        name: "LSR",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_LSR,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "JMP",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_JMP,
        cycles: 3,
    },
    Opcode {
        name: "XOR",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_EOR,
        cycles: 4,
    },
    Opcode {
        name: "LSR",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_LSR,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 6,
    },
    Opcode {
        name: "BVC",
        addr_t: Cpu::addr_REL,
        operation: Cpu::op_BVC,
        cycles: 2,
    },
    Opcode {
        name: "XOR",
        addr_t: Cpu::addr_IDY,
        operation: Cpu::op_EOR,
        cycles: 5,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 8,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "XOR",
        addr_t: Cpu::addr_ZPX,
        operation: Cpu::op_EOR,
        cycles: 4,
    },
    Opcode {
        name: "LSR",
        addr_t: Cpu::addr_ZPX,
        operation: Cpu::op_LSR,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 6,
    },
    Opcode {
        name: "CLI",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_CLI,
        cycles: 2,
    },
    Opcode {
        name: "XOR",
        addr_t: Cpu::addr_ABY,
        operation: Cpu::op_EOR,
        cycles: 4,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 7,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "XOR",
        addr_t: Cpu::addr_ABX,
        operation: Cpu::op_EOR,
        cycles: 4,
    },
    Opcode {
        name: "LSR",
        addr_t: Cpu::addr_ABX,
        operation: Cpu::op_LSR,
        cycles: 7,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 7,
    },
    Opcode {
        name: "RTS",
        addr_t: Cpu::addr_NUL,
        operation: Cpu::op_RTS,
        cycles: 6,
    },
    Opcode {
        name: "ADC",
        addr_t: Cpu::addr_IDX,
        operation: Cpu::op_ADC,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 8,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 3,
    },
    Opcode {
        name: "ADC",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_ADC,
        cycles: 3,
    },
    Opcode {
        name: "ROR",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_ROR,
        cycles: 5,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 5,
    },
    Opcode {
        name: "PLA",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_PLA,
        cycles: 4,
    },
    Opcode {
        name: "ADC",
        addr_t: Cpu::addr_IMM,
        operation: Cpu::op_ADC,
        cycles: 2,
    },
    Opcode {
        name: "ROR",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_ROR,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "JMP",
        addr_t: Cpu::addr_IND,
        operation: Cpu::op_JMP,
        cycles: 5,
    },
    Opcode {
        name: "ADC",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_ADC,
        cycles: 4,
    },
    Opcode {
        name: "ROR",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_ROR,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 6,
    },
    Opcode {
        name: "BVS",
        addr_t: Cpu::addr_REL,
        operation: Cpu::op_BVS,
        cycles: 2,
    },
    Opcode {
        name: "ADC",
        addr_t: Cpu::addr_IDY,
        operation: Cpu::op_ADC,
        cycles: 5,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 8,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "ADC",
        addr_t: Cpu::addr_ZPX,
        operation: Cpu::op_ADC,
        cycles: 4,
    },
    Opcode {
        name: "ROR",
        addr_t: Cpu::addr_ZPX,
        operation: Cpu::op_ROR,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 6,
    },
    Opcode {
        name: "SEI",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_SEI,
        cycles: 2,
    },
    Opcode {
        name: "ADC",
        addr_t: Cpu::addr_ABY,
        operation: Cpu::op_ADC,
        cycles: 4,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 7,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "ADC",
        addr_t: Cpu::addr_ABX,
        operation: Cpu::op_ADC,
        cycles: 4,
    },
    Opcode {
        name: "ROR",
        addr_t: Cpu::addr_ABX,
        operation: Cpu::op_ROR,
        cycles: 7,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 7,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "STA",
        addr_t: Cpu::addr_IDX,
        operation: Cpu::op_STA,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 6,
    },
    Opcode {
        name: "STX",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_STY,
        cycles: 3,
    },
    Opcode {
        name: "STA",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_STA,
        cycles: 3,
    },
    Opcode {
        name: "STX",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_STX,
        cycles: 3,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 3,
    },
    Opcode {
        name: "DEY",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_DEY,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "TXA",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_TXA,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "STY",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_STY,
        cycles: 4,
    },
    Opcode {
        name: "STA",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_STA,
        cycles: 4,
    },
    Opcode {
        name: "STX",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_STX,
        cycles: 4,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "BCC",
        addr_t: Cpu::addr_REL,
        operation: Cpu::op_BCC,
        cycles: 2,
    },
    Opcode {
        name: "STA",
        addr_t: Cpu::addr_IDY,
        operation: Cpu::op_STA,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 6,
    },
    Opcode {
        name: "STX",
        addr_t: Cpu::addr_ZPX,
        operation: Cpu::op_STY,
        cycles: 4,
    },
    Opcode {
        name: "STA",
        addr_t: Cpu::addr_ZPX,
        operation: Cpu::op_STA,
        cycles: 4,
    },
    Opcode {
        name: "STA",
        addr_t: Cpu::addr_ZPY,
        operation: Cpu::op_STA,
        cycles: 4,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "TYA",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_TYA,
        cycles: 2,
    },
    Opcode {
        name: "STA",
        addr_t: Cpu::addr_ABY,
        operation: Cpu::op_STA,
        cycles: 5,
    },
    Opcode {
        name: "TXS",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_TXS,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 5,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 5,
    },
    Opcode {
        name: "STA",
        addr_t: Cpu::addr_ABX,
        operation: Cpu::op_STA,
        cycles: 5,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 5,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 5,
    },
    Opcode {
        name: "LDY",
        addr_t: Cpu::addr_IMM,
        operation: Cpu::op_LDY,
        cycles: 2,
    },
    Opcode {
        name: "LDA",
        addr_t: Cpu::addr_IDX,
        operation: Cpu::op_LDA,
        cycles: 6,
    },
    Opcode {
        name: "LDX",
        addr_t: Cpu::addr_IMM,
        operation: Cpu::op_LDX,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 6,
    },
    Opcode {
        name: "LDY",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_LDY,
        cycles: 3,
    },
    Opcode {
        name: "LDA",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_LDA,
        cycles: 3,
    },
    Opcode {
        name: "LDX",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_LDX,
        cycles: 3,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 3,
    },
    Opcode {
        name: "TAY",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_TAY,
        cycles: 2,
    },
    Opcode {
        name: "LDA",
        addr_t: Cpu::addr_IMM,
        operation: Cpu::op_LDA,
        cycles: 2,
    },
    Opcode {
        name: "TAX",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_TAX,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "LDY",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_LDY,
        cycles: 4,
    },
    Opcode {
        name: "LDA",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_LDA,
        cycles: 4,
    },
    Opcode {
        name: "LDX",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_LDX,
        cycles: 4,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "BCS",
        addr_t: Cpu::addr_REL,
        operation: Cpu::op_BCS,
        cycles: 2,
    },
    Opcode {
        name: "LDA",
        addr_t: Cpu::addr_IDY,
        operation: Cpu::op_LDA,
        cycles: 5,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 5,
    },
    Opcode {
        name: "LDY",
        addr_t: Cpu::addr_ZPX,
        operation: Cpu::op_LDY,
        cycles: 4,
    },
    Opcode {
        name: "LDA",
        addr_t: Cpu::addr_ZPX,
        operation: Cpu::op_LDA,
        cycles: 4,
    },
    Opcode {
        name: "LDX",
        addr_t: Cpu::addr_ZPY,
        operation: Cpu::op_LDX,
        cycles: 4,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "CLV",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_CLV,
        cycles: 2,
    },
    Opcode {
        name: "LDA",
        addr_t: Cpu::addr_ABY,
        operation: Cpu::op_LDA,
        cycles: 4,
    },
    Opcode {
        name: "TSX",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_TSX,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "LDY",
        addr_t: Cpu::addr_ABX,
        operation: Cpu::op_LDY,
        cycles: 4,
    },
    Opcode {
        name: "LDA",
        addr_t: Cpu::addr_ABX,
        operation: Cpu::op_LDA,
        cycles: 4,
    },
    Opcode {
        name: "LDX",
        addr_t: Cpu::addr_ABY,
        operation: Cpu::op_LDX,
        cycles: 4,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "CPY",
        addr_t: Cpu::addr_IMM,
        operation: Cpu::op_CPY,
        cycles: 2,
    },
    Opcode {
        name: "CMP",
        addr_t: Cpu::addr_IDX,
        operation: Cpu::op_CMP,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 8,
    },
    Opcode {
        name: "CPY",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_CPY,
        cycles: 3,
    },
    Opcode {
        name: "CMP",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_CMP,
        cycles: 3,
    },
    Opcode {
        name: "DEC",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_DEC,
        cycles: 5,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 5,
    },
    Opcode {
        name: "INY",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_INY,
        cycles: 2,
    },
    Opcode {
        name: "CMP",
        addr_t: Cpu::addr_IMM,
        operation: Cpu::op_CMP,
        cycles: 2,
    },
    Opcode {
        name: "DEX",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_DEX,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "CPY",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_CPY,
        cycles: 4,
    },
    Opcode {
        name: "CMP",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_CMP,
        cycles: 4,
    },
    Opcode {
        name: "DEC",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_DEC,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 6,
    },
    Opcode {
        name: "BNE",
        addr_t: Cpu::addr_REL,
        operation: Cpu::op_BNE,
        cycles: 2,
    },
    Opcode {
        name: "CMP",
        addr_t: Cpu::addr_IDY,
        operation: Cpu::op_CMP,
        cycles: 5,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 8,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "CMP",
        addr_t: Cpu::addr_ZPX,
        operation: Cpu::op_CMP,
        cycles: 4,
    },
    Opcode {
        name: "DEC",
        addr_t: Cpu::addr_ZPX,
        operation: Cpu::op_DEC,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 6,
    },
    Opcode {
        name: "CLD",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_CLD,
        cycles: 2,
    },
    Opcode {
        name: "CMP",
        addr_t: Cpu::addr_ABY,
        operation: Cpu::op_CMP,
        cycles: 4,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 7,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "CMP",
        addr_t: Cpu::addr_ABX,
        operation: Cpu::op_CMP,
        cycles: 4,
    },
    Opcode {
        name: "DEC",
        addr_t: Cpu::addr_ABX,
        operation: Cpu::op_DEC,
        cycles: 7,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 7,
    },
    Opcode {
        name: "CPX",
        addr_t: Cpu::addr_IMM,
        operation: Cpu::op_CPX,
        cycles: 2,
    },
    Opcode {
        name: "SBC",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_SBC,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 8,
    },
    Opcode {
        name: "CPX",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_CPX,
        cycles: 3,
    },
    Opcode {
        name: "SBC",
        addr_t: Cpu::addr_ZPX,
        operation: Cpu::op_SBC,
        cycles: 3,
    },
    Opcode {
        name: "INC",
        addr_t: Cpu::addr_ZPG,
        operation: Cpu::op_INC,
        cycles: 5,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 5,
    },
    Opcode {
        name: "INX",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_INX,
        cycles: 2,
    },
    Opcode {
        name: "SBC",
        addr_t: Cpu::addr_IMM,
        operation: Cpu::op_SBC,
        cycles: 2,
    },
    Opcode {
        name: "NOP",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "CPX",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_CPX,
        cycles: 4,
    },
    Opcode {
        name: "SBC",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_SBC,
        cycles: 4,
    },
    Opcode {
        name: "INC",
        addr_t: Cpu::addr_ABS,
        operation: Cpu::op_INC,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 6,
    },
    Opcode {
        name: "BEQ",
        addr_t: Cpu::addr_REL,
        operation: Cpu::op_BEQ,
        cycles: 2,
    },
    Opcode {
        name: "SBC",
        addr_t: Cpu::addr_ABX,
        operation: Cpu::op_SBC,
        cycles: 5,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 8,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "SBC",
        addr_t: Cpu::addr_ABY,
        operation: Cpu::op_SBC,
        cycles: 4,
    },
    Opcode {
        name: "INC",
        addr_t: Cpu::addr_ZPX,
        operation: Cpu::op_INC,
        cycles: 6,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 6,
    },
    Opcode {
        name: "SED",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_SED,
        cycles: 2,
    },
    Opcode {
        name: "SBC",
        addr_t: Cpu::addr_IDX,
        operation: Cpu::op_SBC,
        cycles: 4,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 2,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 7,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 4,
    },
    Opcode {
        name: "SBC",
        addr_t: Cpu::addr_IDY,
        operation: Cpu::op_SBC,
        cycles: 4,
    },
    Opcode {
        name: "INC",
        addr_t: Cpu::addr_ABX,
        operation: Cpu::op_INC,
        cycles: 7,
    },
    Opcode {
        name: "NUL",
        addr_t: Cpu::addr_ACC,
        operation: Cpu::op_NOP,
        cycles: 7,
    },
];

pub struct Cpu {
    pub bus: Bus,
    pub pc: u16,
    pub cycles: u8,
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub reg_sp: u8,
    pub flag: cpu_flag::Flag,
    // other
    pub tick_count: u64,
    pub fetched: u8,
    pub abs_addr: u16,
    pub relative_addr_offset: i8,
    pub is_accumulator_opcode: bool,
}

impl Cpu {
    pub fn new() -> Self {
        let flag = cpu_flag::Flag {
            flag_n: 0,
            flag_v: 0,
            flag_b: 0,
            flag_d: 0,
            flag_i: 0,
            flag_z: 0,
            flag_c: 0,
        };

        Self {
            bus: Bus::new(),
            pc: 0x0600,
            cycles: 0,
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            reg_sp: 0xFF,
            flag,
            tick_count: 0,
            fetched: 0,
            abs_addr: 0,
            relative_addr_offset: 0,
            is_accumulator_opcode: false,
        }
    }

    /*
    CPU -> PPU Integration Point, its kinda weird but basically we call this in our main loop and it will tick both the CPU and PPU
    Move to the bus??? but the CPU -> bus releation is whack.
     */
    #[inline(always)]
    pub fn execute_cpu_ppu(&mut self) {
        if self.bus.dma_cycles > 0 {
            self.bus.dma_cycles -= 1;
        } else {
            if self.bus.ppu.get_and_reset_nmi_triggered() {
                self.trigger_nmi(); // applies nmi instantly, but adds the clock cost
            }
            self.tick();
        }

        self.bus.ppu.tick(&self.bus.rom);
        self.bus.ppu.tick(&self.bus.rom);
        self.bus.ppu.tick(&self.bus.rom);
    }

    fn write_value(&mut self, value: u8) {
        if self.is_accumulator_opcode {
            self.reg_a = value;
        } else {
            self.bus.write_ram(self.abs_addr, value);
        }
    }

    fn fetch(&mut self) {
        if !self.is_accumulator_opcode {
            self.fetched = self.bus.read_ram(self.abs_addr);
        }
    }

    pub fn ready_to_execute_next_instruction(&self) -> bool {
        return self.cycles == 0;
    }

    pub fn trigger_nmi(&mut self) {
        self.cycles += self.nmi();
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;

        if self.cycles > 0 {
            self.cycles -= 1;
            return;
        }

        if self.cycles == 0 {
            // TODO when PPU registers are implemented, update the preemptive write register to the actual register here
            // Writes to the PPU registers should occur AFTER cycles have passed for a given instruction (not at the  beginning)

            let current_opcode: u8 = self.bus.read_ram(self.pc);
            self.pc += 1;

            let opcode: &Opcode = &OPCODE_LOOKUP[current_opcode as usize];
            self.is_accumulator_opcode = opcode.addr_t as usize == Cpu::addr_ACC as usize;

            self.cycles += (opcode.addr_t as fn(cpu: &mut Cpu) -> u8)(self);
            self.cycles += (opcode.operation as fn(cpu: &mut Cpu) -> u8)(self);
            self.cycles += opcode.cycles;
            self.cycles -= 1;
        }
    }

    pub fn get_optcode(&self, pc_addr: u16) -> &Opcode<'_> {
        let current_opcode: u8 = self.bus.read_ram_immutable_debug(pc_addr); // TODO do we need to use the mutable read?
        let opcode: &Opcode = &OPCODE_LOOKUP[current_opcode as usize];
        return opcode;
    }

    #[allow(dead_code)]
    fn get_reg_sr(&self) -> u8 {
        return self.flag.get_sr();
    }

    #[allow(dead_code)]
    pub fn run_until_interrupt(&mut self) {
        loop {
            self.tick();
            if self.pc == 0x00 {
                break;
            }
        }
    }

    pub fn nmi(&mut self) -> u8 {
        self.flag.set_flag_i(true);
        self.flag.set_flag_b(false);

        self.push_stack_u16(self.pc); // TODO verify this is right
        self.push_stack_u8(self.flag.get_sr());

        self.pc = self.bus.read_ram(0xFFFA) as u16 | (self.bus.read_ram(0xFFFB) as u16) << 8;
        return 8;
    }

    #[allow(dead_code)]
    pub fn irq(&mut self) -> u8 {
        if self.flag.get_flag_i() {
            return 0;
        }
        // flag.i must be 0

        self.flag.set_flag_i(true);
        self.flag.set_flag_b(false);

        self.push_stack_u16(self.pc); // TODO verify this is right
        self.push_stack_u8(self.flag.get_sr());

        // Same as BRK
        self.pc = self.bus.read_ram(0xFFFE) as u16 | (self.bus.read_ram(0xFFFF) as u16) << 8;
        return 7;
    }

    pub fn reset(&mut self) {
        self.pc = self.bus.read_ram(0xFFFC) as u16 | (self.bus.read_ram(0xFFFD) as u16) << 8;

        self.reg_a = 0;
        self.reg_x = 0;
        self.reg_y = 0;
        self.reg_sp = 0xFD;
        self.flag.set_sr(0);
        self.flag.set_flag_i(true); // Interrupt Disable flag set on reset

        self.abs_addr = 0x0000;
        self.relative_addr_offset = 0x00;
        self.fetched = 0x00;

        self.tick_count = 0;
        self.cycles = 7;
    }

    // ---- Start of Memory Access ---- //

    fn addr_NUL(&mut self) -> u8 {
        self.fetched = self.reg_a; // TODO should we fetch A?
        return 0;
    }

    // GOOD
    fn addr_ACC(&mut self) -> u8 {
        self.fetched = self.reg_a;
        return 0;
    }

    // GOOD
    fn addr_IMM(&mut self) -> u8 {
        self.abs_addr = self.pc;
        self.pc += 1;
        return 0;
    }

    // GOOD
    fn addr_ZPG(&mut self) -> u8 {
        self.abs_addr = self.bus.read_ram(self.pc) as u16;
        self.pc += 1;
        return 0;
    }

    // GOOD
    fn addr_ZPX(&mut self) -> u8 {
        self.abs_addr = self.bus.read_ram(self.pc) as u16 + self.reg_x as u16;
        self.abs_addr &= 0x00FF;
        self.pc += 1;
        return 0;
    }

    // GOOD
    fn addr_ZPY(&mut self) -> u8 {
        self.abs_addr = self.bus.read_ram(self.pc) as u16 + self.reg_y as u16;
        self.abs_addr &= 0x00FF;
        self.pc += 1;
        return 0;
    }

    // GOOD
    fn addr_ABS(&mut self) -> u8 {
        let abs_addr_lo = self.bus.read_ram(self.pc) as u16;
        self.pc += 1;
        let abs_addr_hi = self.bus.read_ram(self.pc) as u16;
        self.pc += 1;
        self.abs_addr = abs_addr_hi << 8 | abs_addr_lo;
        return 0;
    }

    // GOOD
    fn addr_ABX(&mut self) -> u8 {
        let abs_addr_lo = self.bus.read_ram(self.pc) as u16;
        self.pc += 1;
        let abs_addr_hi = self.bus.read_ram(self.pc) as u16;
        self.pc += 1;

        let temp: u32 = (abs_addr_hi << 8 | abs_addr_lo) as u32 + self.reg_x as u32;
        self.abs_addr = (temp & 0xFFFF) as u16; // Assumed this is correct

        // changing page costs extra
        if self.abs_addr & 0xFF00 != abs_addr_hi << 8 {
            return 1;
        }
        return 0;
    }

    // GOOD
    fn addr_ABY(&mut self) -> u8 {
        let abs_addr_lo = self.bus.read_ram(self.pc) as u16;
        self.pc += 1;
        let abs_addr_hi = self.bus.read_ram(self.pc) as u16;
        self.pc += 1;

        let temp: u32 = (abs_addr_hi << 8 | abs_addr_lo) as u32 + self.reg_y as u32;
        self.abs_addr = (temp & 0xFFFF) as u16; // Assumed this is correct

        // changing page costs extra
        if self.abs_addr & 0xFF00 != abs_addr_hi << 8 {
            return 1;
        }
        return 0;
    }

    // MAYBE ? kind of complex
    fn addr_IND(&mut self) -> u8 {
        let ptr_addr_lo = self.bus.read_ram(self.pc) as u16;
        self.pc += 1;
        let ptr_addr_hi = self.bus.read_ram(self.pc) as u16;
        self.pc += 1;

        let ptr: u16 = (ptr_addr_hi << 8 | ptr_addr_lo) as u16;
        let ptr2: u16 = ((ptr & 0xFF00) | ((ptr + 1) & 0x00FF)) as u16; //replicate 6502 page-boundary wraparound bug

        self.abs_addr = (self.bus.read_ram(ptr2) as u16) << 8 | self.bus.read_ram(ptr) as u16;

        return 0;
    }

    // GOOD
    fn addr_IDX(&mut self) -> u8 {
        let ptr_addr = (self.bus.read_ram(self.pc) as u16 + self.reg_x as u16) & 0xFF;
        self.pc += 1;

        let abs_addr_lo = self.bus.read_ram(ptr_addr) as u16;
        let abs_addr_hi = self.bus.read_ram(ptr_addr + 1) as u16;

        self.abs_addr = abs_addr_hi << 8 | abs_addr_lo;

        return 0;
    }

    // GOOD
    fn addr_IDY(&mut self) -> u8 {
        let ptr_addr = self.bus.read_ram(self.pc) as u16;
        self.pc += 1;

        let abs_addr_lo = self.bus.read_ram(ptr_addr) as u16;
        let abs_addr_hi = self.bus.read_ram(ptr_addr + 1) as u16;

        self.abs_addr = (abs_addr_hi << 8 | abs_addr_lo) + self.reg_y as u16;

        if self.abs_addr & 0xFF00 != abs_addr_hi << 8 {
            return 1;
        }
        return 0;
    }

    fn addr_REL(&mut self) -> u8 {
        self.relative_addr_offset = self.bus.read_ram(self.pc) as i8;
        self.pc += 1;
        return 0;
    }

    fn push_stack_u16(&mut self, value: u16) {
        self.bus
            .write_ram(0x100 + self.reg_sp as u16, ((value >> 8) & 0xFF) as u8);
        self.reg_sp = decrement_u8(self.reg_sp);

        self.bus
            .write_ram(0x100 + self.reg_sp as u16, (value & 0xFF) as u8);
        self.reg_sp = decrement_u8(self.reg_sp);
    }

    fn push_stack_u8(&mut self, value: u8) {
        self.bus.write_ram(0x100 + self.reg_sp as u16, value);
        self.reg_sp = decrement_u8(self.reg_sp);
    }

    fn pull_stack_u8(&mut self) -> u8 {
        self.reg_sp = increment_u8(self.reg_sp);
        return self.bus.read_ram(0x100 + self.reg_sp as u16);
    }

    fn pull_stack_u16(&mut self) -> u16 {
        self.reg_sp = increment_u8(self.reg_sp);
        let val_lo: u16 = self.bus.read_ram(0x100 + self.reg_sp as u16) as u16;
        self.reg_sp = increment_u8(self.reg_sp);
        let val_hi: u16 = self.bus.read_ram(0x100 + self.reg_sp as u16) as u16;

        return val_lo | (val_hi << 8);
    }

    fn set_z_n_flags(&mut self, val: u8) {
        self.flag.set_flag_z(val == 0);
        self.flag.set_flag_n(val & 0x80 != 0);
    }

    // TODO verify this logic is correct
    fn set_overflow_flag(&mut self, result: u16, acc: u8, mem: u8) {
        self.flag
            .set_flag_v((result ^ acc as u16) & (result ^ mem as u16) & 0x0080 != 0);
    }

    // ---- Start of Opcodes ---- //

    fn op_NOP(&mut self) -> u8 {
        return 0;
    }

    fn op_ADC(&mut self) -> u8 {
        self.fetch();

        let sum_u16: u16 = self.reg_a as u16 + self.fetched as u16 + self.flag.flag_c as u16;
        let sum_u8: u8 = (sum_u16 & 0xff) as u8;

        self.flag.set_flag_c(sum_u16 > 0xff);
        self.set_overflow_flag(sum_u16, self.reg_a, self.fetched);
        self.set_z_n_flags(sum_u8);

        self.reg_a = sum_u8;
        //println!("ADC = {:#x}", self.reg_a);
        return 0;
    }

    fn op_AND(&mut self) -> u8 {
        self.fetch();

        self.reg_a &= self.fetched;

        self.set_z_n_flags(self.reg_a);
        return 0;
    }

    fn op_BIT(&mut self) -> u8 {
        self.fetch();

        self.flag.set_flag_z(self.reg_a & self.fetched == 0x00);
        self.flag.set_flag_v(self.fetched & (1 << 6) != 0);
        self.flag.set_flag_n(self.fetched & (1 << 7) != 0);
        return 0;
    }

    fn op_LDA(&mut self) -> u8 {
        self.fetch();

        self.reg_a = self.fetched;

        self.set_z_n_flags(self.reg_a);
        return 0;
    }

    fn op_LDX(&mut self) -> u8 {
        self.fetch();

        self.reg_x = self.fetched;

        self.set_z_n_flags(self.reg_x);

        return 0;
    }

    fn op_LDY(&mut self) -> u8 {
        self.fetch();

        self.reg_y = self.fetched;

        self.set_z_n_flags(self.reg_y);

        return 0;
    }

    fn op_STA(&mut self) -> u8 {
        self.bus.write_ram(self.abs_addr, self.reg_a);
        return 0;
    }

    fn op_STX(&mut self) -> u8 {
        self.bus.write_ram(self.abs_addr, self.reg_x);
        return 0;
    }

    fn op_STY(&mut self) -> u8 {
        self.bus.write_ram(self.abs_addr, self.reg_y);
        return 0;
    }

    fn op_TAX(&mut self) -> u8 {
        self.reg_x = self.reg_a;
        self.set_z_n_flags(self.reg_x);
        return 0;
    }

    fn op_TAY(&mut self) -> u8 {
        self.reg_y = self.reg_a;
        self.set_z_n_flags(self.reg_y);
        return 0;
    }

    fn op_TSX(&mut self) -> u8 {
        self.reg_x = self.reg_sp;
        self.set_z_n_flags(self.reg_x);
        return 0;
    }

    fn op_TXA(&mut self) -> u8 {
        self.reg_a = self.reg_x;
        self.set_z_n_flags(self.reg_a);
        return 0;
    }

    fn op_TXS(&mut self) -> u8 {
        self.reg_sp = self.reg_x;
        return 0;
    }

    fn op_TYA(&mut self) -> u8 {
        self.reg_a = self.reg_y;
        self.set_z_n_flags(self.reg_a);
        return 0;
    }

    fn op_CLC(&mut self) -> u8 {
        self.flag.set_flag_c(false);
        return 0;
    }

    fn op_CLD(&mut self) -> u8 {
        self.flag.set_flag_d(false);
        return 0;
    }

    fn op_CLI(&mut self) -> u8 {
        self.flag.set_flag_i(false);
        return 0;
    }

    fn op_CLV(&mut self) -> u8 {
        self.flag.set_flag_v(false);
        return 0;
    }

    fn op_DEC(&mut self) -> u8 {
        self.fetch();

        let temp: u8 = decrement_u8(self.fetched);
        self.bus.write_ram(self.abs_addr, temp);
        self.set_z_n_flags(temp);
        return 0;
    }

    fn op_DEX(&mut self) -> u8 {
        self.reg_x = decrement_u8(self.reg_x);
        self.set_z_n_flags(self.reg_x);
        return 0;
    }

    fn op_DEY(&mut self) -> u8 {
        self.reg_y = decrement_u8(self.reg_y);
        self.set_z_n_flags(self.reg_y);
        return 0;
    }

    fn op_EOR(&mut self) -> u8 {
        self.fetch();
        self.reg_a ^= self.fetched;

        self.set_z_n_flags(self.reg_a);
        return 0;
    }

    fn op_INC(&mut self) -> u8 {
        self.fetch();

        let temp: u8 = increment_u8(self.fetched);
        self.bus.write_ram(self.abs_addr, temp);
        self.set_z_n_flags(temp);
        return 0;
    }

    fn op_INX(&mut self) -> u8 {
        self.reg_x = increment_u8(self.reg_x);
        self.set_z_n_flags(self.reg_x);
        return 0;
    }

    fn op_INY(&mut self) -> u8 {
        self.reg_y = increment_u8(self.reg_y);
        self.set_z_n_flags(self.reg_y);
        return 0;
    }

    // Shared function for jumps
    fn op_jump(&mut self, do_jump_condition: bool) -> u8 {
        let mut cycle_cost: u8 = 0;

        if do_jump_condition {
            cycle_cost += 1;

            let updated_pc = (self.pc as i32 + self.relative_addr_offset as i32) as u16;

            if updated_pc & 0xFF00 != self.pc & 0xFF00 {
                cycle_cost += 1;
            }
            self.pc = updated_pc;
        }
        return cycle_cost;
    }

    // carry flag
    fn op_BCS(&mut self) -> u8 {
        return self.op_jump(self.flag.get_flag_c());
    }

    fn op_BCC(&mut self) -> u8 {
        return self.op_jump(!self.flag.get_flag_c());
    }

    // zero
    fn op_BEQ(&mut self) -> u8 {
        return self.op_jump(self.flag.get_flag_z());
    }
    fn op_BNE(&mut self) -> u8 {
        return self.op_jump(!self.flag.get_flag_z());
    }

    // negative
    fn op_BMI(&mut self) -> u8 {
        return self.op_jump(self.flag.get_flag_n());
    }
    fn op_BPL(&mut self) -> u8 {
        return self.op_jump(!self.flag.get_flag_n());
    }

    // overflow
    fn op_BVS(&mut self) -> u8 {
        return self.op_jump(self.flag.get_flag_v());
    }
    fn op_BVC(&mut self) -> u8 {
        return self.op_jump(!self.flag.get_flag_v());
    }

    fn op_JMP(&mut self) -> u8 {
        self.pc = self.abs_addr;
        return 0;
    }

    fn op_JSR(&mut self) -> u8 {
        self.push_stack_u16(self.pc - 1);
        self.pc = self.abs_addr;
        return 0;
    }

    fn op_LSR(&mut self) -> u8 {
        self.fetch();

        self.flag.set_flag_c(self.fetched & 0x01 != 0);

        let value = self.fetched >> 1;

        self.set_z_n_flags(value);
        self.write_value(value);
        return 0;
    }

    fn op_ORA(&mut self) -> u8 {
        self.fetch();
        self.reg_a |= self.fetched;
        self.set_z_n_flags(self.reg_a);
        return 0;
    }

    fn op_PHA(&mut self) -> u8 {
        self.push_stack_u8(self.reg_a);
        return 0;
    }

    fn op_PHP(&mut self) -> u8 {
        self.push_stack_u8(self.flag.get_sr() | 0x10); // FLAG BREAK
        return 0;
    }

    fn op_PLA(&mut self) -> u8 {
        self.reg_a = self.pull_stack_u8();
        self.set_z_n_flags(self.reg_a);
        return 0;
    }

    fn op_PLP(&mut self) -> u8 {
        let sr: u8 = self.pull_stack_u8();
        self.flag.set_sr(sr);
        return 0;
    }

    fn op_ROL(&mut self) -> u8 {
        self.fetch();
        let value: u8 = (self.fetched << 1) | self.flag.flag_c;

        self.flag.set_flag_c(self.fetched & 0x80 != 0);
        self.set_z_n_flags(value);

        self.write_value(value);

        return 0;
    }

    fn op_ROR(&mut self) -> u8 {
        self.fetch();
        let value: u8 = (self.flag.flag_c << 7) | (self.fetched >> 1);

        self.flag.set_flag_c(self.fetched & 0x01 != 0);
        self.set_z_n_flags(value);

        self.write_value(value);

        return 0;
    }

    fn op_ASL(&mut self) -> u8 {
        self.fetch();
        let value: u8 = self.fetched << 1;

        self.flag.set_flag_c(self.fetched & 0x80 != 0);
        self.set_z_n_flags(value);

        self.write_value(value);

        return 0;
    }

    fn op_RTI(&mut self) -> u8 {
        let sr: u8 = self.pull_stack_u8();
        let pc: u16 = self.pull_stack_u16();
        self.flag.set_sr(sr);
        self.pc = pc;
        return 0;
    }

    fn op_RTS(&mut self) -> u8 {
        let pc: u16 = self.pull_stack_u16();
        self.pc = pc + 1;
        return 0;
    }

    fn op_SEC(&mut self) -> u8 {
        self.flag.set_flag_c(true);
        return 0;
    }

    fn op_SED(&mut self) -> u8 {
        self.flag.set_flag_d(true);
        return 0;
    }

    fn op_SEI(&mut self) -> u8 {
        self.flag.set_flag_i(true);
        return 0;
    }

    fn op_CMP(&mut self) -> u8 {
        self.fetch();

        let result: u8 = if self.reg_a < self.fetched {
            0xFF - (self.fetched - self.reg_a)
        } else {
            self.reg_a - self.fetched
        };

        self.flag.set_flag_c(self.reg_a >= self.fetched);
        self.flag.set_flag_z(self.reg_a == self.fetched);
        self.flag.set_flag_n(result & 0x80 != 0);
        return 0;
    }

    fn op_CPX(&mut self) -> u8 {
        self.fetch();

        let result: u8 = if self.reg_x < self.fetched {
            0xFF - (self.fetched - self.reg_x)
        } else {
            self.reg_x - self.fetched
        };

        self.flag.set_flag_c(self.reg_x >= self.fetched);
        self.flag.set_flag_z(self.reg_x == self.fetched);
        self.flag.set_flag_n(result & 0x80 != 0);
        return 0;
    }

    fn op_CPY(&mut self) -> u8 {
        self.fetch();

        let result: u8 = if self.reg_y < self.fetched {
            0xFF - (self.fetched - self.reg_y)
        } else {
            self.reg_y - self.fetched
        };

        self.flag.set_flag_c(self.reg_y >= self.fetched);
        self.flag.set_flag_z(self.reg_y == self.fetched);
        self.flag.set_flag_n(result & 0x80 != 0);
        return 0;
    }

    fn op_SBC(&mut self) -> u8 {
        self.fetch();

        let value: u8 = self.fetched ^ 0xFF;
        let sum_u16: u16 = self.reg_a as u16 + value as u16 + self.flag.flag_c as u16;
        let sum_u8: u8 = (sum_u16 & 0xFF) as u8;

        self.flag.set_flag_c(sum_u16 > 0xff);
        self.set_overflow_flag(sum_u16, self.reg_a, value);
        self.set_z_n_flags(sum_u8);

        self.reg_a = sum_u8;
        return 0;
    }

    fn op_BRK(&mut self) -> u8 {
        self.flag.set_flag_i(true);

        self.push_stack_u16(self.pc + 1); // TODO verify this is right
        self.push_stack_u8(self.flag.get_sr() | 0x10); // FLAG BREAK

        self.pc = self.bus.read_ram(0xFFFE) as u16 | (self.bus.read_ram(0xFFFF) as u16) << 8;
        return 0;
    }

    // ---- End of Opcodes ---- //
}

pub struct Opcode<'a> {
    name: &'a str,
    addr_t: fn(cpu: &mut Cpu) -> u8,
    operation: fn(cpu: &mut Cpu) -> u8,
    cycles: u8,
}

impl Opcode<'_> {
    fn map_known_address_labels(addr: u16) -> Option<String> {
        match addr {
            // PPU Registers
            0x2000 => Some("PPUCTRL".to_string()),
            0x2001 => Some("PPUMASK".to_string()),
            0x2002 => Some("PPUSTATUS".to_string()),
            0x2003 => Some("OAMADDR".to_string()),
            0x2004 => Some("OAMDATA".to_string()),
            0x2005 => Some("PPUSCROLL".to_string()),
            0x2006 => Some("PPUADDR".to_string()),
            0x2007 => Some("PPUDATA".to_string()),
            // APU and I/O
            0x4014 => Some("OAMDMA".to_string()),
            0x4015 => Some("APU_CTRL".to_string()),
            0x4016 => Some("JOY1".to_string()),
            0x4017 => Some("JOY2".to_string()),
            // APU Square 1
            0x4000 => Some("SQ1_VOL".to_string()),
            0x4001 => Some("SQ1_SWEEP".to_string()),
            0x4002 => Some("SQ1_LO".to_string()),
            0x4003 => Some("SQ1_HI".to_string()),
            // APU Square 2
            0x4004 => Some("SQ2_VOL".to_string()),
            0x4005 => Some("SQ2_SWEEP".to_string()),
            0x4006 => Some("SQ2_LO".to_string()),
            0x4007 => Some("SQ2_HI".to_string()),
            // APU Triangle
            0x4008 => Some("TRI_LINEAR".to_string()),
            0x400A => Some("TRI_LO".to_string()),
            0x400B => Some("TRI_HI".to_string()),
            // APU Noise
            0x400C => Some("NOISE_VOL".to_string()),
            0x400E => Some("NOISE_LO".to_string()),
            0x400F => Some("NOISE_HI".to_string()),
            // APU DMC
            0x4010 => Some("DMC_FREQ".to_string()),
            0x4011 => Some("DMC_RAW".to_string()),
            0x4012 => Some("DMC_START".to_string()),
            0x4013 => Some("DMC_LEN".to_string()),

            // Interrupt Vectors
            0xFFFA => Some("NMI_VEC_LO".to_string()),
            0xFFFB => Some("NMI_VEC_HI".to_string()),
            0xFFFC => Some("RESET_VEC_LO".to_string()),
            0xFFFD => Some("RESET_VEC_HI".to_string()),
            0xFFFE => Some("IRQ_VEC_LO".to_string()),
            0xFFFF => Some("IRQ_VEC_HI".to_string()),
            _ => None,
        }
    }

    // This attempts to handle both u16 memory refs and u8 relative refs
    // WARNING: Not complete.
    pub fn get_memory_addr_accessed(&self, cpu: &Cpu, pc_value: u16) -> Option<u16> {
        if self.get_opcode_byte_size() == 3 {
            Some(
                (cpu.bus.read_ram_immutable_debug(pc_value + 1) as u16)
                    | ((cpu.bus.read_ram_immutable_debug(pc_value + 2) as u16) << 8),
            )
        } else if self.get_opcode_byte_size() == 2 {
            if self.addr_t as usize == Cpu::addr_REL as usize {
                Some(
                    ((pc_value as i32)
                        + (cpu.bus.read_ram_immutable_debug(pc_value + 1) as i8) as i32
                        + 2) as u16,
                )
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn map_addr_labels(&self, debugger: &Debugger, addr_u16: Option<u16>) -> String {
        let addr_mapped_str = if let Some(addr) = addr_u16 {
            Self::map_known_address_labels(addr)
                .or_else(|| {
                    debugger
                        .check_symbol_at_memory_access(addr)
                        .then(|| debugger.get_symbol_at_memory_access(addr))
                })
                .unwrap_or_else(|| format!("{:04X}", addr))
        } else {
            String::new()
        };
        return addr_mapped_str;
    }

    pub fn get_instruction_decoded(&self, cpu: &Cpu, debugger: &Debugger, pc_value: u16) -> String {
        let mut addr_u8: u8 = 0xDD; //TODO fix use of a dead addr
        if self.get_opcode_byte_size() == 2 {
            addr_u8 = cpu.bus.read_ram_immutable_debug(pc_value + 1);
        }

        let addr_u16: Option<u16> = self.get_memory_addr_accessed(cpu, pc_value);
        let addr_u16_mapped_str = self.map_addr_labels(debugger, addr_u16);

        if self.addr_t as usize == Cpu::addr_NUL as usize {
            return format!("{:04X}: {}", pc_value, self.name);
        } else if self.addr_t as usize == Cpu::addr_ACC as usize {
            return format!("{:04X}: {}", pc_value, self.name);
        } else if self.addr_t as usize == Cpu::addr_IMM as usize {
            return format!(
                "{:04X}: {} #${:02X} (#{:02})",
                pc_value, self.name, addr_u8, addr_u8
            );
        } else if self.addr_t as usize == Cpu::addr_ZPG as usize {
            return format!("{:04X}: {} ${:02X}", pc_value, self.name, addr_u8);
        } else if self.addr_t as usize == Cpu::addr_ZPX as usize {
            return format!("{:04X}: {} ${:02X},X", pc_value, self.name, addr_u8);
        } else if self.addr_t as usize == Cpu::addr_ZPY as usize {
            return format!("{:04X}: {} ${:02X},Y", pc_value, self.name, addr_u8);
        } else if self.addr_t as usize == Cpu::addr_ABS as usize {
            // TODO should fix to use addr_u16_mapped_str = get_memory_addr_accessed()
            return format!(
                "{:04X}: {} ${}",
                pc_value,
                self.name,
                format!("{:04X}", addr_u16.unwrap())
            );
        } else if self.addr_t as usize == Cpu::addr_ABX as usize {
            return format!("{:04X}: {} ${},X", pc_value, self.name, addr_u16_mapped_str);
        } else if self.addr_t as usize == Cpu::addr_ABY as usize {
            return format!("{:04X}: {} ${},Y", pc_value, self.name, addr_u16_mapped_str);
        } else if self.addr_t as usize == Cpu::addr_IND as usize {
            return format!("{:04X}: {} (${})", pc_value, self.name, addr_u16_mapped_str);
        } else if self.addr_t as usize == Cpu::addr_IDX as usize {
            return format!("{:04X}: {} (${:02X}, X)", pc_value, self.name, addr_u8);
        } else if self.addr_t as usize == Cpu::addr_IDY as usize {
            return format!("{:04X}: {} (${:02X}), Y", pc_value, self.name, addr_u8);
        } else if self.addr_t as usize == Cpu::addr_REL as usize {
            return format!("{:04X}: {} (${})", pc_value, self.name, addr_u16_mapped_str);
        }
        return String::from("???");
    }

    pub fn get_opcode_byte_size(&self) -> u16 {
        let mut byte_count: u16 = 0;
        if self.addr_t as usize == Cpu::addr_NUL as usize {
            byte_count = 1;
        } else if self.addr_t as usize == Cpu::addr_ACC as usize {
            byte_count = 1;
        } else if self.addr_t as usize == Cpu::addr_IMM as usize {
            byte_count = 2;
        } else if self.addr_t as usize == Cpu::addr_ZPG as usize {
            byte_count = 2;
        } else if self.addr_t as usize == Cpu::addr_ZPX as usize {
            byte_count = 2;
        } else if self.addr_t as usize == Cpu::addr_ZPY as usize {
            byte_count = 2;
        } else if self.addr_t as usize == Cpu::addr_ABS as usize {
            byte_count = 3;
        } else if self.addr_t as usize == Cpu::addr_ABX as usize {
            byte_count = 3;
        } else if self.addr_t as usize == Cpu::addr_ABY as usize {
            byte_count = 3;
        } else if self.addr_t as usize == Cpu::addr_IND as usize {
            byte_count = 3;
        } else if self.addr_t as usize == Cpu::addr_IDX as usize {
            byte_count = 2;
        } else if self.addr_t as usize == Cpu::addr_IDY as usize {
            byte_count = 2;
        } else if self.addr_t as usize == Cpu::addr_REL as usize {
            byte_count = 2;
        }

        assert!(byte_count != 0);

        return byte_count;
    }

    pub fn is_rts(&self) -> bool {
        self.name == "RTS"
    }
}
