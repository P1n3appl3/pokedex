use std::{
    env, fs,
    io::{Cursor, Seek, SeekFrom::Start},
};

use anyhow::{Context, Result};
use binrw::BinReaderExt;

use pokedex::{
    NdsHeader,
    fs::{FsVisitor, parse_nitro_fs},
};
use smol_str::SmolStr;

fn main() -> Result<()> {
    let rom_path = env::args().nth(1).context("pass a ROM path")?;
    let rom = fs::read(rom_path).context("couldn't find ROM")?;

    let mut cursor = Cursor::new(&rom[..]);
    let header: NdsHeader = cursor.read_le()?;
    cursor.seek(Start(header.banner_offset as u64))?;

    let mut v = TreeVisitor(0);
    parse_nitro_fs(header.fnt, header.fat, &mut cursor, &mut v)?;

    Ok(())
}

struct TreeVisitor(usize);

impl<'a> FsVisitor<'a> for TreeVisitor {
    fn push_dir(&mut self, name: SmolStr, id: u16) -> bool {
        print!("{}", "  ".repeat(self.0));
        println!("\x1b[1m{}\x1b[0m \x1b[2m(0x{:x})\x1b[0m", name, id);
        self.0 += 1;
        true
    }

    fn pop_dir(&mut self) -> bool {
        self.0 -= 1;
        true
    }

    fn file(&mut self, name: SmolStr, contents: &[u8], id: u16) -> bool {
        let size = contents.len();
        print!("{}", "  ".repeat(self.0));
        println!(
            "\x1b[3m{}\x1b[0m{}\x1b[2m{size:>8} @ 0x{:04x}\x1b[0m",
            name,
            " ".repeat(60 - name.len() - self.0 * 2),
            id
        );
        true
    }
}
