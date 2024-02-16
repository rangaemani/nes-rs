use core::panic;
use std::collections::HashMap;
use crate::{bus::Bus, opcode};

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
    #[derive(Clone)]
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
    pub bus: Bus,
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
pub trait Memory{
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
    fn mem_read(&self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }
 
    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data)
    }
    fn mem_read_u16(&self, pos: u16) -> u16 {
        self.bus.mem_read_u16(pos)
    }
  
    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        self.bus.mem_write_u16(pos, data)
    }
}

impl CPU {
    //////CONSTRUCTOR

    pub fn new(bus: Bus) -> Self {
        CPU { 
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: CpuFlags::from_bits_truncate(0b100100),
            program_counter: 0,
            stack_pointer: STACK_RESET,
            memory: [0; 0xFFFF],
            bus: bus,
        }
    }

    ////// ADDRESSNG MODE
    pub fn get_absolute_address(&self, mode: &AddressingMode, addr: u16) -> u16 {
        match mode {
            AddressingMode::ZeroPage => self.mem_read(addr) as u16,

            AddressingMode::Absolute => self.mem_read_u16(addr),

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(addr);
                let address = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(addr);
                let address = pos.wrapping_add(self.register_y) as u16;
                addr
            }

            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(addr);
                let address = base.wrapping_add(self.register_x as u16);
                addr
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(addr);
                let address = base.wrapping_add(self.register_y as u16);
                addr
            }

            AddressingMode::Indirect_X => {
                let base = self.mem_read(addr);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(addr);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }

            _ => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

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
        let address = self.get_operand_address(mode);
        let value = self.mem_read(address);
        self.add_to_register_a(value);
    }

    /// # Logical And 
    /// From: https://www.nesdev.org/obelisk-6502-guide/reference.html#AND.
    /// A logical AND is performed, bit by bit, on the accumulator contents using the contents of a byte of memory.
    fn and(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.mem_read(address);
        self.set_register_a(value & self.register_a);
    }
 
    /// # And Rotate Right
    /// AND byte with accumulator, then rotate one bit right in accu-mulator and check bit 5 and 6:
    /// If both bits are 1: set C, clear V.
    /// If both bits are 0: clear C and V.
    /// If only bit 5 is 1: set V, clear C.
    /// If only bit 6 is 1: set C and V.
    /// Status flags: N,V,Z,C
    fn arr(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let data = self.mem_read(address);
        self.and_with_register_a(data);
        self.ror_accumulator();
        let result = self.register_a;
        let bit_5 = (result >> 5) & 1;
        let bit_6 = (result >> 6) & 1;

        if bit_6 == 1 {
            self.status.insert(CpuFlags::CARRY)
        } else {
            self.status.remove(CpuFlags::CARRY)
        }

        if bit_5 ^ bit_6 == 1 {
            self.status.insert(CpuFlags::OVERFLOW);
        } else {
            self.status.remove(CpuFlags::OVERFLOW);
        }

        self.update_zero_and_negative_flags(result);
    }

    /// # Arithmetic Shift Left
    /// From: https://www.nesdev.org/obelisk-6502-guide/reference.html#ASL.
    /// This operation shifts all the bits of the accumulator or memory contents one bit left. 
    /// Bit 0 is set to 0 and bit 7 is placed in the carry flag. 
    /// The effect of this operation is to multiply the memory contents by 2 (ignoring 2's complement considerations), setting the carry if the result will not fit in 8 bits.
    fn asl(&mut self, mode: &AddressingMode) -> u8{
        let address = self.get_operand_address(mode);
        let mut data = self.mem_read(address);
        if data >> 7 == 1 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag()
        }
        data = data << 1;
        self.mem_write(address, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    fn asl_accumulator(&mut self) {
        let mut data = self.register_a;
        if data >> 7 == 1 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }
        data = data << 1;
        self.set_register_a(data);
    }

    fn asx(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let data = self.mem_read(address);
        let x_and_a = self.register_x & self.register_a;
        let result = x_and_a.wrapping_sub(data);

        if data <= x_and_a {
            self.status.insert(CpuFlags::CARRY);
        }
        self.update_zero_and_negative_flags(result);

        self.register_x = result;
    }
    /// # Generic Branch Function
    /// Covers all branch functions starting with: https://www.nesdev.org/obelisk-6502-guide/reference.html#BCC.
    /// If a certain condition is met, branch program to a new location
    fn branch(&mut self, condition: bool) {
        if condition {
            let jump: i8 = self.mem_read(self.program_counter) as i8;
            let jump_address = self
                .program_counter
                .wrapping_add(1)
                .wrapping_add(jump as u16);

            self.program_counter = jump_address;
        }
    }
    
    /// # Bit Test 
    /// From: https://www.nesdev.org/obelisk-6502-guide/reference.html#BIT.
    /// This instructions is used to test if one or more bits are set in a target memory location. 
    /// The mask pattern in A is ANDed with the value in memory to set or clear the zero flag, but the result is not kept. 
    /// Bits 7 and 6 of the value from memory are copied into the N and V flags.
    fn bit(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let data = self.mem_read(address);
        let and = self.register_a & data;
        if and == 0 {
            self.status.insert(CpuFlags::ZERO);
        } else {
            self.status.remove(CpuFlags::ZERO);
        }
        self.status.set(CpuFlags::NEGATIVE, data & 0b10000000 > 0);
        self.status.set(CpuFlags::OVERFLOW, data & 0b01000000 > 0);
    }

    /// # Clear Carry Flag 
    /// From: https://www.nesdev.org/obelisk-6502-guide/reference.html#CLC.
    fn clc(&mut self){
        self.clear_carry_flag();
    }

    /// # Clear Decimal Mode Flag 
    /// From: https://www.nesdev.org/obelisk-6502-guide/reference.html#CLD.
    fn cld(&mut self) {
        self.status.remove(CpuFlags::DECIMAL_MODE);
    }
    
    /// # Clear Interrupt Disable Flag
    /// From: https://www.nesdev.org/obelisk-6502-guide/reference.html#CLI.
    fn cli(&mut self) {
        self.status.remove(CpuFlags::INTERRUPT_DISABLE);
    }

    /// # Clear Overflow Flag
    /// From: https://www.nesdev.org/obelisk-6502-guide/reference.html#CLV.
    fn clv(&mut self) {
        self.status.remove(CpuFlags::OVERFLOW);
    }
    /// # Generic Compare Function 
    /// Covers all compary functions including: https://www.nesdev.org/obelisk-6502-guide/reference.html#CMP.
    /// This instruction compares the contents of the given memory location with another memory held value and sets the zero and carry flags as appropriate.
    fn compare(&mut self, mode: &AddressingMode, compare_with: u8) {
        let address = self.get_operand_address(mode);
        let data = self.mem_read(address);
        if data <= compare_with {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        self.update_zero_and_negative_flags(compare_with.wrapping_sub(data));
    }

    /// # Dec + CmP
    /// Subtract 1 from memory (without borrow).
    fn dcp(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let mut data = self.mem_read(address);
        data = data.wrapping_sub(1);
        self.mem_write(address, data);
        // self._update_zero_and_negative_flags(data);
        if data <= self.register_a {
            self.status.insert(CpuFlags::CARRY);
        }
        self.update_zero_and_negative_flags(self.register_a.wrapping_sub(data));
    }

    /// # Decrement Memory
    /// From: https://www.nesdev.org/obelisk-6502-guide/reference.html#DEC.
    /// Subtracts one from the value held at a specified memory location setting the zero and negative flags as appropriate.
    fn dec(&mut self, mode: &AddressingMode) -> u8 {
        let address = self.get_operand_address(mode);
        let mut data = self.mem_read(address);
        data = data.wrapping_sub(1);
        self.mem_write(address, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    /// # Decrement X Register
    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x);
    }
    /// # Decrement Y Register
    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    /// # XOR
    /// An exclusive OR is performed, bit by bit, on the accumulator contents using the contents of a byte of memory.
    fn eor(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let data = self.mem_read(address);
        self.mem_write(address, data ^ self.register_a);  // lol i never knew `^` was the xor op
    }

    /// # Increment
    fn inc(&mut self, mode: &AddressingMode) -> u8 {
        let address = self.get_operand_address(mode);
        let mut data = self.mem_read(address);
        data = data.wrapping_add(1);
        self.mem_write(address, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    /// # Increment X Register
    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    /// # Increment Y Register
    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    /// # Jump
    /// Sets the program counter to the address specified by the operand.
    fn jmp(&mut self){
        let mem_address = self.mem_read_u16(self.program_counter);
        // let indirect_ref = self.mem_read_u16(mem_address);
        //6502 bug mode with with page boundary:
        //  if address $3000 contains $40, $30FF contains $80, and $3100 contains $50,
        // the result of JMP ($30FF) will be a transfer of control to $4080 rather than $5080 as you intended
        // i.e. the 6502 took the low byte of the address from $30FF and the high byte from $3000

        let indirect_ref = if mem_address & 0x00FF == 0x00FF {
            let lo = self.mem_read(mem_address);
            let hi = self.mem_read(mem_address & 0xFF00);
            (hi as u16) << 8 | (lo as u16)
        } else {
            self.mem_read_u16(mem_address)
        };

        self.program_counter = indirect_ref;
    }

    /// # Jump to SubRoutine 
    /// The JSR instruction pushes the address (minus one) of the return point on to the stack and then sets the program counter to the target memory address.
    fn jsr(&mut self) {
        self.stack_push_u16(self.program_counter + 2 - 1);
        let target_address = self.mem_read_u16(self.program_counter);
        self.program_counter = target_address
    }

    /// # Load Data (into) Accumulator
    fn lda(&mut self, mode: &AddressingMode){
        let address = self.get_operand_address(mode);
        let value = self.mem_read(address);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    /// # Load Data into X register
    fn ldx(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.mem_read(address);

        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    /// # Load Y Register
    fn ldy(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.mem_read(address);

        self.register_y = value;
        self.update_zero_and_negative_flags(self.register_y);
    }

    /// # Logical (bit) Shift Right
    /// Each of the bits in A or M is shift one place to the right. 
    /// The bit that was in bit 0 is shifted into the carry flag. 
    /// Bit 7 is set to zero.
    fn lsr(&mut self, mode: &AddressingMode) -> u8 {
        let address = self.get_operand_address(mode);
        let mut data = self.mem_read(address);
        if data & 1 == 1 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }
        data = data >> 1;
        self.mem_write(address, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    fn lsr_accumulator(&mut self) {
        let mut data = self.register_a;
        if data & 1 == 1 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }
        data = data >> 1;
        self.set_register_a(data);
    }

    /// # Logical Inclusive Or
    /// An inclusive OR is performed, bit by bit, on the accumulator contents using the contents of a byte of memory.
    fn ora(&mut self, mode: &AddressingMode){
        let address = self.get_operand_address(mode);
        let data = self.mem_read(address);
        self.set_register_a(self.register_a | data);
    } 

    /// # Push Accumulator to stack
    fn pha(&mut self){
        self.stack_push(self.register_a);
    }
    
    /// # Push Processor Status flags onto stack
    /// Pushes a copy of the status flags on to the stack.
    fn php(&mut self){
        let mut flags = self.status.clone();
        flags.insert(CpuFlags::BREAK);
        flags.insert(CpuFlags::BREAK2);
        self.stack_push(flags.bits());
    }

    /// # Pull Accumulator
    /// Pulls an 8 bit value from the stack and into the accumulator. 
    /// The zero and negative flags are set as appropriate.
    fn pla(&mut self) {
        let data = self.stack_pop();
        self.set_register_a(data);
    }

    /// # Pull Processor Status
    /// Pulls an 8 bit value from the stack and into the processor flags. 
    /// The flags will take on new states as determined by the value pulled.
    fn plp(&mut self) {
        self.status = CpuFlags::from_bits_truncate(self.stack_pop());
        self.status.remove(CpuFlags::BREAK);
        self.status.insert(CpuFlags::BREAK2);
    }

    /// # Rotate Left
    /// Move each of the bits in either A or M one place to the left. 
    /// Bit 0 is filled with the current value of the carry flag whilst the old bit 7 becomes the new carry flag value.
    fn rol(&mut self, mode: &AddressingMode) -> u8 {
        let address = self.get_operand_address(mode);
        let mut data = self.mem_read(address);
        let previous_carry_flag_set = self.status.contains(CpuFlags::CARRY);

        if data >> 7 == 1 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }
        data = data << 1;
        if previous_carry_flag_set {
            data = data | 1;
        }
        self.mem_write(address, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    /// # Rotate Left Accumulator
    fn rol_accumulator(&mut self) {
        let mut data = self.register_a;
        let previous_carry_flag_set = self.status.contains(CpuFlags::CARRY);

        if data >> 7 == 1 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }
        data = data << 1;
        if previous_carry_flag_set {
            data = data | 1;
        }
        self.set_register_a(data);
    }

    /// # Rotate Right
    fn ror(&mut self, mode: &AddressingMode) -> u8 {
        let address = self.get_operand_address(mode);
        let mut data = self.mem_read(address);
        let previous_carry_value_set = self.status.contains(CpuFlags::CARRY);

        if data & 1 == 1 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }
        data = data >> 1;
        if previous_carry_value_set {
            data = data | 0b10000000;
        }
        self.mem_write(address, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    /// # Rotate Right Accumulator
    fn ror_accumulator(&mut self) {
        let mut data = self.register_a;
        let previous_carry_value_set = self.status.contains(CpuFlags::CARRY);

        if data & 1 == 1 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }
        data = data >> 1;
        if previous_carry_value_set {
            data = data | 0b10000000;
        }
        self.set_register_a(data);
    }

    /// # Return from Interrupt
    fn rti(&mut self) {
        self.status = CpuFlags::from_bits_truncate(self.stack_pop());
        self.status.remove(CpuFlags::BREAK);
        self.status.insert(CpuFlags::BREAK2);

        self.program_counter = self.stack_pop_u16();
    }

    /// # Return from Subroutine
    fn rts(&mut self) {
        self.program_counter = self.stack_pop_u16() + 1;
    }

    /// # Subtract with Carry
    /// This instruction subtracts the contents of a memory location to the accumulator together with the not of the carry bit. 
    /// If overflow occurs the carry bit is clear, this enables multiple byte subtraction to be performed.
    fn sbc(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let data = self.mem_read(address);
        self.add_to_register_a(((data as i8).wrapping_neg().wrapping_sub(1)) as u8);
        
    }

    ///// FLAGSET OPS
    /// # Carry Flag
    fn sec(&mut self) {
        self.set_carry_flag();
    }

    /// # Decimal Flag
    fn sed(&mut self) {
        self.status.insert(CpuFlags::DECIMAL_MODE);
    }

    /// # Interrupt Flag
    fn sei(&mut self) {
        self.status.insert(CpuFlags::INTERRUPT_DISABLE);
    }

    /// # Store Accumulator
    fn sta(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        self.mem_write(address, self.register_a);
    }

    /// # Store X Register
    fn stx(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        self.mem_write(address, self.register_x);
    }

    /// # Store Y Register
    fn sty(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        self.mem_write(address, self.register_y);
    }

    /// # Transfer Accumulator to X
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    /// # Transfer Accumulator to Y
    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    /// # Transfer Stack pointer to X
    fn tsx(&mut self) {
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }

    /// # Transfer X to Accumulator
    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    /// # Transfer X to Stack pointer
    fn txs(&mut self) {
        self.stack_pointer = self.register_x;
    }

    /// # Transfer Y to Accumulator 
    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }
    
    //// PUSH POPS
    /// # Push Ops (PHA)
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

    /// Updates the `ZERO` and `NEGATIVE` flags based on the result.
    ///
    /// # Arguments
    ///
    /// * `result` - The result of an operation to test against the flags' conditions.
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
    /// Sets the CPU Carry Flag
    fn set_carry_flag(&mut self) {
        self.status.insert(CpuFlags::CARRY)
    }

    /// Clears the CPU Carry Flag
    fn clear_carry_flag(&mut self) {
        self.status.remove(CpuFlags::CARRY)
    }

    /// Sets the accumulator register (`register_a`) to the provided value. Updates zero and negative cpu flags accordingly.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to be inserted into the accumulator register
    fn set_register_a(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    /// Adds the given data to the accumulator (`register_a`), including the carry if set, and updates the CPU flags.
    ///
    /// # Arguments
    ///
    /// * `data` - The 8-bit data to add to the accumulator.
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

    fn sub_from_register_a(&mut self, data: u8) {
        self.add_to_register_a(((data as i8).wrapping_neg().wrapping_sub(1)) as u8);
    }

    fn and_with_register_a(&mut self, data: u8) {
        self.set_register_a(data & self.register_a);
    }

    fn xor_with_register_a(&mut self, data: u8) {
        self.set_register_a(data ^ self.register_a);
    }

    fn or_with_register_a(&mut self, data: u8) {
        self.set_register_a(data | self.register_a);
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
        self.memory[0x0600..(0x0600 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x0600);
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
        self.status = CpuFlags::ZERO;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    ////// CPU INTERPRETER

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    /// # CPU CYCLE IMPLEMENTATION
    /// Fetch next instruction from cpu memory. 
    /// Decode instruction.
    /// Execute instruction.
    /// Repeat.
    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        let ref opcodes: HashMap<u8, &'static opcode::OpCode> = *opcode::OPCODE_MAP;

        loop {
            callback(self);
            ///// FETCH
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            // preserves place in memory for reference
            let program_state = self.program_counter;
            let opcode = opcodes.get(&code).expect(&format!("OpCode {:?} is not recognized", code));
            ///// DECODE
            match code {
                ///// EXECUTE
                /* ADC */
                0x69 |  0x65 |  0x75 |  0x6d |  0x7d |  0x79 |  0x61 |  0x71 => {
                    self.adc(&opcode.mode);
                },

                /* AND */
                0x29 |  0x25 |  0x35 |  0x2d |  0x3d |  0x39 |  0x21 |  0x31 => {
                    self.and(&opcode.mode);
                },

                /*ASL*/ 0x0a => self.asl_accumulator(),

                /* ASL */
                0x06 |  0x16 |  0x0e |  0x1e => {
                    self.asl(&opcode.mode);
                },

                /* BCC */
                0x90 => {
                    self.branch(!self.status.contains(CpuFlags::CARRY));
                },

                /* BCS */
                0xb0 => {
                    self.branch(self.status.contains(CpuFlags::CARRY));
                },

                /* BEQ */
                0xf0 => {
                    self.branch(self.status.contains(CpuFlags::ZERO));
                },

                /* BIT */
                0x24 |  0x2c => {
                    self.bit(&opcode.mode);
                },

                /* BMI */
                0x30 => {
                    self.branch(self.status.contains(CpuFlags::NEGATIVE));
                },

                /* BNE */
                0xd0 => {
                    self.branch(!self.status.contains(CpuFlags::ZERO));
                },

                /* BPL */
                0x10 => {
                    self.branch(!self.status.contains(CpuFlags::NEGATIVE));
                },

                /* BRK */
                0x00 => return,

                /* BVC */
                0x50 => {
                    self.branch(!self.status.contains(CpuFlags::OVERFLOW));
                },

                /* BVS */
                0x70 => {
                    self.branch(self.status.contains(CpuFlags::OVERFLOW));
                },

                /* CLC */
                0x18 => self.clc(),

                /* CLD */
                0xd8 => self.cld(),

                /* CLI */
                0x58 => self.cli(),

                /* CLV */
                0xb8 => self.clv(),

                /* CMP */
                0xc9 |  0xc5 |  0xd5 |  0xcd |  0xdd |  0xd9 |  0xc1 |  0xd1 => {
                    self.compare(&opcode.mode, self.register_a);
                },

                /* CPX */
                0xe0 |  0xe4 |  0xec => self.compare(&opcode.mode, self.register_x),

                /* CPY */
                0xc0 |  0xc4 |  0xcc => {
                    self.compare(&opcode.mode, self.register_y);
                },

                /* DEC */
                0xc6 |  0xd6 |  0xce |  0xde => {
                    self.dec(&opcode.mode);
                },

                /* DEX */
                0xca => {
                    self.dex();
                },

                /* DEY */
                0x88 => {
                    self.dey();
                },

                /* EOR */
                0x49 |  0x45 |  0x55 |  0x4d |  0x5d |  0x59 |  0x41 |  0x51 => {
                    self.eor(&opcode.mode);
                },

                /* INC */
                0xe6 |  0xf6 |  0xee |  0xfe => {
                    self.inc(&opcode.mode);
                },

                /* INX */
                0xe8 => self.inx(),

                /* INY */
                0xc8 => self.iny(),

                /* JMP Absolute */
                0x4c => {
                    let mem_address = self.mem_read_u16(self.program_counter);
                    self.program_counter = mem_address;
                },

                /* JMP Indirect */
                0x6c => self.jmp(),

                /* JSR */
                0x20 => self.jsr(),

                /* LDA */
                0xa9 |  0xa5 |  0xb5 |  0xad |  0xbd |  0xb9 |  0xa1 |  0xb1 => {
                    self.lda(&opcode.mode);
                },

                /* LDX */
                0xa2 |  0xa6 |  0xb6 |  0xae |  0xbe => {
                    self.ldx(&opcode.mode);
                },

                /* LDY */
                0xa0 |  0xa4 |  0xb4 |  0xac |  0xbc => {
                    self.ldy(&opcode.mode);
                },

                /* LSR */ 0x4a => self.lsr_accumulator(),

                /* LSR */
                0x46 |  0x56 |  0x4e |  0x5e => {
                    self.lsr(&opcode.mode);
                },
                /* ORA */
                0x09 |  0x05 |  0x15 |  0x0d |  0x1d |  0x19 |  0x01 |  0x11 => {
                    self.ora(&opcode.mode);
                },
                /* PHA */
                0x48 => self.pha(),

                /* PHP */
                0x08 => {
                    self.php();
                },

                /* PLA */
                0x68 => {
                    self.pla();
                },

                /* PLP */
                0x28 => {
                    self.plp();
                },

                /*ROL*/ 0x2a => self.rol_accumulator(),

                /* ROL */
                0x26 |   0x36 |   0x2e |   0x3e => {
                    self.rol(&opcode.mode);
                },

                /* ROR */ 0x6a => self.ror_accumulator(),

                /* ROR */
                0x66 |   0x76 |   0x6e |   0x7e => {
                    self.ror(&opcode.mode);
                },

                /* RTI */
                0x40 => {
                    self.rti();
                },

                /* RTS */
                0x60 => {
                    self.rts()
                },

                /* SBC */
                0xe9 |   0xe5 |   0xf5 |   0xed |   0xfd |   0xf9 |   0xe1 |   0xf1 => {
                    self.sbc(&opcode.mode);
                },

                /* SEC */
                0x38 => self.sec(),

                /* SED */
                0xf8 => self.sed(),

                /* SEI */
                0x78 => self.sei(),

                /* STA */
                0x85 |   0x95 |   0x8d |   0x9d |   0x99 |   0x81 |   0x91 => {
                    self.sta(&opcode.mode);
                },

                /* STX */
                0x86 |   0x96 |   0x8e => {
                    self.stx(&opcode.mode)
                },

                /* STY */
                0x84 |   0x94 |   0x8c => {
                    self.sty(&opcode.mode)
                },

                /* TAX */
                0xaa => self.tax(),

                /* TAY */
                0xa8 => {
                    self.tay()
                },

                /* TSX */
                0xba => {
                    self.tsx()
                },

                /* TXA */
                0x8a => {
                    self.txa()
                },

                /* TXS */
                0x9a => {
                    self.txs()
                },

                /* TYA */
                0x98 => {
                    self.tya()
                },

                /* NOP */
                0xea => {
                    // do nothing
                },

                ////// UNOFFICIAL OPCODES

                /* DCP */
                0xc7 | 0xd7 | 0xCF | 0xdF | 0xdb | 0xd3 | 0xc3 => {
                    self.dcp(&opcode.mode);
                }

                /* RLA */
                0x27 | 0x37 | 0x2F | 0x3F | 0x3b | 0x33 | 0x23 => {
                    let data = self.rol(&opcode.mode);
                    self.and_with_register_a(data);
                }

                /* SLO */ 
                0x07 | 0x17 | 0x0F | 0x1f | 0x1b | 0x03 | 0x13 => {
                    let data = self.asl(&opcode.mode);
                    self.or_with_register_a(data);
                }

                /* SRE */ 
                0x47 | 0x57 | 0x4F | 0x5f | 0x5b | 0x43 | 0x53 => {
                    let data = self.lsr(&opcode.mode);
                    self.xor_with_register_a(data);
                }

                /* SKB */
                0x80 | 0x82 | 0x89 | 0xc2 | 0xe2 => {
                    /* 2 byte NOP (immediate ) */
                    
                }

                /* AXS */
                0xCB => {
                    self.asx(&opcode.mode);
                }

                /* ARR */
                0x6B => {
                    self.arr(&opcode.mode);
                }

                /* unofficial SBC */
                0xeb => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let data = self.mem_read(addr);
                    self.sub_from_register_a(data);
                }

                /* ANC */
                0x0b | 0x2b => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let data = self.mem_read(addr);
                    self.and_with_register_a(data);
                    if self.status.contains(CpuFlags::NEGATIVE) {
                        self.status.insert(CpuFlags::CARRY);
                    } else {
                        self.status.remove(CpuFlags::CARRY);
                    }
                }

                /* ALR */
                0x4b => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let data = self.mem_read(addr);
                    self.and_with_register_a(data);
                    self.lsr_accumulator();
                }


                /* NOP read */
                0x04 | 0x44 | 0x64 | 0x14 | 0x34 | 0x54 | 0x74 | 0xd4 | 0xf4 | 0x0c | 0x1c
                | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => {
                    /* read and then do nothing? i guess */
                    let addr = self.get_operand_address(&opcode.mode);
                    let _data = self.mem_read(addr);
                }

                /* RRA */
                0x67 | 0x77 | 0x6f | 0x7f | 0x7b | 0x63 | 0x73 => {
                    let data = self.ror(&opcode.mode);
                    self.add_to_register_a(data);
                }

                /* ISB */
                0xe7 | 0xf7 | 0xef | 0xff | 0xfb | 0xe3 | 0xf3 => {
                    let data = self.inc(&opcode.mode);
                    self.sub_from_register_a(data);
                }

                /* NOPs */
                0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72 | 0x92 | 0xb2 | 0xd2
                | 0xf2 => { /* do nothing */ }

                0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xfa => { /* do nothing */ }
                // sure are a lot of unofficial opcodes that are useless

                /* LAX */
                0xa7 | 0xb7 | 0xaf | 0xbf | 0xa3 | 0xb3 => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let data = self.mem_read(addr);
                    self.set_register_a(data);
                    self.register_x = self.register_a;
                }

                /* SAX */
                0x87 | 0x97 | 0x8f | 0x83 => {
                    let data = self.register_a & self.register_x;
                    let addr = self.get_operand_address(&opcode.mode);
                    self.mem_write(addr, data);
                }

                /* LXA */
                0xab => {
                    self.lda(&opcode.mode);
                    self.tax();
                }

                /* XAA */
                0x8b => {
                    self.register_a = self.register_x;
                    self.update_zero_and_negative_flags(self.register_a);
                    let addr = self.get_operand_address(&opcode.mode);
                    let data = self.mem_read(addr);
                    self.and_with_register_a(data);
                }

                /* LAS */
                0xbb => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let mut data = self.mem_read(addr);
                    data = data & self.stack_pointer;
                    self.register_a = data;
                    self.register_x = data;
                    self.stack_pointer = data;
                    self.update_zero_and_negative_flags(data);
                }

                /* TAS */
                0x9b => {
                    let data = self.register_a & self.register_x;
                    self.stack_pointer = data;
                    let mem_address =
                        self.mem_read_u16(self.program_counter) + self.register_y as u16;

                    let data = ((mem_address >> 8) as u8 + 1) & self.stack_pointer;
                    self.mem_write(mem_address, data)
                }

                /* AHX  Indirect Y */
                0x93 => {
                    let pos: u8 = self.mem_read(self.program_counter);
                    let mem_address = self.mem_read_u16(pos as u16) + self.register_y as u16;
                    let data = self.register_a & self.register_x & (mem_address >> 8) as u8;
                    self.mem_write(mem_address, data)
                }

                /* AHX Absolute Y*/
                0x9f => {
                    let mem_address =
                        self.mem_read_u16(self.program_counter) + self.register_y as u16;

                    let data = self.register_a & self.register_x & (mem_address >> 8) as u8;
                    self.mem_write(mem_address, data)
                }

                /* SHX */
                0x9e => {
                    let mem_address =
                        self.mem_read_u16(self.program_counter) + self.register_y as u16;
                    let data = self.register_x & ((mem_address >> 8) as u8 + 1);
                    self.mem_write(mem_address, data)
                }

                /* SHY */
                0x9c => {
                    let mem_address =
                        self.mem_read_u16(self.program_counter) + self.register_x as u16;
                    let data = self.register_y & ((mem_address >> 8) as u8 + 1);
                    self.mem_write(mem_address, data)
                }

                _ => todo!()
            }
            if program_state == self.program_counter {
                self.program_counter += (opcode.length - 1) as u16;
            }    ///// REPEAT
        }
    }
}

#[cfg(test)]
mod test {
    // use super::*;
    // use crate::cartridge::test;

    // #[test]
    // fn test_0xa9_lda_immediate_load_data() {
    //     let bus = Bus::new(test::test_rom());
    //     let mut cpu = CPU::new(bus);
    //     cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
    //     assert_eq!(cpu.register_a, 5);
    //     assert!(cpu.status.bits() & 0b0000_0010 == 0b00);
    //     assert!(cpu.status.bits() & 0b1000_0000 == 0);
    // }

    // #[test]
    // fn test_0xaa_tax_move_a_to_x() {
    //     let bus = Bus::new(test::test_rom());
    //     let mut cpu = CPU::new(bus);
    //     cpu.register_a = 10;
    //     cpu.load_and_run(vec![0xaa, 0x00]);

    //     assert_eq!(cpu.register_x, 10)
    // }

    // #[test]
    // fn test_5_ops_working_together() {
    //     let bus = Bus::new(test::test_rom());
    //     let mut cpu = CPU::new(bus);
    //     cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

    //     assert_eq!(cpu.register_x, 0xc1)
    // }

    // #[test]
    // fn test_inx_overflow() {
    //     let bus = Bus::new(test::test_rom());
    //     let mut cpu = CPU::new(bus);
    //     cpu.register_x = 0xff;
    //     cpu.load_and_run(vec![0xe8, 0xe8, 0x00]);

    //     assert_eq!(cpu.register_x, 1)
    // }

    // #[test]
    // fn test_lda_from_memory() {
    //     let bus = Bus::new(test::test_rom());
    //     let mut cpu = CPU::new(bus);
    //     cpu.mem_write(0x10, 0x55);

    //     cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

    //     assert_eq!(cpu.register_a, 0x55);
    // }
}
