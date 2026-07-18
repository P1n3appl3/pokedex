use std::{env, fs, io::Cursor};

use anyhow::{Context, Result};
use binrw::{BinRead, BinReaderExt};

use pokedex::{
    Narc,
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

#[derive(BinRead, Copy, Clone, Debug)]
struct Pokemon {
    base_hp: u8,
    attack: u8,
    defence: u8,
    speed: u8,
    special_attack: u8,
    special_defence: u8,
    type1: u8,
    type2: u8,
    catch_rate: u8,
    exp: u8,
    evs: u16,
    item1: u16,
    item2: u16,
    gender: u8,
    hatch_cycle: u8,
    base_happy: u8,
    exprate: u8,
    egg_group1: u8,
    egg_group2: u8,
    ability1: u8,
    ability2: u8,
    flee: u8,
    color: u8,
    // 92 tms, 8 hms
    tms: [u8; 13],
}
static_assertions::const_assert_eq!(std::mem::size_of::<Pokemon>(), 40);

#[derive(BinRead, Copy, Clone, Debug)]
struct Evo {
    method: u16,
    param: u16,
    target: u16,
}

#[derive(BinRead, Copy, Clone, Debug)]
struct Evos([Evo; 7]);
