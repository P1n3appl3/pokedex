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
    let Some(FsEntry::File(contents)) = fs.get("poketool/personal/personal.narc".into())
    else {
        panic!("personal.narc not found");
    };

    let mut c = Cursor::new(contents);
    let narc: Narc = c.read_ne().unwrap();
    let img_start = c.position() as u32 + narc.fnt_size;
    for (i, f) in narc.fat.iter().enumerate() {
        let entry = &contents[(img_start + f.start) as usize..(img_start + f.end) as usize];
        let mut c = Cursor::new(entry);
        let pokemon: Pokemon = c.read_ne().unwrap();
        println!("\x1b[1m===== {i} {:#?}\x1b[0m", pokemon);
    }

    Ok(())
}

#[derive(BinRead, Copy, Clone, Debug)]
#[repr(C)]
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
}
static_assertions::const_assert_eq!(std::mem::size_of::<Pokemon>(), 26);
