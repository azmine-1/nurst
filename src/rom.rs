const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A]; // "NES" + MS-DOS EOF
const PRG_ROM_PAGE_SIZE: usize = 16384; // 16 KB
const CHR_ROM_PAGE_SIZE: usize = 8192;  // 8 KB

#[derive(Debug, PartialEq)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub mirroring: Mirroring,
}

impl Rom {
    pub fn new(raw: &[u8]) -> Result<Rom, String> {
        // Check header signature
        if &raw[0..4] != NES_TAG {
            return Err("File is not in iNES format".to_string());
        }

        let mapper = (raw[7] & 0b1111_0000) | (raw[6] >> 4);

        let ines_ver = (raw[7] >> 2) & 0b11;
        if ines_ver != 0 {
            return Err("NES2.0 format is not supported".to_string());
        }

        let four_screen = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 0b1 != 0;
        let mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        let prg_rom_size = raw[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_PAGE_SIZE;

        let skip_trainer = raw[6] & 0b100 != 0;

        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        Ok(Rom {
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec(),
            mapper,
            mirroring,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rom_creation() {
        let test_rom = vec![
            0x4E, 0x45, 0x53, 0x1A, // NES\x1a
            0x02, // 2 * 16KB PRG ROM
            0x01, // 1 * 8KB CHR ROM
            0x01, // Mapper 0, vertical mirroring
            0x00, // Mapper 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Padding
        ];

        let mut rom_data = test_rom.clone();
        rom_data.extend(vec![0; 2 * PRG_ROM_PAGE_SIZE]);
        rom_data.extend(vec![0; 1 * CHR_ROM_PAGE_SIZE]);

        let rom = Rom::new(&rom_data).unwrap();

        assert_eq!(rom.prg_rom.len(), 2 * PRG_ROM_PAGE_SIZE);
        assert_eq!(rom.chr_rom.len(), 1 * CHR_ROM_PAGE_SIZE);
        assert_eq!(rom.mapper, 0);
        assert_eq!(rom.mirroring, Mirroring::Vertical);
    }
}
