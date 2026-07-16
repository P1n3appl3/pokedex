use std::{env, fs, io::Cursor};

use anyhow::{Context, Result};
use binrw::BinReaderExt;

use pokedex::{
    Narc,
    fs::{Bytes, FsEntry, NitroFs},
    msg::decode_msg,
    sprite::{Rgcn, Rlcn},
};

fn main() -> Result<()> {
    let rom_path = env::args().nth(1).context("pass a ROM path")?;
    let rom = fs::read(rom_path).context("couldn't find ROM")?;
    // let max_strings = env::args().nth(2).map(|s| s.parse().unwrap()).unwrap_or(usize::MAX);

    let fs = NitroFs::new(&rom)?;
    let Some(FsEntry::File(contents)) = fs.get("poketool/icongra/poke_icon.narc".into())
    else {
        panic!("poke_icon.narc not found");
    };

    let mut c = Cursor::new(contents);
    let narc: Narc = c.read_le().unwrap();
    let img_start = c.position() as u32 + narc.fnt_size;
    for (i, f) in narc.fat.iter().enumerate() {
        let entry = &contents[(img_start + f.start) as usize..(img_start + f.end) as usize];
        let mut c = Cursor::new(entry);
        println!("{i}: {}", pretty_hex::pretty_hex(&entry));
        match &entry[..4] {
            b"RLCN" => {
                let palette: Rlcn = c.read_le().unwrap();
                println!("\x1b[1m===== {i} {:#?}\x1b[0m", palette);
            }
            b"RGCN" => {
                let sprite: Rgcn = c.read_le().unwrap();
                println!("\x1b[1m===== {i} {:#?}\x1b[0m", sprite);
            }
            _ => {
                println!("{i}: {}", str::from_utf8(&entry[..4]).unwrap());
            }
        }
    }
    Ok(())
}
