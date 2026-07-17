use binrw::BinRead;
use owo_colors::{OwoColorize, Rgb};

use crate::Color;

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
        f.write_str("\n")?;
        for c in self.ttlp.colors {
            f.write_fmt(format_args!("{}", "  ".on_color::<Rgb>(c.into())))?;
        }
        f.write_str("\n")?;
        f.write_str("0 1 2 3 4 5 6 7 8 9 A B C D E F")
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
    pub colors: [Color; 16],
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
