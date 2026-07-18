use std::{env, fs, io::Cursor};

use anyhow::{Context, Result};
use binrw::BinReaderExt;

use memchr::memmem;
use owo_colors::{OwoColorize, Rgb};
use pokedex::{
    Color, Game, Narc, UPPER,
    fs::{FsEntry, NitroFs},
    sprite::{Rgcn, Rlcn},
};

fn lz77(bytes: &[u8]) -> Vec<u8> {
    let mut decompressed = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let control = bytes[i];
        i += 1;
        for j in (0..8).rev() {
            if control & 1 << j == 0 {
                decompressed.push(bytes[i]);
                i += 1
            } else {
                let copy_offset = ((bytes[i] as u16 & 0xF0) << 4 | bytes[i + 1] as u16) + 1;
                let mut count = bytes[i] % 0x0F;
                let data = (&decompressed[decompressed.len() - copy_offset as usize..]);
                let n = 0;
                // while count != 0 {
                //     decompressed.push(data[n % data.len()]);
                //     n += 1;
                //     count -= 1;
                // }
            }
        }
    }
    decompressed
}

fn main() -> Result<()> {
    let rom_path = env::args().nth(1).context("pass a ROM path")?;
    let rom = fs::read(rom_path).context("couldn't find ROM")?;

    let fs = NitroFs::new(&rom)?;
    let game = fs.header.game()?;
    let Some(FsEntry::File(contents)) = fs.get(game.box_sprites().into()) else {
        panic!("poke_icon.narc not found");
    };

    let arm9 = &rom[fs.header.arm9.rom_offset as usize
        ..(fs.header.arm9.rom_offset + fs.header.arm9.size) as usize];
    println!("{}", pretty_hex::pretty_hex(&&arm9[arm9.len() - 400..]));
    let mut _decompressed = Default::default();
    let arm9 = if matches!(game, Game::HeartGold | Game::SoulSilver) {
        _decompressed = lz77(arm9);
        &_decompressed
    } else {
        arm9
    };
    let pattern = [0, 1, 1, 1, 0, 0, 0, 0, 2, 2, 1, 1, 0, 1, 2, 2].as_slice();
    let offset = memmem::find(arm9, pattern).unwrap();
    let palettes = &arm9[offset..];
    debug_assert!(memmem::find(&palettes[500..], pattern).is_none());
    // println!("{}", pretty_hex::pretty_hex(&&palettes[..400]));

    let mut c = Cursor::new(contents);
    let narc: Narc = c.read_le()?;
    let img_start = c.position() as u32 + narc.fnt_size;
    let mut colors = Vec::new();
    for (i, f) in narc.fat.iter().enumerate() {
        let entry = &contents[(img_start + f.start) as usize..(img_start + f.end) as usize];
        let mut c = Cursor::new(entry);
        match &entry[..4] {
            b"RLCN" => {
                // println!("{i}: {}", pretty_hex::pretty_hex(&entry));
                let palette: Rlcn = c.read_le().unwrap();
                colors = palette.ttlp.colors.clone();
                for pal in &mut colors {
                    pal[0] = Color(0);
                }
                println!("\x1b[1m===== {i}\x1b[0m  {:#?}", palette);
            }
            b"RGCN" => {
                let _sprite: Rgcn = c.read_le()?;
                let mut img = [[Color(0); 32]; 64];
                println!("pokemon #{}", i - 7);
                let pal = palettes[i - 7] as usize; // offset for palette/anim/other files
                for (i, &b) in entry[c.position() as usize..].iter().enumerate() {
                    let (x, y) = tile_coord(i * 2, 32);
                    img[y][x] = colors[pal][b as usize & 0xF];
                    img[y][x + 1] = colors[pal][b as usize >> 4];
                }
                let a = &img[..img.len() / 2];
                let b = &img[img.len() / 2..];
                for ([a1, a2], [b1, b2]) in a.as_chunks::<2>().0.iter().zip(b.as_chunks().0)
                {
                    for (&top, &bottom) in a1.iter().zip(a2) {
                        print!(
                            "{}",
                            UPPER.color::<Rgb>(top.into()).on_color::<Rgb>(bottom.into())
                        );
                    }
                    print!("    ");
                    for (&top, &bottom) in b1.iter().zip(b2) {
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
