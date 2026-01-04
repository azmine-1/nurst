mod bus; 

pub struct PPU {
    ctrl: u8,
    mask: u8,
    status: u8,
    oam_addr: u8,
    oam_data: u8,
    scroll: u8,
    addr: u8,
    data: u8,
    oam_dma: u8,
    io_db: Bus,
}
fn create_ppu_bus(ppu_registers: [u8; 8], empty_bus: Bus) -> Bus {
    let new_bus = Bus{
        ram: [0; 2048],
        ppu_registers
    } 
}
impl PPU {
    pub fn new() -> Self {
       Self {
            ctrl: 0x2000,
            mask: 0x2001, 
            status: 0x2002, 
            oam_addr: 0x2003, 
            oam_data: 0x2004, 
            scroll: 0x2005, 
            addr: 0x2006, 
            data: 0x2007, 
        } 
    }
}


