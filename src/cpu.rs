use core::panic;
use std::collections::HashMap;
use crate::opcode;

const STACK: u16 = 0x0100;
const STACK_RESET: u8 = 0xfd;

bitflags! {
    /// # Status Register (P) http://wiki.nesdev.com/w/index.php/Status_flags
    ///
    ///  7 6 5 4 3 2 1 0
    ///  N V _ B D I Z C
    ///  | |   | | | | +--- Carry Flag
    ///  | |   | | | +----- Zero Flag
    ///  | |   | | +------- Interrupt Disable
    ///  | |   | +--------- Decimal Mode (not used on NES)
    ///  | |   +----------- Break Command
    ///  | +--------------- Overflow Flag
    ///  +----------------- Negative Flag
    ///
    pub struct CpuFlags: u8 {
        const CARRY             = 0b00000001;
        const ZERO              = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100;
        const DECIMAL_MODE      = 0b00001000;
        const BREAK             = 0b00010000;
        const BREAK2            = 0b00100000;
        const OVERFLOW          = 0b01000000;
        const NEGATIVE           = 0b10000000;
    }
}

pub struct CPU {
    pub register_a: u8,           // CPU (A)CCUMULATOR REGISTER
    pub register_x: u8,           // OFFSET REGISTERS
    pub register_y: u8,
    pub status: CpuFlags,             // PROCESSOR STATUS FLAG REGISTER
    pub program_counter: u16,   // CURRENT POSITION IN PROGRAM
    pub stack_pointer: u8,      // STACK LOCATION
    memory: [u8; 0xFFFF],       // GENERIC REPRESENTATION OF NES MEMORY -> {ROM + RAM + IO MEMORY MAP}
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

//////MEMORY FUNCTIONS
trait Memory{
    fn mem_read(&self, address: u16) -> u8; 

    fn mem_write(&mut self, address: u16, data: u8);
    

    /// Reads a  16-bit word from the memory at the specified address.
    ///
    /// # Arguments
    ///
    /// * `pos` - The memory address to read from.
    ///
    /// # Returns
    ///
    /// * `u16` - The  16-bit word read from the memory.
    fn mem_read_u16(&self, position: u16) -> u16 {
        let lo = self.mem_read(position) as u16;
        let hi = self.mem_read(position + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    /// Writes a  16-bit word to the memory at the specified address.
    ///
    /// # Arguments
    ///
    /// * `pos` - The memory address to write to.
    /// * `data` - The  16-bit word to write to the memory.
    fn mem_write_u16(&mut self, position: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(position, lo);
        self.mem_write(position + 1, hi);
    }
}

impl Memory for CPU {
    /// Reads a byte from the memory at the specified address.
    ///
    /// # Arguments
    ///
    /// * `address` - The memory address to read from.
    ///
    /// # Returns
    ///
    /// * `u8` - The byte read from the memory.
    fn mem_read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    /// Writes a byte to the memory at the specified address.
    ///
    /// # Arguments
    ///
    /// * `address` - The memory address to write to.
    /// * `data` - The byte to write to the memory.
    fn mem_write(&mut self, address: u16, data: u8) {
        self.memory[address as usize] = data;
    }
}

impl CPU {
    //////CONSTRUCTOR

    pub fn new() -> Self {
        CPU { 
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: CpuFlags::from_bits_truncate(0b100100),
            program_counter: 0,
            stack_pointer: STACK_RESET,
            memory: [0; 0xFFFF] 
        }
    }

    ////// ADDRESSNG MODE
    /// # Get Operand Address
    /// Based on which addressing mode is engaged, modify cpu register values
    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,

            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            AddressingMode::ZeroPage_X => {
                let position = self.mem_read(self.program_counter);
                let address = position.wrapping_add(self.register_x) as u16;
                address
            }
            AddressingMode::ZeroPage_Y => {
                let position = self.mem_read(self.program_counter);
                let address = position.wrapping_add(self.register_y) as u16;
                address
            },
            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let address = base.wrapping_add(self.register_x as u16);
                address
            },
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let address = base.wrapping_add(self.register_y as u16);
                address
            },
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);
                let pointer: u8 = (base as u8).wrapping_add(self.register_x);
                let low = self.mem_read(pointer as u16);
                let high = self.mem_read(pointer.wrapping_add(1) as u16);
                (high as u16) << 8 | (low as u16)
            },
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);
                let low = self.mem_read(base as u16);
                let high = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (high as u16) << 8 | (low as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            },
            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            },
        }
    }

    //////OPCODE FUNCTIONS
    /// # Add With Carry 
    /// From: https://www.nesdev.org/obelisk-6502-guide/reference.html#ADC.
    /// This instruction adds the contents of a memory location to the accumulator together with the carry bit. 
    /// If overflow occurs the carry bit is set, this enables multiple byte addition to be performed.
    fn adc(&mut self, mode: &AddressingMode) {
        todo!()
    }

    /// # Logical And 
    /// From: https://www.nesdev.org/obelisk-6502-guide/reference.html#AND.
    /// A logical AND is performed, bit by bit, on the accumulator contents using the contents of a byte of memory.
    fn and(&mut self, mode: &AddressingMode) {
        todo!()
    }

    /// # Arithmetic Shift Left
    /// From: https://www.nesdev.org/obelisk-6502-guide/reference.html#ASL.
    /// This operation shifts all the bits of the accumulator or memory contents one bit left. 
    /// Bit 0 is set to 0 and bit 7 is placed in the carry flag. 
    /// The effect of this operation is to multiply the memory contents by 2 (ignoring 2's complement considerations), setting the carry if the result will not fit in 8 bits.
    fn asl(&mut self, mode: &AddressingMode) {
        todo!()
    }

    /// # Generic Branch Function
    /// Covers all branch functions starting with: https://www.nesdev.org/obelisk-6502-guide/reference.html#BCC.
    /// If a certain condition is met, branch program to a new location
    fn branch(&mut self, condition: bool) {
        if condition {
            let jump: i8 = self.mem_read(self.program_counter) as i8;
            let jump_addr = self
                .program_counter
                .wrapping_add(1)
                .wrapping_add(jump as u16);

            self.program_counter = jump_addr;
        }
    }
    
    /// # Bit Test 
    /// From: https://www.nesdev.org/obelisk-6502-guide/reference.html#BIT.
    /// This instructions is used to test if one or more bits are set in a target memory location. 
    /// The mask pattern in A is ANDed with the value in memory to set or clear the zero flag, but the result is not kept. 
    /// Bits 7 and 6 of the value from memory are copied into the N and V flags.
    fn bit(mut self, mode: &AddressingMode) {
        todo!()
    }

    /// # Clear Carry Flag 
    /// From: https://www.nesdev.org/obelisk-6502-guide/reference.html#CLC.
    fn clc(&mut self){
        todo!()
    }

    /// # Clear Decimal Mode Flag 
    /// From: https://www.nesdev.org/obelisk-6502-guide/reference.html#CLD.
    fn cld(&mut self) {
        todo!()
    }
    
    /// # Clear Interrupt Disable Flag
    /// From: https://www.nesdev.org/obelisk-6502-guide/reference.html#CLI.
    fn cli(&mut self) {
        todo!()
    }

    /// # Clear Overflow Flag
    /// From: https://www.nesdev.org/obelisk-6502-guide/reference.html#CLV.
    fn clv(&mut self) {
        todo!()
    }
    /// # Generic Compare Function 
    /// Covers all compary functions including: https://www.nesdev.org/obelisk-6502-guide/reference.html#CMP.
    /// This instruction compares the contents of the given memory location with another memory held value and sets the zero and carry flags as appropriate.
    fn compare(&mut self, mode: &AddressingMode, compare_with: u8) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        if data <= compare_with {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        self.update_zero_and_negative_flags(compare_with.wrapping_sub(data));
    }

    fn dec(&mut self) {
        todo!()
    }

    fn dex(&mut self) {

    }
    
    fn dey(&mut self) {
        todo!()
    }

    fn eor(&mut self) {
        todo!()
    }

    fn inc(&mut self, mode: &AddressingMode) {
        todo!()
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn iny(&mut self) {
        todo!()
    }

    fn lda(&mut self, mode: &AddressingMode){
        let address = self.get_operand_address(mode);
        let value = self.mem_read(address);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        todo!()
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        todo!()
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        todo!()
    }

    fn nop(&mut self){
        todo!()
    }

    fn ora(&mut self, mode: &AddressingMode){
        todo!()
    } 
    
    fn php(&mut self){
        todo!()
    }

    fn pla(&mut self) {
        todo!()
    }

    fn plp(&mut self) {
        todo!()
    }

    fn rol(&mut self, mode: &AddressingMode) -> u8 {
        todo!()
    }

    fn rol_accumulator(&mut self) {
        todo!()
    }

    fn ror(&mut self, mode: &AddressingMode) -> u8 {
        todo!()
    }

    fn ror_accumulator(&mut self) {
        todo!()
    }

    fn rti(&mut self) {
        todo!()
    }

    fn rts(&mut self) {
        todo!()
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        todo!()
    }

    fn sec(&mut self) {
        todo!()
    }

    fn sed(&mut self) {
        todo!()
    }

    fn sei(&mut self) {
        todo!()
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        self.mem_write(address, self.register_a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        todo!()
    }

    fn sty(&mut self, mode: &AddressingMode) {
        todo!()
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn tay(&mut self) {
        todo!()
    }

    fn tsx(&mut self) {
        todo!()   
    }

    fn txa(&mut self) {
        todo!()
    }

    fn txs(&mut self) {
        todo!()
    }

    fn tya(&mut self) {
        todo!()
    }
    
    //// PUSH POPS
    fn stack_push(&mut self, data: u8) {
        self.mem_write((STACK as u16) + self.stack_pointer as u16, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1)
    }

    fn stack_push_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.stack_push(hi);
        self.stack_push(lo);
    }

    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read((STACK as u16) + self.stack_pointer as u16)
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;

        hi << 8 | lo
    }

    ////// HELPER METHODS

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status.insert(CpuFlags::ZERO);
        } else {
            self.status.remove(CpuFlags::ZERO);
        }

        if result & 0b1000_0000 != 0 {
            self.status.insert(CpuFlags::NEGATIVE);
        } else {
            self.status.remove(CpuFlags::NEGATIVE);
        }
    }

    fn set_carry_flag(&mut self) {
        self.status.insert(CpuFlags::CARRY)
    }

    fn clear_carry_flag(&mut self) {
        self.status.remove(CpuFlags::CARRY)
    }

    fn set_register_a(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn add_to_register_a(&mut self, data: u8) {
        let sum = self.register_a as u16
            + data as u16
            + (if self.status.contains(CpuFlags::CARRY) {
                1
            } else {
                0
            }) as u16;

        let carry = sum > 0xff;

        if carry {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        let result = sum as u8;

        if (data ^ result) & (result ^ self.register_a) & 0x80 != 0 {
            self.status.insert(CpuFlags::OVERFLOW);
        } else {
            self.status.remove(CpuFlags::OVERFLOW)
        }

        self.set_register_a(result);
    }


    ////// STATE MANAGEMENT
    /// Loads a program into memory starting at address  0x8000.
    ///
    /// # Arguments
    ///
    /// * `program` - A vector of bytes representing the program to be loaded.
    ///
    /// # Effects
    ///
    /// Sets the program counter to the start of the loaded program.
    pub fn load(&mut self, program: Vec<u8>){
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    /// Loads a program into memory and runs it.
    ///
    /// # Arguments
    ///
    /// * `program` - A vector of bytes representing the program to be loaded and run.
    ///
    /// # Effects
    ///
    /// Calls `load` to load the program into memory and then calls `run` to execute the program.
    pub fn load_and_run(&mut self, program: Vec<u8>){
        self.load(program);
        self.reset();
        self.run()
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    ////// CPU INTERPRETER

    /// # CPU CYCLE IMPLEMENTATION
    /// Fetch next instruction from cpu memory. 
    /// Decode instruction.
    /// Execute instruction.
    /// Repeat.
    pub fn run(&mut self){
        let ref opcodes: HashMap<u8, &'static opcode::OpCode> = *opcode::OPCODE_MAP; 

        loop {
            // FETCH
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            // preserves place in memory for reference
            let program_state = self.program_counter;
            let opcode = opcodes.get(&code).expect(&format!("OpCode {:?} is not recognized", code));
            // DECODE
            match code {
                // OPCODE: LDA - Load Data into Accumulator register -> sets appropriate status flags
                0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => {
                    self.lda(&opcode.mode);
                }
                // OPCODE: STA - STores value from Accumulator register in memory
                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => {
                    self.sta(&opcode.mode);
                }
                // OPCODE: TAX - Transfer from Accumulator register to X register -> sets appropriate status flags
                0xAA => {
                    // Transfer value  
                    self.tax();
                }
                // OPCODE: INX - INcrement X register value
                0xE8 => {
                    self.inx();
                }
                // OPCODE: BRK - Break -> return
                0x00 => {
                    return;
                }
                _ => todo!()
            }

            if program_state == self.program_counter {
                self.program_counter += (opcode.length - 1) as u16;
            }
        }
    }
}

// #[cfg(test)]
// mod test {
//     use std::vec;

//     use super::*;

//     ///// LDA TESTS
//     #[test]
//     fn test_0xa9_lda_immediate_load_data() {
//         let mut cpu = CPU::new();
//         cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
//         assert_eq!(cpu.register_a, 5);
//         assert!(cpu.status & 0b0000_0010 == 0);
//         assert!(cpu.status & 0b1000_0000 == 0);
//     }

//     #[test]
//     fn test_0xa9_lda_zero_flag() {
//         let mut cpu = CPU::new();
//         cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
//         assert!(cpu.status & 0b0000_0010 == 0b10);
//     }

//     #[test]
//     fn test_lda_from_memory(){
//         let mut cpu = CPU::new();
//         cpu.mem_write(0x10, 0x55);

//         cpu.load_and_run(vec![0xa5, 0x10, 0x00]);
//         assert_eq!(cpu.register_a, 0x55);
//     }
//     //// TAX TESTS
//     #[test]
//     fn test_0xaa_tax_move_a_to_x() {
//         let mut cpu = CPU::new();
//         cpu.load_and_run(vec![0xa9, 0x0A,0xaa, 0x00]);

//         assert_eq!(cpu.register_x, 10)
//     }
//     ///// INX TESTS
//     #[test]
//     fn test_inx_overflow() {
//         let mut cpu = CPU::new();
//         cpu.load_and_run(vec![0xa9, 0xff, 0xaa,0xe8, 0xe8, 0x00]);

//         assert_eq!(cpu.register_x, 1)
//     }
//     ///// GENERAL TESTS
//     #[test]
//     fn test_lda_tax_inx_brk_ops_working_together() {
//         let mut cpu = CPU::new();
//         cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

//         assert_eq!(cpu.register_x, 0xc1)
//     }
// }