use std::{env, fs, io::Cursor};

use anyhow::{Context, Result};
use binrw::BinReaderExt;

use pokedex::{
    Narc,
    fs::{Bytes, FsEntry, NitroFs},
    msg::decode_msg,
};

fn main() -> Result<()> {
    let rom_path = env::args().nth(1).context("pass a ROM path")?;
    let rom = fs::read(rom_path).context("couldn't find ROM")?;
    let max_strings = env::args().nth(2).map(|s| s.parse().unwrap()).unwrap_or(usize::MAX);

    let fs = NitroFs::new(&rom)?;
    let Some(FsEntry::File(contents)) = fs.get("msgdata/msg.narc".into()) else {
        panic!("msg.narc not found");
    };

    let mut c = Cursor::new(contents);
    let narc: Narc = c.read_ne().unwrap();
    let img_start = c.position() as u32 + narc.fnt_size;
    for (i, f) in narc.fat.iter().enumerate() {
        let entry = &contents[(img_start + f.start) as usize..(img_start + f.end) as usize];
        println!("\x1b[1m===== {i} {}\x1b[0m", Bytes(entry.len() as u32));
        for (i, s) in decode_msg(entry).unwrap().iter().enumerate().take(max_strings) {
            println!("{i:>4}: {s:?}");
        }
        println!()
    }
    Ok(())
}
