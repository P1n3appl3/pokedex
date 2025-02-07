#![allow(unused)]
use binrw::BinRead;
use owo_colors::{OwoColorize, Rgb};
use smol_str::SmolStr;

use std::{
    ffi::{CStr, FromBytesUntilNulError},
    fmt::{self, Debug, Formatter, Write},
    str::{self, Utf8Error},
};

// https://problemkaputt.de/gbatek-ds-cartridge-header.htm
#[derive(Clone, Debug, BinRead)]
#[br(assert(self.logo_crc == 0xCF56))]
pub struct NdsHeader {
    #[br(try_map = null_str::<12>)]
    pub title: SmolStr,
    #[br(try_map = fixed_string::<4>)]
    pub game_code: SmolStr,
    #[br(try_map = fixed_string::<2>)]
    pub maker_code: SmolStr,
    pub unit_code: u8,
    pub encryption_seed_select: u8,
    pub capacity: u8,
    #[br(pad_before = 8)]
    pub region: u8,
    pub rom_version: u8,
    pub internal_flags: u8, // (autostart)
    pub arm9: BinaryHeader,
    pub arm7: BinaryHeader,
    pub fnt: Slice,
    pub fat: Slice,
    pub arm9_overlay: Slice,
    pub arm7_overlay: Slice,
    pub cr_settings: u32, // "card control register"
    pub secure_cr_settings: u32,
    pub banner_offset: u32,
    pub secure_area_crc: u16,
    pub secure_transfer_timeout: u16,
    pub arm9_autoload: u32,
    pub arm7_autoload: u32,
    pub secure_disable: u64,
    pub rom_size: u32,
    pub header_size: u32,
    #[br(pad_before = 212)] // 56 for reserved data, 156 for nintendo logo
    pub logo_crc: u16,
    pub header_crc: u16,
}

// https://problemkaputt.de/gbatek-ds-cartridge-icon-title.htm
#[derive(Clone, Debug, BinRead)]
pub struct Banner {
    pub version: u16,
    pub crc: u16, // checksum of the rest of this struct, not including first 32 bytes
    #[br(pad_before(28))]
    pub icon: Icon,
    #[br(try_map = u16_string::<128>)]
    pub jp: SmolStr,
    #[br(try_map = u16_string::<128>)]
    pub en: SmolStr,
    #[br(try_map = u16_string::<128>)]
    pub fr: SmolStr,
    #[br(try_map = u16_string::<128>)]
    pub de: SmolStr,
    #[br(try_map = u16_string::<128>)]
    pub it: SmolStr,
    #[br(try_map = u16_string::<128>)]
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

#[derive(Debug, Copy, Clone, BinRead)]
pub struct Color(u16); // bgr555, i'll use top bit for alpha?

impl Icon {
    fn get(&self, x: usize, y: usize) -> Color {
        let sh = /*1 -*/ (x & 1);
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

#[derive(Clone, BinRead)]
pub struct BinaryHeader {
    rom_offset: u32,
    entry_addr: u32,
    load_addr: u32,
    size: u32,
}

impl Debug for BinaryHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[u8; 0x{:x}] @ 0x{:x} -> {:x} ~ {:x}",
            self.size, self.rom_offset, self.entry_addr, self.load_addr
        )
    }
}

#[derive(Clone, BinRead)]
pub struct Slice {
    offset: u32,
    size: u32,
}

// #[derive(Debug, Clone, Error)]
// enum StrError {
//     Null(FromBytesUntilNulError),
//     Unicode(Utf8Error),
// }

type StrResult = anyhow::Result<SmolStr>;

fn null_str<const N: usize>(bytes: [u8; N]) -> StrResult {
    let cstr = CStr::from_bytes_until_nul(&bytes[..])?;
    Ok(SmolStr::new(cstr.to_str()?))
}

fn fixed_string<const N: usize>(bytes: [u8; N]) -> StrResult {
    Ok(str::from_utf8(&bytes).map(SmolStr::new)?)
}

fn u16_string<const N: usize>(bytes: [u16; N]) -> StrResult {
    let s = String::from_utf16(&bytes[..bytes.iter().position(|&b| b == 0).unwrap()])?;
    Ok(SmolStr::new(s.replace('\n', " | ")))
}

impl Debug for Slice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[u8; 0x{:x}] @ 0x{:x}", self.size, self.offset)
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
