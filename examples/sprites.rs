use std::{env, fs};

use anyhow::{Context, Result};

use pokedex::{
    fs::NitroFs,
    sprite::{self, get_palettes},
};

fn main() -> Result<()> {
    let rom_path = env::args().nth(1).context("pass a ROM path")?;
    let rom = fs::read(rom_path).context("couldn't find ROM")?;
    let fs = NitroFs::new(&rom)?;

    let buf = &mut Vec::new();
    let palettes = get_palettes(&rom, &fs, buf);

    for i in 0..600 {
        if sprite::draw(i, false, palettes, &fs).is_err() {
            break;
        }
    }

    Ok(())
}
