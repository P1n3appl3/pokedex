use binrw::BinRead;

use crate::Color;

// color resource
#[derive(Clone, Debug, BinRead)]
#[br(magic = b"RLCN")]
pub struct Rlcn {
    #[br(assert(byte_order == 0xFFFE))]
    pub byte_order: u16,
    #[br(assert(version == 0x100))]
    pub version: u16,
    pub size: u32,
    #[br(assert(header_size == 0x10))]
    pub header_size: u16,
    #[br(assert(sections == 2))]
    pub sections: u16,
    pub ttlp: Ttlp,
}

// palette data
#[derive(Clone, Debug, BinRead)]
#[br(magic = b"Ttlp")]
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
    #[br(assert(byte_order == 0xFFFE))]
    pub byte_order: u16,
    #[br(assert(version == 1))]
    pub version: u16,
    pub size: u32,
    #[br(assert(header_size == 0x10))]
    pub header_size: u16,
    #[br(assert(sections == 2))]
    pub sections: u16,
    pub rahc: Rahc,
}

// palette data
#[derive(Clone, Debug, BinRead)]
#[br(magic = b"RAHC")]
pub struct Rahc {
    // #[br(assert(header_size == 0x20))]
    // pub header_size: u32,
    pub size: u32,
    pub tile_count: u16,
    #[br(assert(tile_size == 0x20))]
    pub tile_size: u16,
    pub bit_depth: u32,
    #[br(assert(padding == 0))]
    pub padding: u32,
    #[br(assert(tile_count * 1024 == tile_data_size))]
    pub tile_data_size: u16,
    #[br(assert(unknown == 0x24))]
    unknown: u16,
    // pub data: Vec<u8>,
}
