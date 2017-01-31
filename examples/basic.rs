
extern crate mptree;

use mptree::{decoder, header};
use std::io::{Read, Cursor};
use std::fs::{File};

fn main() {
    let file = File::open("examples/whatislove.mp3").unwrap();
    let mut frame_reader = decoder::FrameReader::new(file);

    for _ in 0..50 {
        let header_data = frame_reader.read_until_header().unwrap();
        println!("{:?}", header_data);
        let header = header::Header::construct(&header_data);
        println!("{:?}", header);
    }
}
