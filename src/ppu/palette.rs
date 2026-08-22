/// The 64 NES colours as 0x00RRGGBB. Values are the widely used "FCEUX"
/// approximation of the 2C02's composite output.
pub const NES_PALETTE: [u32; 64] = [
    0x666666, 0x002A88, 0x1412A7, 0x3B00A4, 0x5C007E, 0x6E0040, 0x6C0600, 0x561D00,
    0x333500, 0x0B4800, 0x005200, 0x004F08, 0x00404D, 0x000000, 0x000000, 0x000000,
    0xADADAD, 0x155FD9, 0x4240FF, 0x7527FE, 0xA01ACC, 0xB71E7B, 0xB53120, 0x994E00,
    0x6B6D00, 0x388700, 0x0C9300, 0x008F32, 0x007C8D, 0x000000, 0x000000, 0x000000,
    0xFFFEFF, 0x64B0FF, 0x9290FF, 0xC676FF, 0xF36AFF, 0xFE6ECC, 0xFE8170, 0xEA9E22,
    0xBCBE00, 0x88D800, 0x5CE430, 0x45E082, 0x48CDDE, 0x4F4F4F, 0x000000, 0x000000,
    0xFFFEFF, 0xC0DFFF, 0xD3D2FF, 0xE8C8FF, 0xFBC2FF, 0xFEC4EA, 0xFECCC5, 0xF7D8A5,
    0xE4E594, 0xCFEF96, 0xBDF4AB, 0xB3F3CC, 0xB5EBF2, 0xB8B8B8, 0x000000, 0x000000,
];

/// Apply the PPUMASK greyscale bit and the three colour-emphasis bits.
pub fn apply_mask(color_index: u8, mask: u8) -> u32 {
    let index = if mask & 0x01 != 0 { color_index & 0x30 } else { color_index & 0x3F };
    let rgb = NES_PALETTE[index as usize];

    if mask & 0xE0 == 0 {
        return rgb;
    }

    // Emphasis attenuates the two channels that are *not* emphasised.
    let (mut r, mut g, mut b) = ((rgb >> 16) & 0xFF, (rgb >> 8) & 0xFF, rgb & 0xFF);
    let dim = |c: u32| (c as f32 * 0.75) as u32;
    if mask & 0x20 != 0 {
        g = dim(g);
        b = dim(b);
    }
    if mask & 0x40 != 0 {
        r = dim(r);
        b = dim(b);
    }
    if mask & 0x80 != 0 {
        r = dim(r);
        g = dim(g);
    }
    (r << 16) | (g << 8) | b
}
