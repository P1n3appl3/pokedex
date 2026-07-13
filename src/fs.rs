// http://problemkaputt.de/gbatek-ds-cartridge-nitrorom-and-nitroarc-file-systems.htm
use std::{
    fmt::{self, Debug, Display, Formatter},
    io::{Cursor, Seek, SeekFrom::Start},
    str::Utf8Error,
};

use anyhow::{Result, anyhow};
use binrw::{BinRead, BinReaderExt};
use modular_bitfield_msb::prelude::*;
use smol_str::SmolStr;

use crate::Slice;

pub trait FsVisitor {
    fn push_dir(&mut self, _name: SmolStr, _id: u16) -> bool {
        true
    }

    fn pop_dir(&mut self) -> bool {
        true
    }

    fn file(&mut self, _name: SmolStr, _contents: &[u8], _id: u16) -> bool {
        true
    }
}

pub fn parse_nitro_fs<T: FsVisitor>(
    fnt: Slice,
    fat: Slice,
    f: &mut Cursor<Vec<u8>>,
    v: &mut T,
) -> Result<(FileAllocationTable, Vec<Directory>)> {
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
                v.pop_dir();
                debug_assert!(!e.header.is_dir());
                break;
            }
            if !e.header.is_dir() {
                e.id = cur_id;
                cur_id += 1;
                let file = &files[e.id as usize];
                let raw = &f.get_ref()[file.start as usize..file.end as usize];
                if !v.file(e.name.clone(), raw, e.id) {
                    return Err(anyhow!("terminated early"));
                }
            } else {
                v.push_dir(e.name.clone(), e.id);
                stack.push((f.position(), cur_id));
                let dir = dirs[(e.id & 0xFFF) as usize];
                stack.push(((fnt.offset + dir.subtable_offset) as u64, dir.first_id));
                break;
            }
        }
    }

    Ok((files, dirs))
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Bytes(pub u32);
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
    pub start: u32,
    pub end: u32,
}
pub type FileAllocationTable = Vec<FatEntry>;
