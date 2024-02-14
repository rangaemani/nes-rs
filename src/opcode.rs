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
        //// BREAK
        OpCode::new(0x00, "BRK",  1,  7, AddressingMode::NoneAddressing),
        //// TAX
        OpCode::new(0xaa, "TAX",  1,  2, AddressingMode::NoneAddressing),
        //// INX
        OpCode::new(0xe8, "INX",  1,  2, AddressingMode::NoneAddressing),
        //// LDA
        OpCode::new(0xa9, "LDA",  2,  2, AddressingMode::Immediate),
        OpCode::new(0xa5, "LDA",  2,  3, AddressingMode::ZeroPage),
        OpCode::new(0xb5, "LDA",  2,  4, AddressingMode::ZeroPage_X),
        OpCode::new(0xad, "LDA",  3,  4, AddressingMode::Absolute),
        OpCode::new(0xbd, "LDA",  3,  4 /*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0xb9, "LDA",  3,  4 /*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0xa1, "LDA",  2,  6, AddressingMode::Indirect_X),
        OpCode::new(0xb1, "LDA",  2,  5 /*+1 if page crossed*/, AddressingMode::Indirect_Y),
        //// STA
        OpCode::new(0x85, "STA",  2,  3, AddressingMode::ZeroPage),
        OpCode::new(0x95, "STA",  2,  4, AddressingMode::ZeroPage_X),
        OpCode::new(0x8d, "STA",  3,  4, AddressingMode::Absolute),
        OpCode::new(0x9d, "STA",  3,  5, AddressingMode::Absolute_X),
        OpCode::new(0x99, "STA",  3,  5, AddressingMode::Absolute_Y),
        OpCode::new(0x81, "STA",  2,  6, AddressingMode::Indirect_X),
        OpCode::new(0x91, "STA",  2,  6, AddressingMode::Indirect_Y),
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