use std::fmt::{self, Debug, Formatter, Write};

use binrw::BinRead;
use owo_colors::{OwoColorize, Rgb};
use smol_str::SmolStr;

use crate::{Color, wide_smol_str};

// https://problemkaputt.de/gbatek-ds-cartridge-icon-title.htm
#[derive(Clone, Debug, BinRead)]
pub struct Banner {
    pub version: u16,
    pub crc: u16, // checksum of the rest of this struct, not including first 32 bytes
    #[br(pad_before(28))]
    pub icon: Icon,
    #[br(pad_size_to = 256, try_map = wide_smol_str)]
    pub jp: SmolStr,
    #[br(pad_size_to = 256, try_map = wide_smol_str)]
    pub en: SmolStr,
    #[br(pad_size_to = 256, try_map = wide_smol_str)]
    pub fr: SmolStr,
    #[br(pad_size_to = 256, try_map = wide_smol_str)]
    pub de: SmolStr,
    #[br(pad_size_to = 256, try_map = wide_smol_str)]
    pub it: SmolStr,
    #[br(pad_size_to = 256, try_map = wide_smol_str)]
    pub es: SmolStr,
}

// https://gbatemp.net/threads/reading-the-nds-icon-data.45360/#post-9167077
#[derive(Clone, BinRead)]
pub struct Icon {
    pub tile_data: [[Tile; 4]; 4],
    pub palette: [Color; 16], // 0 is transparent so it's ignored
}

// each entry is a nibble which indexes into the 16-color palette
type Tile = [[u8; 4]; 8];

impl Icon {
    fn get(&self, x: usize, y: usize) -> Color {
        let sh = x & 1;
        let px = (self.tile_data[y >> 3][x >> 3][y & 0b111][(x & 0b110) >> 1]
            & (0xF << (4 * sh)))
            >> (4 * sh);
        Color(self.palette[px as usize].0 | (if px == 0 { 0x80 } else { 0 }))
    }
}

const UPPER: char = '▀';
const LOWER: char = '▄';

fn scale_5to8(x: u8) -> u8 {
    assert!(x <= 0x1F);
    (x << 3) | (x >> 2)
}

impl Color {
    fn b(self) -> u8 {
        scale_5to8((self.0 >> 10) as u8)
    }
    fn g(self) -> u8 {
        scale_5to8(((self.0 >> 5) & 0x1F) as u8)
    }
    fn r(self) -> u8 {
        scale_5to8((self.0 & 0x1F) as u8)
    }
}

impl From<Color> for Rgb {
    fn from(c: Color) -> Self {
        Self(c.r(), c.g(), c.b())
    }
}

impl Debug for Icon {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_char('\n')?;
        for y in 0..16 {
            for x in 0..32 {
                let fg = self.get(x, y * 2);
                let bg = self.get(x, y * 2 + 1);
                write!(f, "{}", UPPER.color::<Rgb>(fg.into()).on_color::<Rgb>(bg.into()))?;
            }
            f.write_char('\n')?;
        }
        if !f.alternate() {
            f.write_char('\n')?;
            for c in self.palette {
                write!(f, "{}", "  ".on_color::<Rgb>(c.into()))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scale() {
        assert_eq!(scale_5to8(0), 0);
        assert_eq!(scale_5to8(0x1F), 0xFF);
    }
}
