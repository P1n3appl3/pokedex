// http://problemkaputt.de/gbatek-ds-cartridge-nitrorom-and-nitroarc-file-systems.htm
use std::{
    fmt::{self, Debug, Display, Formatter},
    io::{Cursor, Read, Seek, SeekFrom::Start},
    os::unix::ffi::OsStrExt,
    str::Utf8Error,
};

use anyhow::Result;
use binrw::{BinRead, BinReaderExt};
use modular_bitfield_msb::prelude::*;
use smol_str::SmolStr;

use crate::{Narc, Slice, charset::CHARSET};

pub fn parse_nitro_fs(fnt: Slice, fat: Slice, f: &mut Cursor<Vec<u8>>) -> Result<()> {
    let mut files: FileAllocationTable = Vec::new();
    f.seek(Start(fat.offset as u64))?;
    for _ in 0..fat.size as usize / std::mem::size_of::<FatEntry>() {
        files.push(f.read_ne()?);
    }

    f.seek(Start(fnt.offset as u64))?;
    let mut dirs: Vec<Directory> = vec![f.read_ne()?];
    for _ in 0..dirs[0].parent_id - 1 {
        dirs.push(f.read_ne()?)
    }

    let root = dirs[0];
    let mut stack = vec![((fnt.offset + root.subtable_offset) as u64, root.first_id)];
    while let Some((pos, mut cur_id)) = stack.pop() {
        f.seek(Start(pos))?;
        loop {
            let mut e: Entry = f.read_ne()?;
            if e.header.len() == 0 {
                debug_assert!(!e.header.is_dir());
                break;
            }
            // print!("{}", "  ".repeat(stack.len()));
            if !e.header.is_dir() {
                e.id = cur_id;
                cur_id += 1;
                let file = &files[e.id as usize];
                let size = Bytes(file.end - file.start).to_string();
                if e.name == "msg.narc" {
                    let raw = &f.get_ref()[file.start as usize..file.end as usize];
                    let mut c = Cursor::new(raw);
                    let narc: Narc = c.read_ne()?;
                    let img_start = c.position() as u32 + narc.fnt_size;
                    for (i, f) in narc.fat.iter().enumerate() {
                        // if i != 615 {
                        //     continue;
                        // }
                        println!(
                            "{i} 0x{:X} {}",
                            f.start + file.start + img_start as u32,
                            Bytes(f.end - f.start)
                        );
                        decode_msg(
                            &raw[(img_start + f.start) as usize
                                ..(img_start + f.end) as usize],
                        )?;
                    }
                    return Ok(());
                }
                // println!(
                //     "\x1b[3m{}\x1b[0m{}\x1b[2m{size:>8} @ 0x{:04x}\x1b[0m",
                //     e.name,
                //     " ".repeat(60 - e.name.len() - stack.len() * 2),
                //     e.id,
                // );
            } else {
                // println!("\x1b[1m{}\x1b[0m \x1b[2m(0x{:x})\x1b[0m", e.name, e.id);
                stack.push((f.position(), cur_id));
                let dir = dirs[(e.id & 0xFFF) as usize];
                stack.push(((fnt.offset + dir.subtable_offset) as u64, dir.first_id));
                break;
            }
        }
    }

    Ok(())
}

// transcribed to rust straight from
// https://projectpokemon.org/rawdb/platinum/formats/msg.php
fn decode_msg(bytes: &[u8]) -> Result<()> {
    let mut c = Cursor::new(bytes);
    let count: u16 = c.read_ne()?;
    let seed: u16 = c.read_ne()?;
    let mut offsets = Vec::new();
    let mut sizes = Vec::new();
    // let mut strings = Vec::new();

    for i in 1..=count {
        let temp = seed as u64 * i as u64;
        let key = ((temp * 0x2fd) & 0xFFFF) | ((temp * 0x2FD0000) & 0xFFFF0000);
        offsets.push(c.read_le::<u32>()? ^ key as u32);
        sizes.push(c.read_le::<u32>()? ^ key as u32);
    }

    for i in 1..=count {
        let offset = offsets[i as usize - 1];
        let size = sizes[i as usize - 1];
        let mut key = (0x91BD3 * i as u32) & 0xFFFF;
        c.seek(Start(offset as u64))?;
        let mut string = Vec::new();
        for _ in 1..=size {
            string.push(c.read_ne::<u16>()? ^ key as u16);
            key = (key + 0x493d) & 0xFFFF;
        }
        if string.first() == Some(&0xF100) {
            todo!("handle compression")
        }
        let mut s = String::new();
        let mut chars = string.into_iter();
        while let Some(c) = chars.next() {
            if c == 0xFFFF {
                break;
            } else if c == 0xFFFE {
                let n = chars.next().unwrap();
                let count = chars.next().unwrap();
                dbg!(count);
                s.push_str("VAR{");
                s.push_str(&format!("{n:x}"));
                for _ in 0..count {
                    s.push(':');
                    s.push_str(&chars.next().unwrap().to_string());
                }
                s.push('}');
            } else if c == 0xE000 {
                s.push('\n');
            } else if c == 0x25BC || c == 0x25BD {
                s.push_str("{PROMPT}");
            } else {
                s.push(CHARSET[c as usize]);
            }
        }
        println!("String {i}: {s}\n")
    }

    Ok(())
}

#[derive(Copy, Clone, Default, Debug)]
struct Bytes(u32);
impl Display for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let n = self.0;
        const K: u32 = 1024;
        const M: u32 = 0x100000;
        const G: u32 = 0x40000000;
        match n {
            0..K => write!(f, "{n}B"),
            K..M => write!(f, "{:.1}KiB", n as f32 / K as f32),
            M..G => write!(f, "{:.1}MiB", n as f32 / M as f32),
            G.. => write!(f, "{:.1}GiB", n as f32 / G as f32),
        }
    }
}

#[derive(BinRead, Debug, Copy, Clone)]
pub struct Directory {
    pub subtable_offset: u32,
    pub first_id: u16,
    pub parent_id: u16, // for root dir this is total dir number
}

#[bitfield]
#[derive(BinRead, Debug, Copy, Clone, Default)]
#[br(map = Self::from_bytes)]
pub struct EntryHeader {
    is_dir: bool,
    len: B7,
}

// TODO: enum with is_dir bool as magic for variant selection
#[derive(BinRead, Clone)]
pub struct Entry {
    pub header: EntryHeader,
    #[br(count = header.len() as usize, try_map = smol_str)]
    pub name: SmolStr,
    #[br(if(header.is_dir()))] // else increment starting from first_id
    pub id: u16,
}

impl Debug for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (0x{:x}): '{}'",
            if self.header.is_dir() { "Dir" } else { "File" },
            self.id,
            self.name
        )?;
        Ok(())
    }
}

fn smol_str(s: Vec<u8>) -> Result<SmolStr, Utf8Error> {
    Ok(SmolStr::new(std::str::from_utf8(&s)?))
}

#[derive(BinRead, Debug, Copy, Clone)]
pub struct FatEntry {
    start: u32,
    end: u32,
}
pub type FileAllocationTable = Vec<FatEntry>;
