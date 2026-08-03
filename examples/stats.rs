use std::{env, fs, io::Cursor};

use anyhow::{Context, Result};
use binrw::{BinRead, BinReaderExt};

use pokedex::{
    Evos, Narc, Pokemon,
    fs::{FsEntry, NitroFs},
};

fn main() -> Result<()> {
    let rom_path = env::args().nth(1).context("pass a ROM path")?;
    let rom = fs::read(rom_path).context("couldn't find ROM")?;

    let fs = NitroFs::new(&rom)?;
    let game = fs.header.game()?;
    let Some(FsEntry::File(contents)) = fs.get(game.stats().into()) else {
        panic!("personal.narc not found");
    };

    let mut c = Cursor::new(contents);
    let narc: Narc = c.read_le().unwrap();
    let img_start = c.position() as u32 + narc.fnt_size;
    for (i, f) in narc.fat.iter().enumerate() {
        let entry = &contents[(img_start + f.start) as usize..(img_start + f.end) as usize];
        let mut c = Cursor::new(entry);
        println!("{i}: {}", pretty_hex::simple_hex(&entry));
        let pokemon: Pokemon = c.read_le().unwrap();
        println!("\x1b[1m===== {i} {:#?}\x1b[0m", pokemon);
    }

    let Some(FsEntry::File(contents)) = fs.get(game.base_forms().into()) else {
        panic!("pms.narc not found");
    };
    for (i, j) in contents.chunks_exact(2).enumerate() {
        println!("pokemon {i} base form is {}", u16::from_le_bytes(*j.as_array().unwrap()));
    }

    let Some(FsEntry::File(contents)) = fs.get(game.evolutions().into()) else {
        panic!("evo.narc not found");
    };

    let mut c = Cursor::new(contents);
    let narc: Narc = c.read_le().unwrap();
    let img_start = c.position() as u32 + narc.fnt_size;
    for (i, f) in narc.fat.iter().enumerate() {
        let entry = &contents[(img_start + f.start) as usize..(img_start + f.end) as usize];
        let mut c = Cursor::new(entry);
        println!("{i}: {}", pretty_hex::simple_hex(&entry));
        let evos: Evos = c.read_le().unwrap();
        println!("\x1b[1m===== {i} {:#?}\x1b[0m", evos.0);
    }

    Ok(())
}
