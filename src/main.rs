use anyhow::{anyhow, Context, Result};
use nds::{
    parser::{NDSParser, Table},
    Extractor,
};
use nitro_fs::FileSystem;

use std::{env, ffi::OsStr, fs, path::Path};

// upsamples from the 5 bit color channels used on the DS to 8 bit
fn upsample_color(x: u8) -> u8 {
    assert!(x < 32);
    (x << 3) + (x >> 2)
}

fn list_narcs(rom: &Vec<u8>) -> Result<()> {
    let header = NDSParser::try_from(rom).context("failed to parse ROM")?;
    let get_table = |t: Table| &rom[t.offset as usize..(t.offset + t.length) as usize];
    let fs = FileSystem::new(get_table(header.fnt), get_table(header.fat))
        .context("couldn't read filesystem")?;
    println!("found {} files:", fs.count());
    for f in fs.files() {
        println!(
            "{}: {:<60} {}",
            f.id,
            f.path.to_string_lossy(),
            f.alloc.end - f.alloc.start
        );
    }
    Ok(())
}

// `ndstool -x diamond.nds -d extracted`
fn extract_rom(p: &str, out: &str) -> Result<()> {
    let rom = Extractor::new(p, true).context("couldn't find ROM")?;
    rom.extract(out)?;
    Ok(())
}

fn get_icon_narc(rom: &Vec<u8>) -> Result<&[u8]> {
    let header = NDSParser::try_from(rom).context("failed to parse ROM")?;
    let get_table = |t: Table| &rom[t.offset as usize..(t.offset + t.length) as usize];
    let fs = FileSystem::new(get_table(header.fnt), get_table(header.fat))
        .context("couldn't read filesystem")?;
    for f in fs.files() {
        if let Some("poke_icon.narc") = f.path.file_name().and_then(OsStr::to_str) {
            return Ok(&rom[f.alloc.start as usize..f.alloc.end as usize]);
        }
    }
    Err(anyhow!("couldn't find poke_icon.narc"))
}

fn main() -> Result<()> {
    // let half_block = '▄';
    let rom_path = env::args().nth(1).context("pass a ROM path")?;
    // extract_rom(&rom_path, "dump")?;
    let rom = fs::read(&rom_path).context("couldn't find ROM")?;

    // list_narcs(&rom)?;

    let icon_data = get_icon_narc(&rom)?;
    println!("{}", icon_data.len());

    Ok(())
}
