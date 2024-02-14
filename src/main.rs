pub struct CPU {
    pub register_a: u8,           // CPU (A)CCUMULATOR REGISTER
    pub register_x: u8,
    pub status: u8,             // PROCESSOR STATUS FLAG REGISTER
    pub program_counter: u16,   // CURRENT POSITION IN PROGRAM
}

impl CPU {
    pub fn new() -> Self {
        CPU { register_a: 0, register_x: 0, status: 0, program_counter: 0 }
    }

    /// # CPU CYCLE IMPLEMENTATION
    /// Fetch next instruction from cpu memory. 
    /// Decode instruction.
    /// Execute instruction.
    /// Repeat.
    /// # Arguments
    ///
    /// * `program` - A vector of bytes representing the program to be interpreted.
    pub fn interpret(&mut self, program: Vec<u8>){
        self.program_counter = 0;

        loop {
            // FETCH
            let opcode = program[self.program_counter as usize];
            self.program_counter += 1;
            // DECODE
            match opcode {
                // OPCODE: LDA - Load Data into Accumulator register -> sets appropriate status flags
                0xA9 => {
                    let parameter = program[self.program_counter as usize];
                    self.program_counter += 1;
                    self.register_a = parameter;

                    // Update the zero flag based on the value of the accumulator register (A).
                    if self.register_a ==  0 {
                        self.status |=  0b0000_0010; // Set the zero flag if A is zero.
                    } else {
                        self.status &= !0b0000_0010; // Clear the zero flag if A is non-zero.
                    }

                    // Update the negative flag based on the sign bit of the accumulator register (A).
                    if self.register_a &  0b1000_0000 !=  0 {
                        self.status |=  0b1000_0000; // Set the negative flag if the sign bit is set.
                    } else {
                        self.status &= !0b1000_0000; // Clear the negative flag if the sign bit is not set.
                    }
                }
                // OPCODE: TAX - Transfer from Accumulator register to X register -> sets appropriate status flags
                0xAA => {
                    // Transfer value  
                    self.register_x = self.register_a;
                    if self.register_x == 0 {
                        self.status = self.status | 0b0000_0010;
                    } else {
                        self.status = self.status & 0b1111_1101;
                    }

                    if self.register_x & 0b1000_0000 != 0 {
                        self.status = self.status | 0b1000_0000;
                    } else {
                        self.status = self.status & 0b0111_1111;
                    }
                }

                // OPCODE: BRK - Break -> return
                0x00 => {
                    return;
                }
                _ => todo!()
            }
        }
    }
}

fn main(){}

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;

    #[test]
    /// # Test LDA opcode 
    /// Checks that parameter is properly loaded into accumulator register
    fn test_0xa9_lda_immediate_load_data(){
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0); 
    }

    #[test]
    /// # Test LDA opcode
    /// Checks that status flags are set appropriately based on passed param
    fn test_0xa9_lda_zero_flag(){
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    /// # Test TAX opcode
    fn test_0xaa_tax_move_a_to_x(){
        let mut cpu = CPU::new();
        cpu.register_a = 10;
        cpu.interpret(vec![0xaa, 0x00]);
        assert_eq!(cpu.register_x, 10);
    }
}