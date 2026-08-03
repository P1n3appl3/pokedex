use std::io::{Cursor, Seek, SeekFrom::Start};

use anyhow::Result;
use binrw::BinReaderExt;

use crate::{Game, charset::CHARSET};

impl Game {
    pub fn msgs(self) -> Msgs {
        use Game::*;
        match self {
            Diamond | Pearl => Msgs {
                name: 362,
                nickname: 621,
                pokedex_prose: 615,
                weight: 619,
                height: 620,
                ability: 610,
                ability_desc: 612,
                item: 392,
                item_desc: 391,
                r#move: 647,
                move_desc: 646,
                nature: 190,
                status: 219,
                stat: 495,
                r#type: 565,
            },
            Platinum => todo!(),
            HeartGold | SoulSilver => todo!(),
        }
    }
}

pub struct Msgs {
    pub name: usize,
    pub nickname: usize,
    pub pokedex_prose: usize,
    pub weight: usize,
    pub height: usize,
    pub ability: usize,
    pub ability_desc: usize,
    pub item: usize,
    pub item_desc: usize,
    pub r#move: usize,
    pub move_desc: usize,
    pub nature: usize,
    pub status: usize,
    pub r#type: usize,
    pub stat: usize,
}

// https://projectpokemon.org/rawdb/platinum/formats/msg.php
// https://github.com/projectpokemon/ppre_legacy/blob/master/poketext.py
pub fn decode_msg(bytes: &[u8]) -> Result<Vec<String>> {
    let mut c = Cursor::new(bytes);
    let count: u16 = c.read_le()?;
    let seed: u16 = c.read_le()?;

    let mut offsets = Vec::new();
    let mut sizes = Vec::new();
    for i in 1..=count {
        let temp = seed as u64 * i as u64;
        let key = ((temp * 0x2FD) & 0xFFFF) | ((temp * 0x2FD0000) & 0xFFFF0000);
        offsets.push(c.read_le::<u32>()? ^ key as u32);
        sizes.push(c.read_le::<u32>()? ^ key as u32);
    }

    let mut out = Vec::new();
    for i in 1..=count {
        let offset = offsets[i as usize - 1];
        let size = sizes[i as usize - 1];
        let mut key = (0x91BD3 * i as u32) & 0xFFFF;
        c.seek(Start(offset as u64))?;
        let mut string = Vec::new();
        for _ in 1..=size {
            string.push(c.read_le::<u16>()? ^ key as u16);
            key = (key + 0x493D) & 0xFFFF;
        }
        if string.first() == Some(&0xF100) {
            let mut new = Vec::new();
            let mut container = 0u32;
            let mut bit = 0;
            let mut chars = string.iter().skip(1).take_while(|&&c| c != 0xFFFF);
            loop {
                if bit < 9 {
                    let Some(&c) = chars.next() else { break };
                    container &= (1 << bit) - 1;
                    container |= (c as u32) << bit;
                    // println!("popped {:04x}", c);
                    bit += 15;
                }
                // println!("{bit}, {container:024b} {:09b}", container >> (bit - 9));
                let temp = container & 0x1FF;
                // println!("extracted {temp}");
                if 0b1_1111_1111 == temp {
                    break;
                }
                new.push(temp as u16);
                bit -= 9;
                container >>= 9;
            }
            string = new;
        }
        let mut s = String::new();
        let mut chars = string.into_iter();
        while let Some(c) = chars.next() {
            if c == 0xFFFF {
                break;
            } else if c == 0xFFFE {
                let n = chars.next().unwrap();
                let count = chars.next().unwrap();
                s.push_str("VAR{");
                s.push_str(&format!("0x{n:x}"));
                for _ in 0..count {
                    s.push(':');
                    s.push_str(&chars.next().unwrap().to_string());
                }
                s.push('}');
            } else if c == 0xE000 {
                s.push('\n');
            } else if c == 0x25BC || c == 0x25BD {
                s.push('\r');
            } else if c == 0x1E0 {
                s.push_str("ᴾₖ");
            } else if c == 0x1E1 {
                s.push_str("ᴹₙ");
            } else {
                s.push(CHARSET[c as usize]);
            }
        }
        out.push(s)
    }

    Ok(out)
}
