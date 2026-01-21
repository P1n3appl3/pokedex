use std::{
    env, fs,
    io::{Cursor, Seek, SeekFrom::Start},
};

use anyhow::{Context, Result};
use binrw::BinReaderExt;

use pokedex::{NdsHeader, banner::Banner, fs::parse_nitro_fs};

fn main() -> Result<()> {
    let rom_path = env::args().nth(1).context("pass a ROM path")?;
    let rom = fs::read(rom_path).context("couldn't find ROM")?;
    let mut cursor = Cursor::new(rom);

    let header: NdsHeader = cursor.read_ne()?;
    println!("{:#x?}", &header);

    cursor.seek(Start(header.banner_offset as u64))?;
    let banner: Banner = cursor.read_ne()?;
    println!("{:#x?}", &banner);

    parse_nitro_fs(header.fnt, header.fat, &mut cursor)?;

    Ok(())
}
