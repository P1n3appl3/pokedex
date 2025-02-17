// http://problemkaputt.de/gbatek-ds-cartridge-nitrorom-and-nitroarc-file-systems.htm
use std::{
    fmt::{self, Debug, Display, Formatter},
    io::{Read, Seek, SeekFrom::Start},
    ops::Range,
    str::Utf8Error,
};

use anyhow::Result;
use binrw::{BinRead, BinReaderExt, VecArgs};
use modular_bitfield_msb::prelude::*;
use smol_str::SmolStr;

use crate::Slice;

macro_rules! doo {
    (($($body:tt)+) whilst $cond:tt) => {
        loop {
            $($body)+;
            if ! $cond { break; }
        }
    };
}

pub fn parse_nitro_fs(fnt: Slice, fat: Slice, f: &mut (impl Read + Seek)) -> Result<()> {
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
    let mut segments = vec![0];
    let mut d_entries = Vec::new();
    let mut f_entries = Vec::new();
    for d in &dirs {
        let mut cur_id = d.first_id;
        f.seek(Start((fnt.offset + d.subtable_offset) as u64))?;
        loop {
            let mut e: Entry = f.read_ne()?;
            // print!("{}", "  ".repeat(segments.len() - 1));
            if !e.header.is_dir() {
                if e.header.len() == 0 {
                    segments.pop();
                    break;
                }
                e.id = cur_id;
                cur_id += 1;
                let f = &files[e.id as usize];
                // println!("{:20}: {:>}", e.name, Bytes(f.end - f.start));
                f_entries.push(e);
            } else {
                // doo!((
                //     let cur = *segments.last().unwrap();
                //     let parent = dirs[(cur & !0xF000) as usize].parent_id;
                // ) whilst {
                //     if cur == parent {
                //         segments.push();
                //     }
                // });
                segments.push(e.id);
                // println!("{} (0x{:x})", e.name, e.id);

                d_entries.push(e);
            }
        }
    }

    let mut segments = vec![0xF000];
    let (mut fi, mut di) = (0, 0);
    while fi < f_entries.len() {
        let d = &d_entries[di];
        let parent = dirs[(d.id & !0xF000) as usize].parent_id;
        while parent != *segments.last().unwrap() {
            segments.pop();
        }
        let indent = "  ".repeat(segments.len() - 1);
        segments.push(d.id);
        println!("{indent}{} (0x{:04x})", d.name, d.id);

        let mut f = &f_entries[fi];
        while dirs[(d.id & !0xF000) as usize].first_id > f.id {
            println!("{indent}  {}", f.name);
            fi += 1;
            f = &f_entries[fi];
        }
        di += 1;
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

// #[derive(BinRead, Clone, Debug)]
// struct Dirs {
//     root: Directory,
//     #[br(count=root.parent_id)]
//     dirs: Vec<Directory>,
// }

#[bitfield]
#[derive(BinRead, Debug, Copy, Clone, Default)]
#[br(map = Self::from_bytes)]
pub struct EntryHeader {
    is_dir: bool,
    len: B7,
}

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
struct FatEntry {
    start: u32,
    end: u32,
}
type FileAllocationTable = Vec<FatEntry>;
