// io.rs --- 
// 
// Filename: io.rs
// Author: Louise <louise>
// Created: Wed Jan  3 15:30:01 2018 (+0100)
// Last-Updated: Wed Jan 31 01:39:53 2018 (+0100)
//           By: Louise <louise>
//
use cpu::ARM7TDMI;
use gpu::GPU;
use apu::APU;
use keypad::Keypad;
use irq::IrqManager;

use byteorder::{ByteOrder, LittleEndian};
use std::fs::File;
use std::io::Read;

#[macro_use] mod macros;

use rgba_common::Platform;

pub struct Interconnect {
    bios: Vec<u8>,
    rom:  Vec<u8>,
    iram: [u8; 0x8000],
    eram: [u8; 0x40000],
    io: [u16; 0x200],

    gpu: GPU,
    apu: APU,
    pub keypad: Keypad,

    cycles_to_spend: u32,
    rom_len: usize,
    waitstates: [[[u32; 2]; 3]; 16],

    postflg: u8,
    irq: IrqManager,
}

impl Interconnect {
    pub fn new() -> Interconnect {
        Interconnect {
            bios: vec![],
            rom:  vec![],
            iram: [0; 0x8000],
            eram: [0; 0x40000],
            io:   [0; 0x200],
            gpu: GPU::new(),
            apu: APU::new(),
            keypad: Keypad::default(),

            cycles_to_spend: 0,
            rom_len: 0,
            waitstates: [
                [[1, 1], [1, 1], [1, 1]], // BIOS
                [[1, 1], [1, 1], [1, 1]],
                [[3, 3], [3, 3], [6, 6]], // ERAM
                [[1, 1], [1, 1], [1, 1]], // IRAM
                [[1, 1], [1, 1], [1, 1]], // IO
                [[1, 1], [1, 1], [2, 2]], // Palette RAM
                [[1, 1], [1, 1], [2, 2]], // VRAM
                [[1, 1], [1, 1], [1, 1]], // OAM
                [[5, 5], [5, 5], [8, 8]], // GamePak WaitState 0
                [[5, 5], [5, 5], [8, 8]],
                [[5, 5], [5, 5], [8, 8]], // GamePak WaitState 1
                [[5, 5], [5, 5], [8, 8]],
                [[5, 5], [5, 5], [8, 8]], // GamePak WaitState 2
                [[5, 5], [5, 5], [8, 8]],
                [[1, 1], [1, 1], [1, 1]], // GamePak SRAM
                [[1, 1], [1, 1], [1, 1]],
            ],

            postflg: 0,
            irq: IrqManager::new(),
        }
    }

    pub fn declare_access(&mut self, address: usize, width: usize) {
        self.cycles_to_spend += self.waitstates[address >> 24][width][0];
    }

    pub fn delay(&mut self, cycles: u32) {
        self.cycles_to_spend += cycles;
    }

    pub fn spend(&mut self, cpu: &mut ARM7TDMI) {
        self.gpu.spend_cycles(self.cycles_to_spend, &mut self.irq);
        self.irq.handle(cpu);
        
        self.cycles_to_spend = 0;
    }

    pub fn render<T: Platform>(&mut self, platform: &mut T) {
        self.gpu.render(platform);
    }
    
    pub fn read_u32(&self, address: usize) -> u32 {
        match address & 0x0F000000 {
            0x00000000 if address < 0x4000 =>
                LittleEndian::read_u32(&self.bios[address..]),
            0x02000000 =>
                LittleEndian::read_u32(&self.eram[(address & 0x3ffff)..]),
            0x03000000 =>
                LittleEndian::read_u32(&self.iram[(address & 0x7fff)..]),
            0x04000000 => self.io_read_u32(address),
            0x05000000 => self.gpu.pram_read_u32(address),
            0x06000000 => self.gpu.vram_read_u32(address),
            0x07000000 => self.gpu.oam_read_u32(address),
            0x08000000 |
            0x09000000 |
            0x0A000000 |
            0x0B000000 |
            0x0C000000 |
            0x0D000000 => if (address & 0x01FFFFFF) < self.rom_len {
                LittleEndian::read_u32(&self.rom[(address & 0x01FFFFFF)..])
            } else {
                unused_pattern!(address, 32) as u32
            },
            _ => { warn!("Unmapped read_u32 from {:08x}", address); 0 },
        }
    }

    pub fn read_u16(&self, address: usize) -> u16 {
        match address & 0x0F000000 {
            0x00000000 if address < 0x4000 =>
                LittleEndian::read_u16(&self.bios[address..]),
            0x03000000 =>
                LittleEndian::read_u16(&self.iram[(address & 0x7fff)..]),
            0x04000000 => self.io_read_u16(address),
            0x05000000 => self.gpu.pram_read_u16(address),
            0x06000000 => self.gpu.vram_read_u16(address),
            0x07000000 => self.gpu.oam_read_u16(address),
            0x08000000 |
            0x09000000 |
            0x0A000000 |
            0x0B000000 |
            0x0C000000 |
            0x0D000000 => if (address & 0x01FFFFFF) < self.rom_len {
                LittleEndian::read_u16(&self.rom[(address & 0x01FFFFFF)..])
            } else {
                unused_pattern!(address, 16) as u16
            },
            _ => { warn!("Unmapped read_u16 from {:08x}", address); 0 },
        }
    }

    pub fn read_u8(&self, address: usize) -> u8 {
        match address & 0x0F000000 {
            0x00000000 if address < 0x4000 => self.bios[address],
            0x03000000 => self.iram[address & 0x7fff],
            0x04000000 => self.io_read_u8(address),
            0x08000000 |
            0x09000000 |
            0x0A000000 |
            0x0B000000 |
            0x0C000000 |
            0x0D000000 => if (address & 0x01FFFFFF) < self.rom_len {
                self.rom[address & 0x01FFFFFF]
            } else {
                unused_pattern!(address, 8) as u8
            }
            _ => { warn!("Unmapped read_u8 from {:08x}", address); 0 },
        }
    }

    fn io_read_u32(&self, address: usize) -> u32 {
        match address {
            _ => {
                (self.io_read_u16(address & 0xFFFFFFFC) as u32)
                    | ((self.io_read_u16((address & 0xFFFFFFFE) | 2) as u32) << 16)
            }
        }
    }
    
    fn io_read_u16(&self, address: usize) -> u16 {
        match address {
            KEYINPUT => self.keypad.as_register(),
            IE => self.irq.i_e,
            IF => self.irq.i_f,
            0x04000000...0x04000056 => self.gpu.io_read_u16(address),
            0x04000060...0x040000A8 => self.apu.io_read_u16(address),
            _ => { warn!("Unmapped read_u16 from {:08x} (IO)", address); 0 }
        }
    }
    
    fn io_read_u8(&self, address: usize) -> u8 {
        match address {
            POSTFLG => self.postflg,
            _ => (self.io_read_u16(address & 0xFFFFFFFE) >> ((address & 0x1) << 3)) as u8,
        }
    }

    pub fn write_u32(&mut self, address: usize, value: u32) {
        match address & 0x0F000000 {
            0x00000000 if address < 0x4000 => warn!("Ignored write to BIOS"),
            0x02000000 => LittleEndian::write_u32(
                &mut self.eram[(address & 0x3ffff)..], value
            ),
            0x03000000 => LittleEndian::write_u32(
                &mut self.iram[(address & 0x7fff)..], value
            ),
            0x04000000 => self.io_write_u32(address, value),
            0x05000000 => self.gpu.pram_write_u32(address, value),
            0x06000000 => self.gpu.vram_write_u32(address, value),
            0x07000000 => self.gpu.oam_write_u32(address, value),
            _ => warn!("Unmapped write_u32 to {:08x} (value={:08x})", address, value)
        }
    }

    pub fn write_u16(&mut self, address: usize, value: u16) {
        match address & 0x0F000000 {
            0x00000000 if address < 0x4000 => warn!("Ignored write to BIOS"),
            0x02000000 => LittleEndian::write_u16(
                &mut self.eram[(address & 0x3ffff)..], value
            ),
            0x03000000 => LittleEndian::write_u16(
                &mut self.iram[(address & 0x7fff)..], value
            ),
            0x04000000 => self.io_write_u16(address, value),
            0x05000000 => self.gpu.pram_write_u16(address, value),
            0x06000000 => self.gpu.vram_write_u16(address, value),
            0x07000000 => self.gpu.oam_write_u16(address, value),
            _ => warn!("Unmapped write_u16 to {:08x} (value={:04x})", address, value),
        }
    }
    
    pub fn write_u8(&mut self, address: usize, value: u8) {
        match address & 0x0F000000 {
            0x00000000 if address < 0x4000 => warn!("Ignored write to BIOS"),
            0x03000000 => self.iram[address & 0x7fff] = value,
            0x04000000 => self.io_write_u8(address, value),
            _ => warn!("Unmapped write_u8 to {:08x} (value={:02x})", address, value),
        }
    }

    fn io_write_u32(&mut self, address: usize, value: u32) {
        match address {
            _ => {
                self.io_write_u16(address, value as u16);
                self.io_write_u16(address | 2, (value >> 16) as u16);
            }
        }
    }
    
    fn io_write_u16(&mut self, address: usize, value: u16) {
        self.io[(address & 0x3FF) >> 1] = value;
        
        match address {
            0x04000000...0x04000056 => self.gpu.io_write_u16(address, value),
            0x04000060...0x040000A8 => self.apu.io_write_u16(address, value),

            // Ignore DMA writes for now
            // 0x040000c6 => { }
            // 0x040000d2 => { }
            
            IE => self.irq.i_e = value,
            IF => self.irq.write_if(value),
            IME => self.irq.write_ime(value),
            _ => warn!("Unmapped write_u16 to {:08x} (IO, value={:04x})", address, value),
        }
    }
    
    fn io_write_u8(&mut self, address: usize, value: u8) {
        match address {
            HALTCNT => self.irq.halt = true,
            POSTFLG => self.postflg = value,
            _ => {
                let value16 = ((value as u16) << ((address & 1) << 3))
                    | (self.io[(address & 0x3FF) >> 1] & !(0xFF << ((address & 1) << 3)));
                
                self.io_write_u16(address & 0xFFFFFFFE, value16);
            }
        }
    }

    // IRQ
    #[inline]
    pub fn halt(&self) -> bool { self.irq.halt }

    // Frame
    #[inline]
    pub fn is_frame(&self) -> bool { self.gpu.is_frame() }
    pub fn ack_frame(&mut self) { self.gpu.ack_frame(); }
    
    pub fn load_bios(&mut self, filename: &str) -> Result<(), &'static str> {
        match File::open(filename) {
            Ok(mut file) => {    
                info!("BIOS file opened");
            
                if let Err(e) = file.read_to_end(&mut self.bios) {
                    error!("Error reading BIOS file : {}", e);
                    Err("Error reading BIOS")
                } else {
                    Ok(())
                }
            }

            Err(e) => {
                error!("Couldn't load BIOS : {}", e);
                Err("Error opening BIOS file")
            }
        }
    }

    pub fn load_rom(&mut self, filename: &str) -> bool {
        match File::open(filename) {
            Ok(mut file) => {
                info!("ROM file opened");

                if let Err(e) = file.read_to_end(&mut self.rom) {
                    error!("Error reading ROM file : {}", e);
                    false
                } else {
                    self.rom_len = self.rom.len();
                    true
                }
            }
            Err(e) => {
                error!("Couldn't open ROM file : {}", e);
                false
            }
        }
    }
}

const KEYINPUT: usize = 0x04000130;
const IE:       usize = 0x04000200;
const IF:       usize = 0x04000202;
const IME:      usize = 0x04000208;
const POSTFLG:  usize = 0x04000300;
const HALTCNT:  usize = 0x04000301;
