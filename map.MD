# NES Emulator Project Map

## Chapter  3: Writing NES Emulator in Rust

###  3.1. Getting Started ☑️

- Setting up the development environment for Rust
- Understanding the basics of NES architecture

###  3.2. Memory Addressing Modes

- Implementing  16-bit memory addressing
- Accessing only registers
- Accessing the first  255 bytes of RAM
- Accessing memory space after the first  255 (4-7 CPU cycles)

###  3.3. The Rest of the Instructions

- Implementing support for  256 different machine instructions
- Reusing foundational instructions for the entire set
- Handling unofficial additional opcodes

###  3.4. Running Our First Game

- Playing the Snake game as a test case
- Integrating the CPU with PPU and APU
- Executing commands issued by the CPU

###  6.1. Emulating PPU Registers

- Implementing PPU registers for graphics rendering
- Managing PPU memory addresses and data

###  6.2. Emulating NMI Interrupt

- Handling Non-Maskable Interrupts (NMIs) for screen updates
- Ensuring smooth visuals during gameplay

###  6.3. Rendering CHR Rom Tiles

- Parsing and rendering character ROM tiles
- Implementing sprite and background layers

###  6.4. Rendering Static Screen

- Displaying static elements on the screen
- Ensuring proper timing and synchronization with the CPU

## Appendix

- Notes on the NES console's custom chip  2A03
- Audio Processing Unit (APU) integration
- Decimal mode handling considerations

