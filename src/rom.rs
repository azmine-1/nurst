const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A]; // "NES" + MS-DOS EOF
const PRG_ROM_PAGE_SIZE: usize = 16384; // 16 KB
const CHR_ROM_PAGE_SIZE: usize = 8192; // 8 KB

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
    SingleScreenLower,
    SingleScreenUpper,
}

pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub mirroring: Mirroring,
    /// Cartridge shipped without CHR ROM, so `chr_rom` is really 8 KB of RAM.
    pub chr_ram: bool,
    /// Cartridge has battery-backed PRG RAM at $6000-$7FFF.
    pub battery: bool,
}

impl Rom {
    pub fn new(raw: &[u8]) -> Result<Rom, String> {
        if raw.len() < 16 || raw[0..4] != NES_TAG {
            return Err("File is not in iNES format".to_string());
        }

        let mapper = (raw[7] & 0b1111_0000) | (raw[6] >> 4);
        let ines_ver = (raw[7] >> 2) & 0b11;

        let four_screen = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 0b1 != 0;
        let mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        // NES 2.0 stores the high nibble of each size in byte 9.
        let (prg_pages, chr_pages) = if ines_ver == 2 {
            (
                raw[4] as usize | ((raw[9] as usize & 0x0F) << 8),
                raw[5] as usize | ((raw[9] as usize & 0xF0) << 4),
            )
        } else {
            (raw[4] as usize, raw[5] as usize)
        };

        let prg_rom_size = prg_pages * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = chr_pages * CHR_ROM_PAGE_SIZE;

        let skip_trainer = raw[6] & 0b100 != 0;
        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        if raw.len() < chr_rom_start + chr_rom_size {
            return Err(format!(
                "ROM is truncated: header promises {} bytes but file holds {}",
                chr_rom_start + chr_rom_size,
                raw.len()
            ));
        }

        let chr_ram = chr_rom_size == 0;
        let chr_rom = if chr_ram {
            vec![0; CHR_ROM_PAGE_SIZE]
        } else {
            raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec()
        };

        Ok(Rom {
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom,
            mapper,
            mirroring,
            chr_ram,
            battery: raw[6] & 0b10 != 0,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_rom_bytes() -> Vec<u8> {
        let mut rom_data = vec![
            0x4E, 0x45, 0x53, 0x1A, // NES\x1a
            0x02, // 2 * 16KB PRG ROM
            0x01, // 1 * 8KB CHR ROM
            0x01, // Mapper 0, vertical mirroring
            0x00, // Mapper 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Padding
        ];
        rom_data.extend(vec![0; 2 * PRG_ROM_PAGE_SIZE]);
        rom_data.extend(vec![0; CHR_ROM_PAGE_SIZE]);
        rom_data
    }

    #[test]
    fn test_rom_creation() {
        let rom = Rom::new(&test_rom_bytes()).unwrap();

        assert_eq!(rom.prg_rom.len(), 2 * PRG_ROM_PAGE_SIZE);
        assert_eq!(rom.chr_rom.len(), CHR_ROM_PAGE_SIZE);
        assert_eq!(rom.mapper, 0);
        assert_eq!(rom.mirroring, Mirroring::Vertical);
        assert!(!rom.chr_ram);
    }

    #[test]
    fn missing_chr_rom_becomes_chr_ram() {
        let mut raw = test_rom_bytes();
        raw[5] = 0;
        raw.truncate(16 + 2 * PRG_ROM_PAGE_SIZE);

        let rom = Rom::new(&raw).unwrap();
        assert!(rom.chr_ram);
        assert_eq!(rom.chr_rom.len(), CHR_ROM_PAGE_SIZE);
    }

    #[test]
    fn truncated_file_is_rejected() {
        let raw = test_rom_bytes()[..100].to_vec();
        assert!(Rom::new(&raw).is_err());
    }
}
