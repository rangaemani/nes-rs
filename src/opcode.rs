use crate::cpu::AddressingMode;
use std::collections::HashMap;

/// Represents opcodes present for the NES 2A03 CPU.
///
/// Each opcode has an associated opcode value, abbreviation, length, cycle count, and addressing mode.
pub struct OpCode {
    /// The opcode value (in hex).
    pub opcode: u8,
    /// The abbreviation of the opcode.
    pub abbreviation: &'static str,
    /// The length of the instruction in bytes.
    pub length: u8,
    /// The number of cycles the instruction takes to execute.
    pub cycles: u8,
    /// The addressing mode used by the instruction.
    pub mode: AddressingMode,
}

impl OpCode {
    /// Creates a new `OpCode`.
    ///
    /// # Arguments
    ///
    /// * `opcode` - The opcode value.
    /// * `abbreviation` - The abbreviation of the opcode.
    /// * `length` - The length of the instruction in bytes.
    /// * `cycles` - The number of cycles the instruction takes to execute.
    /// * `mode` - The addressing mode used by the instruction.
    ///
    /// # Returns
    ///
    /// A new `OpCode` instance.
    fn new(opcode: u8, abbreviation: &'static str, length: u8, cycles: u8, mode: AddressingMode) -> Self {
        OpCode { opcode, abbreviation, length, cycles, mode }
    }
}

lazy_static! {
    /// reference vector for all NES opcodes
    pub static ref CPU_OP_CODES: Vec<OpCode> = vec![
       /////////////////////////////////////SPECIAL
        //// BREAK
        OpCode::new(0x00, "BRK",  1,  7, AddressingMode::NoneAddressing),
        //// NOP
        OpCode::new(0xea, "NOP", 1, 2, AddressingMode::NoneAddressing),
       /////////////////////////////////////ARITHMETIC
        //// ADC
        OpCode::new(0x69, "ADC", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x65, "ADC", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x75, "ADC", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x6d, "ADC", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x7d, "ADC", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0x79, "ADC", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0x61, "ADC", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x71, "ADC", 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),
        //// SBC
        OpCode::new(0xe9, "SBC", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xe5, "SBC", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xf5, "SBC", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xed, "SBC", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xfd, "SBC", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0xf9, "SBC", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0xe1, "SBC", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xf1, "SBC", 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),
        //// AND
        OpCode::new(0x29, "AND", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x25, "AND", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x35, "AND", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x2d, "AND", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x3d, "AND", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0x39, "AND", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0x21, "AND", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x31, "AND", 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),
        //// EOR
        OpCode::new(0x49, "EOR", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x45, "EOR", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x55, "EOR", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x4d, "EOR", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x5d, "EOR", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0x59, "EOR", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0x41, "EOR", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x51, "EOR", 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),
        //// ORA
        OpCode::new(0x09, "ORA", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x05, "ORA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x15, "ORA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x0d, "ORA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x1d, "ORA", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0x19, "ORA", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0x01, "ORA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x11, "ORA", 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),
       /////////////////////////////////////SHIFT/UNSHIFT
        //// ASL
        OpCode::new(0x0a, "ASL", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x06, "ASL", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x16, "ASL", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x0e, "ASL", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x1e, "ASL", 3, 7, AddressingMode::Absolute_X),
        //// LSR
        OpCode::new(0x4a, "LSR", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x46, "LSR", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x56, "LSR", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x4e, "LSR", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x5e, "LSR", 3, 7, AddressingMode::Absolute_X),
        //// ROL
        OpCode::new(0x2a, "ROL", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x26, "ROL", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x36, "ROL", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x2e, "ROL", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x3e, "ROL", 3, 7, AddressingMode::Absolute_X),
        //// ROR
        OpCode::new(0x6a, "ROR", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x66, "ROR", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x76, "ROR", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x6e, "ROR", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x7e, "ROR", 3, 7, AddressingMode::Absolute_X),
        //// INC
        OpCode::new(0xe6, "INC", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xf6, "INC", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xee, "INC", 3, 6, AddressingMode::Absolute),
        OpCode::new(0xfe, "INC", 3, 7, AddressingMode::Absolute_X),
        //// INX
        OpCode::new(0xe8, "INX", 1, 2, AddressingMode::NoneAddressing),
        //// INY
        OpCode::new(0xc8, "INY", 1, 2, AddressingMode::NoneAddressing),
        //// DEC
        OpCode::new(0xc6, "DEC", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xd6, "DEC", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xce, "DEC", 3, 6, AddressingMode::Absolute),
        OpCode::new(0xde, "DEC", 3, 7, AddressingMode::Absolute_X),
        //// DEX
        OpCode::new(0xca, "DEX", 1, 2, AddressingMode::NoneAddressing),
        //// DEY
        OpCode::new(0x88, "DEY", 1, 2, AddressingMode::NoneAddressing),
        //// CMP
        OpCode::new(0xc9, "CMP", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xc5, "CMP", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xd5, "CMP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xcd, "CMP", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xdd, "CMP", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0xd9, "CMP", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0xc1, "CMP", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xd1, "CMP", 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),
        //// CPY
        OpCode::new(0xc0, "CPY", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xc4, "CPY", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xcc, "CPY", 3, 4, AddressingMode::Absolute),
        //// CPX
        OpCode::new(0xe0, "CPX", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xe4, "CPX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xec, "CPX", 3, 4, AddressingMode::Absolute),
       /////////////////////////////////////FLOW CONTROL
        ////JMP
        OpCode::new(0x4c, "JMP", 3, 3, AddressingMode::NoneAddressing), //AddressingMode that acts as Immediate
        OpCode::new(0x6c, "JMP", 3, 5, AddressingMode::NoneAddressing), //AddressingMode:Indirect with 6502 bug
        //// JSR
        OpCode::new(0x20, "JSR", 3, 6, AddressingMode::NoneAddressing),
        //// RTS
        OpCode::new(0x60, "RTS", 1, 6, AddressingMode::NoneAddressing),
        //// RTI
        OpCode::new(0x40, "RTI", 1, 6, AddressingMode::NoneAddressing),
        //// BRANCH
        OpCode::new(0xd0, "BNE", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),
        OpCode::new(0x70, "BVS", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),
        OpCode::new(0x50, "BVC", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),
        OpCode::new(0x30, "BMI", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),
        OpCode::new(0xf0, "BEQ", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),
        OpCode::new(0xb0, "BCS", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),
        OpCode::new(0x90, "BCC", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),
        OpCode::new(0x10, "BPL", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),
        //// BIT
        OpCode::new(0x24, "BIT", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x2c, "BIT", 3, 4, AddressingMode::Absolute),
       /////////////////////////////////////LOADERS/UNLOADERS
        //// LDA
        OpCode::new(0xa9, "LDA",  2,  2, AddressingMode::Immediate),
        OpCode::new(0xa5, "LDA",  2,  3, AddressingMode::ZeroPage),
        OpCode::new(0xb5, "LDA",  2,  4, AddressingMode::ZeroPage_X),
        OpCode::new(0xad, "LDA",  3,  4, AddressingMode::Absolute),
        OpCode::new(0xbd, "LDA",  3,  4 /*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0xb9, "LDA",  3,  4 /*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0xa1, "LDA",  2,  6, AddressingMode::Indirect_X),
        OpCode::new(0xb1, "LDA",  2,  5 /*+1 if page crossed*/, AddressingMode::Indirect_Y),
        //// LDX
        OpCode::new(0xa2, "LDX", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa6, "LDX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb6, "LDX", 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0xae, "LDX", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbe, "LDX", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        //// LDY
        OpCode::new(0xa0, "LDY", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa4, "LDY", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb4, "LDY", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xac, "LDY", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbc, "LDY", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        //// STA
        OpCode::new(0x85, "STA",  2,  3, AddressingMode::ZeroPage),
        OpCode::new(0x95, "STA",  2,  4, AddressingMode::ZeroPage_X),
        OpCode::new(0x8d, "STA",  3,  4, AddressingMode::Absolute),
        OpCode::new(0x9d, "STA",  3,  5, AddressingMode::Absolute_X),
        OpCode::new(0x99, "STA",  3,  5, AddressingMode::Absolute_Y),
        OpCode::new(0x81, "STA",  2,  6, AddressingMode::Indirect_X),
        OpCode::new(0x91, "STA",  2,  6, AddressingMode::Indirect_Y),
        //// STX
        OpCode::new(0x86, "STX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x96, "STX", 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0x8e, "STX", 3, 4, AddressingMode::Absolute),
        //// STY
        OpCode::new(0x84, "STY", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x94, "STY", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x8c, "STY", 3, 4, AddressingMode::Absolute),
       /////////////////////////////////////FLAG SET/CLEAR
        OpCode::new(0xD8, "CLD", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x58, "CLI", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xb8, "CLV", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x18, "CLC", 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x38, "SEC", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x78, "SEI", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xf8, "SED", 1, 2, AddressingMode::NoneAddressing),
        //// TRANSFER
        OpCode::new(0xaa, "TAX", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xa8, "TAY", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xba, "TSX", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x8a, "TXA", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x9a, "TXS", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x98, "TYA", 1, 2, AddressingMode::NoneAddressing),
       /////////////////////////////////////STACK PUSHPOP
        OpCode::new(0x48, "PHA", 1, 3, AddressingMode::NoneAddressing),
        OpCode::new(0x68, "PLA", 1, 4, AddressingMode::NoneAddressing),
        OpCode::new(0x08, "PHP", 1, 3, AddressingMode::NoneAddressing),
        OpCode::new(0x28, "PLP", 1, 4, AddressingMode::NoneAddressing),
        
        ////////////////////////////////////UNOFFICIAL OPCODES // WARNING: MAY BE UNSTABLE

        OpCode::new(0xc7, "*DCP", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xd7, "*DCP", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xCF, "*DCP", 3, 6, AddressingMode::Absolute),
        OpCode::new(0xdF, "*DCP", 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0xdb, "*DCP", 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0xd3, "*DCP", 2, 8, AddressingMode::Indirect_Y),
        OpCode::new(0xc3, "*DCP", 2, 8, AddressingMode::Indirect_X),


        OpCode::new(0x27, "*RLA", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x37, "*RLA", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x2F, "*RLA", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x3F, "*RLA", 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x3b, "*RLA", 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0x33, "*RLA", 2, 8, AddressingMode::Indirect_Y),
        OpCode::new(0x23, "*RLA", 2, 8, AddressingMode::Indirect_X),

        OpCode::new(0x07, "*SLO", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x17, "*SLO", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x0F, "*SLO", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x1f, "*SLO", 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x1b, "*SLO", 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0x03, "*SLO", 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0x13, "*SLO", 2, 8, AddressingMode::Indirect_Y),

        OpCode::new(0x47, "*SRE", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x57, "*SRE", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x4F, "*SRE", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x5f, "*SRE", 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x5b, "*SRE", 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0x43, "*SRE", 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0x53, "*SRE", 2, 8, AddressingMode::Indirect_Y),


        OpCode::new(0x80, "*NOP", 2,2, AddressingMode::Immediate),
        OpCode::new(0x82, "*NOP", 2,2, AddressingMode::Immediate),
        OpCode::new(0x89, "*NOP", 2,2, AddressingMode::Immediate),
        OpCode::new(0xc2, "*NOP", 2,2, AddressingMode::Immediate),
        OpCode::new(0xe2, "*NOP", 2,2, AddressingMode::Immediate),


        OpCode::new(0xCB, "*AXS", 2,2, AddressingMode::Immediate),

        OpCode::new(0x6B, "*ARR", 2,2, AddressingMode::Immediate),

        OpCode::new(0xeb, "*SBC", 2,2, AddressingMode::Immediate),

        OpCode::new(0x0b, "*ANC", 2,2, AddressingMode::Immediate),
        OpCode::new(0x2b, "*ANC", 2,2, AddressingMode::Immediate),

        OpCode::new(0x4b, "*ALR", 2,2, AddressingMode::Immediate),
        // OpCode::new(0xCB, "IGN", 3,4 /* or 5*/, AddressingMode::Absolute_X),

        OpCode::new(0x04, "*NOP", 2,3, AddressingMode::ZeroPage),
        OpCode::new(0x44, "*NOP", 2,3, AddressingMode::ZeroPage),
        OpCode::new(0x64, "*NOP", 2,3, AddressingMode::ZeroPage),
        OpCode::new(0x14, "*NOP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x34, "*NOP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x54, "*NOP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x74, "*NOP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xd4, "*NOP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xf4, "*NOP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x0c, "*NOP", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x1c, "*NOP", 3, 4 /*or 5*/, AddressingMode::Absolute_X),
        OpCode::new(0x3c, "*NOP", 3, 4 /*or 5*/, AddressingMode::Absolute_X),
        OpCode::new(0x5c, "*NOP", 3, 4 /*or 5*/, AddressingMode::Absolute_X),
        OpCode::new(0x7c, "*NOP", 3, 4 /*or 5*/, AddressingMode::Absolute_X),
        OpCode::new(0xdc, "*NOP", 3, 4 /* or 5*/, AddressingMode::Absolute_X),
        OpCode::new(0xfc, "*NOP", 3, 4 /* or 5*/, AddressingMode::Absolute_X),

        OpCode::new(0x67, "*RRA", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x77, "*RRA", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x6f, "*RRA", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x7f, "*RRA", 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x7b, "*RRA", 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0x63, "*RRA", 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0x73, "*RRA", 2, 8, AddressingMode::Indirect_Y),


        OpCode::new(0xe7, "*ISB", 2,5, AddressingMode::ZeroPage),
        OpCode::new(0xf7, "*ISB", 2,6, AddressingMode::ZeroPage_X),
        OpCode::new(0xef, "*ISB", 3,6, AddressingMode::Absolute),
        OpCode::new(0xff, "*ISB", 3,7, AddressingMode::Absolute_X),
        OpCode::new(0xfb, "*ISB", 3,7, AddressingMode::Absolute_Y),
        OpCode::new(0xe3, "*ISB", 2,8, AddressingMode::Indirect_X),
        OpCode::new(0xf3, "*ISB", 2,8, AddressingMode::Indirect_Y),

        OpCode::new(0x02, "*NOP", 1,2, AddressingMode::NoneAddressing),
        OpCode::new(0x12, "*NOP", 1,2, AddressingMode::NoneAddressing),
        OpCode::new(0x22, "*NOP", 1,2, AddressingMode::NoneAddressing),
        OpCode::new(0x32, "*NOP", 1,2, AddressingMode::NoneAddressing),
        OpCode::new(0x42, "*NOP", 1,2, AddressingMode::NoneAddressing),
        OpCode::new(0x52, "*NOP", 1,2, AddressingMode::NoneAddressing),
        OpCode::new(0x62, "*NOP", 1,2, AddressingMode::NoneAddressing),
        OpCode::new(0x72, "*NOP", 1,2, AddressingMode::NoneAddressing),
        OpCode::new(0x92, "*NOP", 1,2, AddressingMode::NoneAddressing),
        OpCode::new(0xb2, "*NOP", 1,2, AddressingMode::NoneAddressing),
        OpCode::new(0xd2, "*NOP", 1,2, AddressingMode::NoneAddressing),
        OpCode::new(0xf2, "*NOP", 1,2, AddressingMode::NoneAddressing),

        OpCode::new(0x1a, "*NOP", 1,2, AddressingMode::NoneAddressing),
        OpCode::new(0x3a, "*NOP", 1,2, AddressingMode::NoneAddressing),
        OpCode::new(0x5a, "*NOP", 1,2, AddressingMode::NoneAddressing),
        OpCode::new(0x7a, "*NOP", 1,2, AddressingMode::NoneAddressing),
        OpCode::new(0xda, "*NOP", 1,2, AddressingMode::NoneAddressing),
        // OpCode::new(0xea, "NOP", 1,2, AddressingMode::NoneAddressing),
        OpCode::new(0xfa, "*NOP", 1,2, AddressingMode::NoneAddressing),

        OpCode::new(0xab, "*LXA", 2, 3, AddressingMode::Immediate), //todo: highly unstable and not used
        //http://visual6502.org/wiki/index.php?title=6502_Opcode_8B_%28XAA,_ANE%29
        OpCode::new(0x8b, "*XAA", 2, 3, AddressingMode::Immediate), //todo: highly unstable and not used
        OpCode::new(0xbb, "*LAS", 3, 2, AddressingMode::Absolute_Y), //todo: highly unstable and not used
        OpCode::new(0x9b, "*TAS", 3, 2, AddressingMode::Absolute_Y), //todo: highly unstable and not used
        OpCode::new(0x93, "*AHX", 2, /* guess */ 8, AddressingMode::Indirect_Y), //todo: highly unstable and not used
        OpCode::new(0x9f, "*AHX", 3, /* guess */ 4/* or 5*/, AddressingMode::Absolute_Y), //todo: highly unstable and not used
        OpCode::new(0x9e, "*SHX", 3, /* guess */ 4/* or 5*/, AddressingMode::Absolute_Y), //todo: highly unstable and not used
        OpCode::new(0x9c, "*SHY", 3, /* guess */ 4/* or 5*/, AddressingMode::Absolute_X), //todo: highly unstable and not used

        OpCode::new(0xa7, "*LAX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb7, "*LAX", 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0xaf, "*LAX", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbf, "*LAX", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0xa3, "*LAX", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xb3, "*LAX", 2, 5, AddressingMode::Indirect_Y),

        OpCode::new(0x87, "*SAX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x97, "*SAX", 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0x8f, "*SAX", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x83, "*SAX", 2, 6, AddressingMode::Indirect_X),

    ];

    /// A hashmap mapping opcode values to their corresponding `OpCode` instances for easy access.
    pub static ref OPCODE_MAP: HashMap<u8, &'static OpCode> = {
        let mut map = HashMap::new();
        for operation in &*CPU_OP_CODES {
            map.insert(operation.opcode, operation);
        }
        map
    };
}