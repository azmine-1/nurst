use crate::apu::APU;
use crate::input::Controller;
use crate::mapper::Mapper;
use crate::ppu::PPU;

const RAM_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_END: u16 = 0x3FFF;
const OAM_DMA: u16 = 0x4014;
const CONTROLLER1: u16 = 0x4016;
const CONTROLLER2: u16 = 0x4017;

pub struct Bus {
    ram: [u8; 2048],
    pub ppu: PPU,
    pub apu: APU,
    pub mapper: Box<dyn Mapper>,
    pub controllers: [Controller; 2],

    stall_cycles: u32,
    pub cycles: u64,
    instruction_cycles: u32,
    open_bus: u8,
}

impl Bus {
    pub fn new(mapper: Box<dyn Mapper>, sample_rate: u32) -> Self {
        Self {
            ram: [0; 2048],
            ppu: PPU::new(),
            apu: APU::new(sample_rate),
            mapper,
            controllers: [Controller::new(), Controller::new()],
            stall_cycles: 0,
            cycles: 0,
            instruction_cycles: 0,
            open_bus: 0,
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        self.tick(1);
        self.read_untimed(addr)

    pub fn write(&mut self, addr: u16, data: u8) {
        self.tick(1);
        self.write_untimed(addr, data);
    }

    fn read_untimed(&mut self, addr: u16) -> u8 {
        let value = match addr {
            0x0000..=RAM_END => self.ram[(addr & 0x07FF) as usize],
            PPU_REGISTERS..=PPU_REGISTERS_END => self.ppu.cpu_read(addr, self.mapper.as_mut()),
            0x4015 => self.apu.read_status(),
            CONTROLLER1 => (self.open_bus & 0xE0) | self.controllers[0].read(),
            CONTROLLER2 => (self.open_bus & 0xE0) | self.controllers[1].read(),
            0x4000..=0x401F => self.open_bus, 
            _ => self.mapper.cpu_read(addr),
        };
        self.open_bus = value;
        value
    }

    fn write_untimed(&mut self, addr: u16, data: u8) {
        self.open_bus = data;
        match addr {
            0x0000..=RAM_END => self.ram[(addr & 0x07FF) as usize] = data,
            PPU_REGISTERS..=PPU_REGISTERS_END => {
                self.ppu.cpu_write(addr, data, self.mapper.as_mut())
            }
            OAM_DMA => self.oam_dma(data),
            CONTROLLER1 => {
                self.controllers[0].write_strobe(data);
                self.controllers[1].write_strobe(data);
            }
            0x4000..=0x4013 | 0x4015 | CONTROLLER2 => self.apu.write(addr, data),
            0x4018..=0x401F => {}
            _ => self.mapper.cpu_write(addr, data),
        }
    }

    pub fn peek(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=RAM_END => self.ram[(addr & 0x07FF) as usize],
            PPU_REGISTERS..=PPU_REGISTERS_END => self.ppu.peek(addr),
            0x4000..=0x401F => 0xFF,
            _ => self.mapper.cpu_read(addr),
        }
    }

    pub fn peek_u16(&mut self, addr: u16) -> u16 {
        (self.peek(addr.wrapping_add(1)) as u16) << 8 | self.peek(addr) as u16
    }

    pub fn read_u16_zeropage(&mut self, pos: u8) -> u16 {
        let lo = self.read(pos as u16);
        let hi = self.read(pos.wrapping_add(1) as u16);
        (hi as u16) << 8 | lo as u16
    }

    pub fn read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.read(pos);
        let hi = self.read(pos.wrapping_add(1));
        (hi as u16) << 8 | lo as u16
    }

    fn oam_dma(&mut self, page: u8) {
        let base = (page as u16) << 8;
        for offset in 0..256u16 {
            let value = self.read_untimed(base + offset);
            self.ppu.write_oam_dma(value);
        }
        self.stall_cycles += 513 + (self.cycles % 2) as u32;
    }

    pub fn take_stall_cycles(&mut self) -> u32 {
        std::mem::take(&mut self.stall_cycles)
    }

    pub fn tick(&mut self, cpu_cycles: u32) {
        for _ in 0..cpu_cycles {
            self.cycles += 1;
            self.instruction_cycles += 1;
            for _ in 0..3 {
                self.ppu.tick(self.mapper.as_mut());
            }
            self.apu.tick();
            if let Some(addr) = self.apu.dmc_fetch_address() {
                let sample = self.read_untimed(addr);
                self.apu.dmc_supply_sample(sample);
                // The DMC steals CPU cycles for its fetch.
                self.stall_cycles += 4;
            }
        }
    }

    pub fn begin_instruction(&mut self) {
        self.instruction_cycles = 0;
    }

    pub fn finish_instruction(&mut self, total: u32) {
        self.tick(total.saturating_sub(self.instruction_cycles));
    }

    pub fn poll_nmi(&mut self) -> bool {
        self.ppu.take_nmi()
    }

    pub fn poll_irq(&self) -> bool {
        self.apu.irq_pending() || self.mapper.irq_pending()
    }
}
