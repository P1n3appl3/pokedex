use std::io::Cursor;

use anyhow::{Result, anyhow};
use binrw::{BinRead, BinReaderExt};
use memchr::memmem;
use owo_colors::{OwoColorize, Rgb};

use crate::{
    Color, Game, Narc,
    fs::{FsEntry, NitroFs},
};

// color resource
#[derive(Clone, BinRead)]
#[br(magic = b"RLCN")]
pub struct Rlcn {
    #[br(assert(byte_order == 0xFEFF))]
    pub byte_order: u16,
    #[br(assert(version == 0x100))]
    pub version: u16,
    pub size: u32,
    #[br(assert(header_size == 0x10))]
    pub header_size: u16,
    #[br(assert(sections == 1))]
    pub sections: u16,
    pub ttlp: Ttlp,
}

impl std::fmt::Debug for Rlcn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for colors in &self.ttlp.colors {
            f.write_str("\n")?;
            for &c in colors {
                f.write_fmt(format_args!("{}", "  ".on_color::<Rgb>(c.into())))?;
            }
            f.write_str("\n")?;
            f.write_str("0 1 2 3 4 5 6 7 8 9 A B C D E F")?;
        }
        Ok(())
    }
}

// palette data
#[derive(Clone, Debug, BinRead)]
#[br(magic = b"TTLP")]
pub struct Ttlp {
    pub size: u32,
    pub bit_depth: u32,
    #[br(assert(padding == 0))]
    pub padding: u32,
    pub data_size: u32,
    #[br(assert(count == 16))]
    pub count: u32,
    #[br(count = data_size / count / 2)]
    pub colors: Vec<[Color; 16]>,
}

// nintendo character graphic resource
#[derive(Clone, Debug, BinRead)]
#[br(magic = b"RGCN")]
pub struct Rgcn {
    #[br(assert(byte_order == 0xFEFF))]
    pub byte_order: u16,
    #[br(assert(version == 0x101))]
    pub version: u16,
    pub size: u32,
    #[br(assert(header_size == 0x10))]
    pub header_size: u16,
    #[br(assert(sections == 1))]
    pub sections: u16,
    pub rahc: Rahc,
}

// palette data
#[derive(Clone, Debug, BinRead)]
#[br(magic = b"RAHC")]
pub struct Rahc {
    pub size: u32,
    pub tile_count: u16,
    pub tile_size: u16,
    pub bit_depth: u32,
    // #[br(assert(_padding == 0))]
    _padding: u64,
    // #[br(assert(tile_count * 1024 == tile_data_size))]
    pub tile_data_size: u32,
    #[br(assert(_unknown == 0x18))]
    _unknown: u32,
    // pub data: Vec<u8>,
}

pub fn get_palettes<'a>(rom: &'a [u8], fs: &NitroFs, buf: &'a mut Vec<u8>) -> &'a [u8] {
    let arm9 = &rom[fs.header.arm9.rom_offset as usize
        ..(fs.header.arm9.rom_offset + fs.header.arm9.size) as usize];
    let arm9 = if matches!(fs.header.game().unwrap(), Game::HeartGold | Game::SoulSilver) {
        blz(arm9, buf);
        buf
    } else {
        arm9
    };
    let pattern = [0, 1, 1, 1, 0, 0, 0, 0, 2, 2, 1, 1, 0, 1, 2, 2].as_slice();
    let offset = memmem::find(arm9, pattern).unwrap();
    let palettes = &arm9[offset..];
    debug_assert!(memmem::find(&palettes[500..], pattern).is_none());
    palettes
}

pub fn draw(
    pokemon: usize,
    alt_frame: bool,
    palettes: &[u8],
    fs: &NitroFs,
) -> Result<[[Color; 32]; 32]> {
    let game = fs.header.game()?;
    let Some(FsEntry::File(contents)) = fs.get(game.box_sprites().into()) else {
        panic!("poke_icon.narc not found");
    };

    let mut c = Cursor::new(contents);
    let narc: Narc = c.read_le()?;
    let img_start = c.position() as u32 + narc.fnt_size;

    let mut colors = {
        let f = narc.fat[0];
        let palette_narc =
            &contents[(img_start + f.start) as usize..(img_start + f.end) as usize];
        assert_eq!(palette_narc[..4], *b"RLCN");
        let mut c = Cursor::new(palette_narc);
        let palette: Rlcn = c.read_le().unwrap();
        palette.ttlp.colors
    };
    for pal in &mut colors {
        pal[0] = Color(0);
    }

    let raw_sprite = narc
        .fat
        .iter()
        .filter_map(|f| {
            let entry =
                &contents[(img_start + f.start) as usize..(img_start + f.end) as usize];
            (entry[..4] == *b"RGCN").then_some(entry)
        })
        .nth(pokemon)
        .ok_or_else(|| anyhow!("out of range"))?;

    let mut c = Cursor::new(raw_sprite);
    let _sprite: Rgcn = c.read_le()?;

    let palette = colors[palettes[pokemon] as usize];
    Ok(render(&raw_sprite[c.position() as usize..], palette.as_slice(), alt_frame))
}

// TODO: fix size?
fn render(sprite: &[u8], colors: &[Color], alt: bool) -> [[Color; 32]; 32] {
    let mut img = [[Color(0); 32]; 32];
    let skip = if alt { 512 } else { 0 };
    for (i, &b) in sprite.iter().skip(skip).enumerate().take(512) {
        let (x, y) = tile_coord(i * 2, 32);
        img[y][x] = colors[b as usize & 0xF];
        img[y][x + 1] = colors[b as usize >> 4];
    }
    img
}

pub fn tile_coord(i: usize, width: usize) -> (usize, usize) {
    const TILE_SIZE: usize = 8;
    debug_assert!(width.is_multiple_of(TILE_SIZE));
    let tile = TILE_SIZE.pow(2);
    let tile_local = i % tile;
    let (x, y) = (tile_local % TILE_SIZE, tile_local / TILE_SIZE);
    let global = i / tile;
    let tile_width = width / TILE_SIZE;
    let (tx, ty) = (global % tile_width, global / tile_width);
    (tx * TILE_SIZE + x, ty * TILE_SIZE + y)
}

// TODO
fn blz(bytes: &[u8], _buf: &mut [u8]) {
    let mut i = bytes.len() - 4;
    let extra_space = u32::from_ne_bytes(*bytes[i..].as_array().unwrap());
    let read_offset = bytes[i - 1];
    i -= 4;

    let size = u32::from_ne_bytes(*bytes[i..i + 4].as_array().unwrap()) & 0xFFF0;
    dbg!(size, read_offset, extra_space);
}
