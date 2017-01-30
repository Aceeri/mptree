
extern crate mptree;

use mptree::decoder;
use std::io::{Cursor};

fn main() {
    let data: &[u8] = &[
        0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
        0xFF, 0xEF, 0xF2, 0x11, 0x05,
        0x10, 0x50, 0x08, 0xA8, 0xC3,
    ];
    let cursor = Cursor::new(data);
    let mut frame_reader = decoder::FrameReader::new(cursor);
    println!("{:?}", frame_reader.read_until_header());
}
