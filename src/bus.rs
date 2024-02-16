use crate::{cartridge::Rom, cpu::Memory};

//  _______________ $10000  _______________
// | PRG-ROM       |       |               |
// | Upper Bank    |       |               |
// |_ _ _ _ _ _ _ _| $C000 | PRG-ROM       |
// | PRG-ROM       |       |               |
// | Lower Bank    |       |               |
// |_______________| $8000 |_______________|
// | SRAM          |       | SRAM          |
// |_______________| $6000 |_______________|
// | Expansion ROM |       | Expansion ROM |
// |_______________| $4020 |_______________|
// | I/O Registers |       |               |
// |_ _ _ _ _ _ _ _| $4000 |               |
// | Mirrors       |       | I/O Registers |
// | $2000-$2007   |       |               |
// |_ _ _ _ _ _ _ _| $2008 |               |
// | I/O Registers |       |               |
// |_______________| $2000 |_______________|
// | Mirrors       |       |               |
// | $0000-$07FF   |       |               |
// |_ _ _ _ _ _ _ _| $0800 |               |
// | RAM           |       | RAM           |
// |_ _ _ _ _ _ _ _| $0200 |               |
// | Stack         |       |               |
// |_ _ _ _ _ _ _ _| $0100 |               |
// | Zero Page     |       |               |
// |_______________| $0000 |_______________|

// RICOH 2A03 MEMORY BUS DIAGRAM

const RAM_ADDRESS: u16 = 0x0000;
const RAM_END_ADDRESS: u16 = 0x1FFF;
const PPU_REGISTERS_ADDRESS: u16 = 0x2000;
const PPU_REGISTERS_END_ADDRESS: u16 = 0x3FFF;

pub struct Bus {
    cpu_vram: [u8; 2048],
    rom: Rom,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        Bus {
            cpu_vram: [0; 2048],
            rom,
        }
    }

    fn read_prg_rom(&self, mut address: u16) -> u8 {
        address -= 0x8000;
        if self.rom.prg_rom.len() == 0x4000 && address >= 0x4000 {
            //mirror if needed
            address = address % 0x4000;
        }
        self.rom.prg_rom[address as usize]
    }
}

impl Memory for Bus {
    fn mem_read(&self, address: u16) -> u8 {
        match address {
            RAM_ADDRESS ..= RAM_END_ADDRESS => {
                let mirror_bus_address = address & 0b00000111_11111111;
                self.cpu_vram[mirror_bus_address as usize]
            }
            PPU_REGISTERS_ADDRESS ..= PPU_REGISTERS_END_ADDRESS => {
                let mirror_bus_address = address & 0b00100000_00000111;
                todo!("PPU NOT SUPPORTED YET")
            }
            0x8000..=0xFFFF => self.read_prg_rom(address),
            _ => {
                println!("Ignoring memory address as {:?}", address);
                0
            }
            
        }
    }

    fn mem_write(&mut self, address: u16, data: u8) {
        match address {
            RAM_ADDRESS ..= RAM_END_ADDRESS => {
                let mirror_bus_address = address & 0b11111111111;
                self.cpu_vram[mirror_bus_address as usize] = data;
            }
            PPU_REGISTERS_ADDRESS ..= PPU_REGISTERS_END_ADDRESS => {
                let mirror_bus_address = address & 0b00100000_00000111;
                todo!("PPU NOT SUPPORTED YET");
            }
            0x8000..=0xFFFF => {
                panic!("Attempt to write to Cartridge ROM space")
            }
            _ => {
                println!("Ignoring memory write-access attempt at {:?}", address);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cartridge::test;

    #[test]
    fn test_mem_read_write_to_ram() {
        let mut bus = Bus::new(test::test_rom());
        bus.mem_write(0x01, 0x55);
        assert_eq!(bus.mem_read(0x01), 0x55);
    }
}