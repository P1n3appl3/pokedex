// http://problemkaputt.de/gbatek-ds-cartridge-nitrorom-and-nitroarc-file-systems.htm
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
    io::{Cursor, Seek, SeekFrom::Start},
    str::Utf8Error,
};

use anyhow::Result;
use binrw::{BinRead, BinReaderExt};
use camino::Utf8Path;
use modular_bitfield::prelude::*;
use smol_str::SmolStr;

use crate::{NdsHeader, Slice};

#[derive(Default, Clone, Debug)]
pub struct NitroFs<'a>(pub HashMap<SmolStr, FsEntry<'a>>);

#[derive(Debug, Clone)]
pub enum FsEntry<'a> {
    File(&'a [u8]),
    Dir(HashMap<SmolStr, FsEntry<'a>>),
}

impl<'a> NitroFs<'a> {
    pub fn new(rom: &'a [u8]) -> Result<Self> {
        let mut cursor = Cursor::new(rom);
        let header: NdsHeader = cursor.read_le()?;
        let mut collector = Collector::default();
        parse_nitro_fs(header.fnt, header.fat, &mut cursor, &mut collector)?;
        Ok(collector.fs)
    }

    pub fn get(&self, path: &Utf8Path) -> Option<&FsEntry<'a>> {
        let mut components = path.components();
        let mut cur: &FsEntry = self.0.get(components.next()?.as_str())?;
        for c in components {
            let FsEntry::Dir(map) = cur else {
                return None;
            };
            cur = map.get(c.as_str())?;
        }
        Some(cur)
    }
}

pub trait FsVisitor<'a> {
    fn push_dir(&mut self, _name: SmolStr, _id: u16) -> bool {
        true
    }

    fn pop_dir(&mut self) -> bool {
        true
    }

    fn file(&mut self, _name: SmolStr, _contents: &'a [u8], _id: u16) -> bool {
        true
    }
}

#[derive(Default)]
struct Collector<'a> {
    fs: NitroFs<'a>,
    stack: Vec<SmolStr>,
}

impl<'a> Collector<'a> {
    fn push(&mut self, name: SmolStr, new: FsEntry<'a>) {
        let mut cur = &mut self.fs.0;
        for component in &self.stack {
            let FsEntry::Dir(next) = cur.get_mut(component).unwrap() else {
                unreachable!("")
            };
            cur = next
        }
        cur.insert(name, new);
    }
}

impl<'a> FsVisitor<'a> for Collector<'a> {
    fn push_dir(&mut self, name: SmolStr, _id: u16) -> bool {
        self.push(name.clone(), FsEntry::Dir(Default::default()));
        self.stack.push(name);
        true
    }

    fn pop_dir(&mut self) -> bool {
        self.stack.pop();
        true
    }

    fn file(&mut self, name: SmolStr, contents: &'a [u8], _id: u16) -> bool {
        self.push(name, FsEntry::File(contents));
        true
    }
}

pub fn parse_nitro_fs<'a, T: FsVisitor<'a>>(
    fnt: Slice,
    fat: Slice,
    f: &mut Cursor<&'a [u8]>,
    v: &mut T,
) -> Result<()> {
    let mut files: FileAllocationTable = Vec::new();
    f.seek(Start(fat.offset as u64))?;
    for _ in 0..fat.size as usize / std::mem::size_of::<FatEntry>() {
        files.push(f.read_le()?);
    }

    f.seek(Start(fnt.offset as u64))?;
    let mut dirs: Vec<Directory> = vec![f.read_le()?];
    for _ in 0..dirs[0].parent_id - 1 {
        dirs.push(f.read_le()?)
    }

    let root = dirs[0];
    let mut stack = vec![((fnt.offset + root.subtable_offset) as u64, root.first_id)];
    while let Some((pos, mut cur_id)) = stack.pop() {
        f.seek(Start(pos))?;
        loop {
            let mut e: Entry = f.read_le()?;
            if e.header.len() == 0 {
                if !v.pop_dir() {
                    return Ok(());
                };
                debug_assert!(!e.header.is_dir());
                break;
            }
            if !e.header.is_dir() {
                e.id = cur_id;
                cur_id += 1;
                let file = &files[e.id as usize];
                let raw = &f.get_ref()[file.start as usize..file.end as usize];
                if !v.file(e.name.clone(), raw, e.id) {
                    return Ok(());
                }
            } else {
                if !v.push_dir(e.name.clone(), e.id) {
                    return Ok(());
                };
                stack.push((f.position(), cur_id));
                let dir = dirs[(e.id & 0xFFF) as usize];
                stack.push(((fnt.offset + dir.subtable_offset) as u64, dir.first_id));
                break;
            }
        }
    }

    Ok(())
}

// pub struct FsCollector(HashMap<Entry, Entry>);

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
struct Directory {
    subtable_offset: u32,
    first_id: u16,
    parent_id: u16, // for root dir this is total dir number
}

#[bitfield]
#[derive(BinRead, Debug, Copy, Clone, Default)]
#[br(map = Self::from_bytes)]
pub struct EntryHeader {
    len: B7,
    is_dir: bool,
}

// TODO: enum with is_dir bool as magic for variant selection
#[derive(BinRead, Clone)]
struct Entry {
    header: EntryHeader,
    #[br(count = header.len() as usize, try_map = smol_str)]
    name: SmolStr,
    #[br(if(header.is_dir()))] // else increment starting from first_id
    id: u16,
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
