use std::{env, fs, io::Cursor};

use anyhow::{Context, Result};
use binrw::BinReaderExt;
use smol_str::SmolStr;

use pokedex::{
    Narc, NdsHeader,
    fs::{Bytes, FsVisitor, parse_nitro_fs},
    msg::decode_msg,
};

fn main() -> Result<()> {
    let rom_path = env::args().nth(1).context("pass a ROM path")?;
    let rom = fs::read(rom_path).context("couldn't find ROM")?;
    let max_strings = env::args().nth(2).map(|s| s.parse().unwrap()).unwrap_or(usize::MAX);

    let mut cursor = Cursor::new(rom);
    let header: NdsHeader = cursor.read_ne()?;
    parse_nitro_fs(header.fnt, header.fat, &mut cursor, &mut MsgVisitor(max_strings)).ok();

    Ok(())
}

struct MsgVisitor(usize);

impl FsVisitor for MsgVisitor {
    fn file(&mut self, name: SmolStr, contents: &[u8], _id: u16) -> bool {
        if name != "msg.narc" {
            return true;
        }
        let mut c = Cursor::new(contents);
        let narc: Narc = c.read_ne().unwrap();
        let img_start = c.position() as u32 + narc.fnt_size;
        for (i, f) in narc.fat.iter().enumerate() {
            let entry =
                &contents[(img_start + f.start) as usize..(img_start + f.end) as usize];
            println!("\x1b[1m===== {i} {}\x1b[0m", Bytes(entry.len() as u32));
            for (i, s) in decode_msg(entry).unwrap().iter().enumerate().take(self.0) {
                println!("{i:>4}: {s:?}");
            }
            println!()
        }
        false
    }
}
