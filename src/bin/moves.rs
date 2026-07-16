use std::{env, fs, io::Cursor};

use anyhow::{Context, Result};
use binrw::{BinRead, BinReaderExt, helpers::until_exclusive};
use modular_bitfield::prelude::*;
use pokedex::{
    Narc,
    fs::{FsEntry, NitroFs},
};

fn main() -> Result<()> {
    let rom_path = env::args().nth(1).context("pass a ROM path")?;
    let rom = fs::read(rom_path).context("couldn't find ROM")?;

    let fs = NitroFs::new(&rom)?;
    let Some(FsEntry::File(contents)) = fs.get("poketool/personal/wotbl.narc".into())
    else {
        panic!("wotbl.narc not found");
    };
    let mut c = Cursor::new(contents);
    let narc: Narc = c.read_le().unwrap();
    let img_start = c.position() as u32 + narc.fnt_size;

    for (i, f) in narc.fat.iter().enumerate() {
        let entry = &contents[(img_start + f.start) as usize..(img_start + f.end) as usize];
        let mut c = Cursor::new(entry);
        let learnset: LearnSet = c.read_le().unwrap();
        println!("\x1b[1m===== {i}\x1b[0m");
        println!("{}", pretty_hex::simple_hex(&entry));
        for set in &learnset.moves {
            let lvl = set.level();
            let id = set.move_id();
            println!("lvl {lvl} (0x{lvl:02x}): learn {id} (0x{id:02x})");
        }
    }

    let Some(FsEntry::File(contents)) = fs.get("poketool/waza/waza_tbl.narc".into()) else {
        panic!("waza_tbl.narc not found");
    };
    let mut c = Cursor::new(contents);
    let narc: Narc = c.read_le().unwrap();
    let img_start = c.position() as u32 + narc.fnt_size;

    for (i, f) in narc.fat.iter().enumerate() {
        let entry = &contents[(img_start + f.start) as usize..(img_start + f.end) as usize];
        let mut c = Cursor::new(entry);
        let _move: Move = c.read_le().unwrap();
        println!("\x1b[1m===== {i} {:#?}\x1b[0m", _move);
    }
    Ok(())
}

#[derive(BinRead, Debug, Copy, Clone)]
pub struct Move {
    effect: u16,
    category: u8,
    power: u8,
    r#type: u8,
    accuracy: u8,
    pp: u8,
    effectchance: u8,
    flag1: u16,
    priority: u8,
    flag2: u8,
    contesteffect: u8,
    contesttype: u8,
}

#[bitfield]
#[derive(BinRead, Debug, Copy, Clone)]
#[br(map = Self::from_bytes)]
pub struct LearnMove {
    move_id: B9,
    level: B7,
}
static_assertions::const_assert_eq!(std::mem::size_of::<LearnMove>(), 2);

#[derive(BinRead, Debug, Clone)]
pub struct LearnSet {
    #[br(parse_with = until_exclusive(|x: &LearnMove| x.into_bytes() == 0xFFFFu16.to_be_bytes() ))]
    moves: Vec<LearnMove>,
}
