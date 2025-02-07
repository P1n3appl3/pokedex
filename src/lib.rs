pub mod banner;
pub mod fs;

use binrw::BinRead;
use smol_str::SmolStr;

use std::{ffi::CStr, fmt::Debug};

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
    Ok(std::str::from_utf8(&bytes).map(SmolStr::new)?)
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
