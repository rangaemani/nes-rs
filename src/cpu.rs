use core::panic;
use std::collections::HashMap;
use crate::opcode;

pub struct CPU {
    pub register_a: u8,           // CPU (A)CCUMULATOR REGISTER
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,             // PROCESSOR STATUS FLAG REGISTER
    pub program_counter: u16,   // CURRENT POSITION IN PROGRAM
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
            status: 0,
            program_counter: 0,
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

    //////OPCODES

    fn lda(&mut self, mode: &AddressingMode){
        let address = self.get_operand_address(mode);
        let value = self.mem_read(address);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }
    
    fn sta(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        self.mem_write(address, self.register_a);
    }

    ////// HELPER METHODS

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
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

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;

    ///// LDA TESTS
    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 5);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_lda_from_memory(){
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);
        assert_eq!(cpu.register_a, 0x55);
    }
    //// TAX TESTS
    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x0A,0xaa, 0x00]);

        assert_eq!(cpu.register_x, 10)
    }
    ///// INX TESTS
    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa,0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }
    ///// GENERAL TESTS
    #[test]
    fn test_lda_tax_inx_brk_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }
}