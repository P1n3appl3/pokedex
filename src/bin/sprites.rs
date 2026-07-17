use std::{env, fs, io::Cursor};

use anyhow::{Context, Result};
use binrw::BinReaderExt;

use owo_colors::{OwoColorize, Rgb};
use pokedex::{
    Color, Narc, UPPER,
    fs::{FsEntry, NitroFs},
    sprite::{Rgcn, Rlcn},
};

fn main() -> Result<()> {
    let rom_path = env::args().nth(1).context("pass a ROM path")?;
    let rom = fs::read(rom_path).context("couldn't find ROM")?;

    let fs = NitroFs::new(&rom)?;
    let Some(FsEntry::File(contents)) = fs.get("poketool/icongra/poke_icon.narc".into())
    else {
        panic!("poke_icon.narc not found");
    };

    let mut c = Cursor::new(contents);
    let narc: Narc = c.read_le()?;
    let img_start = c.position() as u32 + narc.fnt_size;
    let mut colors = [Color(0xFF); 16];
    for (i, f) in narc.fat.iter().enumerate() {
        let entry = &contents[(img_start + f.start) as usize..(img_start + f.end) as usize];
        let mut c = Cursor::new(entry);
        match &entry[..4] {
            b"RLCN" => {
                let palette: Rlcn = c.read_le().unwrap();
                colors = palette.ttlp.colors;
                colors[0] = Color(0);
                println!("\x1b[1m===== {i}\x1b[0m  {:#?}", palette);
            }
            b"RGCN" => {
                let _sprite: Rgcn = c.read_le()?;
                let mut img = [[Color(0); 32]; 64];
                println!("{i}");
                for (i, &b) in entry[c.position() as usize..].iter().enumerate() {
                    let (x, y) = tile_coord(i * 2, 32);
                    img[y][x] = colors[b as usize & 0xF];
                    img[y][x + 1] = colors[b as usize >> 4];
                }
                for [r1, r2] in img.as_chunks::<2>().0 {
                    for (&top, &bottom) in r1.iter().zip(r2) {
                        print!(
                            "{}",
                            UPPER.color::<Rgb>(top.into()).on_color::<Rgb>(bottom.into())
                        );
                    }
                    println!()
                }
            }
            _ => {
                println!("{i}: {}", str::from_utf8(&entry[..4]).unwrap());
            }
        }
    }
    Ok(())
}

fn tile_coord(i: usize, width: usize) -> (usize, usize) {
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
/*
0 0,0
1 1,0
...
7 7,0
8 0,1
9 1,1
...
13 6,1
14 7,1
15 0,2
...

*/
