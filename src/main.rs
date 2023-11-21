use anyhow::{Context, Result};
use binrw::{BinRead, BinReaderExt};

use std::{
    env,
    fmt::Debug,
    fs,
    io::Cursor,
    str::{self, Utf8Error},
};

// TODO: smol_str for header fields
type StrResult = std::result::Result<String, Utf8Error>;

fn null_str(bytes: &[u8]) -> StrResult {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    str::from_utf8(&bytes[..end]).map(str::to_owned)
}

fn fixed_string<const N: usize>(bytes: [u8; N]) -> StrResult {
    str::from_utf8(&bytes).map(str::to_owned)
}

// http://problemkaputt.de/gbatek-ds-cartridge-header.htm
#[derive(Clone, Debug, BinRead)]
#[br(assert(self.logo_crc == 0xCF56))]
struct NdsHeader {
    #[br(try_map = |bytes: [u8; 12]| null_str(&bytes))]
    title: String,
    #[br(try_map = fixed_string::<4>)]
    game_code: String,
    #[br(try_map = fixed_string::<2>)]
    maker_code: String,
    unit_code: u8,
    encryption_seed_select: u8,
    capacity: u8,
    #[br(pad_before = 8)]
    region: u8,
    rom_version: u8,
    internal_flags: u8, // (autostart)
    arm9: BinaryHeader,
    arm7: BinaryHeader,
    fnt: Slice,
    fat: Slice,
    arm9_overlay: Slice,
    arm7_overlay: Slice,
    cr_settings: u32,
    secure_cr_settings: u32,
    icon_banner_offset: u32,
    secure_area_crc: u16,
    secure_transfer_timeout: u16,
    arm9_autoload: u32,
    arm7_autoload: u32,
    secure_disable: u64,
    rom_size: u32,
    header_size: u32,
    #[br(pad_before = 212)]
    logo_crc: u16,
    header_crc: u16,
}

#[derive(Clone, BinRead)]
struct BinaryHeader {
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
struct Slice {
    offset: u32,
    size: u32,
}

impl Debug for Slice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[u8; 0x{:x}] @ 0x{:x}", self.size, self.offset)
    }
}

// https://www.romhacking.net/documents/[469]nds_formats.htm
fn main() -> Result<()> {
    let rom_path = env::args().nth(1).context("pass a ROM path")?;
    let rom = fs::read(rom_path).context("couldn't find ROM")?;
    let mut cursor = Cursor::new(rom);

    let header: NdsHeader = cursor.read_ne()?;
    println!("{:#x?}", &header);

    Ok(())
}
