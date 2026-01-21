pub mod banner;
pub mod charset;
pub mod fs;

use std::{
    fmt::{self, Debug, Formatter},
    mem,
    str::Utf8Error,
    string::FromUtf16Error,
};

use binrw::{BinRead, NullString, NullWideString};
use fs::{FatEntry, FileAllocationTable};
use smol_str::SmolStr;

// https://problemkaputt.de/gbatek-ds-cartridge-header.htm
#[derive(Clone, Debug, BinRead)]
#[br(assert(self.logo_crc == 0xCF56))]
pub struct NdsHeader {
    #[br(pad_size_to = 12, try_map = smol_str)]
    pub title: SmolStr,
    #[br(try_map = fixed_str::<4>)]
    pub game_code: SmolStr,
    #[br(try_map = fixed_str::<2>)]
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

#[derive(Clone, Debug, BinRead)]
#[br(magic = b"NARC")]
pub struct Narc {
    #[br(assert(byte_order == 0xFFFE))]
    byte_order: u16,
    #[br(assert(version == 0x0100))]
    version: u16,
    file_size: u32,
    #[br(assert(chunk_size == 16))]
    chunk_size: u16,
    #[br(assert(num_chunks == 3))]
    num_chunks: u16,
    #[br(magic = b"BTAF")] // chunk name (file allocation table block)
    fat_size: u32, // (including ^ and this field, so -8 for actual)
    num_files: u16,
    #[br(magic = 0u16)] // reserved
    #[br(count = num_files)]
    fat: FileAllocationTable,
    #[br(magic = b"BTNF")] // chunk name (file name table block)
    fnt_size: u32, // (including ^ and this field, so -8 for actual)

    // #[br(align_after = 4)]
    fnt: (),
    // #[br(magic = b"GMIF")] // chunk name (file image block)
    // img_size: u32, // (including ^ and this field, so -8 for actual)
    // img_data: (),
}

fn smol_str(s: NullString) -> Result<SmolStr, Utf8Error> {
    Ok(SmolStr::new(std::str::from_utf8(&s)?))
}

fn fixed_str<const N: usize>(s: [u8; N]) -> Result<SmolStr, Utf8Error> {
    Ok(SmolStr::new(std::str::from_utf8(&s)?))
}

pub fn wide_smol_str(s: NullWideString) -> Result<SmolStr, FromUtf16Error> {
    Ok(SmolStr::new(String::from_utf16(&s)?))
}

#[derive(Copy, Clone, BinRead)]
pub struct BinaryHeader {
    pub rom_offset: u32,
    pub entry_addr: u32,
    pub load_addr: u32,
    pub size: u32,
}

impl Debug for BinaryHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[u8; 0x{:x}] @ 0x{:x} -> 0x{:x} (entry: 0x{:x})",
            self.size,
            self.rom_offset,
            self.load_addr,
            self.entry_addr - self.load_addr
        )
    }
}

#[derive(Copy, Clone, BinRead)]
pub struct Slice {
    pub offset: u32,
    pub size: u32,
}

impl Debug for Slice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[u8; 0x{:x}] @ 0x{:x}", self.size, self.offset)
    }
}
