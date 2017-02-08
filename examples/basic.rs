
extern crate mptree;

use mptree::{decoder, header};
use std::io::{Read, Cursor};
use std::fs::{File};

fn main() {
    let file = File::open("examples/whatislove.mp3").unwrap();
    let mut frame_reader = decoder::FrameReader::new(file);

    //let data = [0xFF, 0xFB, 0x78, 0x64];
    //let cursor = Cursor::new(data);
    //let mut frame_reader = decoder::FrameReader::new(cursor);
    /*for _ in 0..50 {
        let header_data = match frame_reader.read_until_header() {
            Ok(header) => header,
            Err(e) => break,
        };

        println!("{:?}", header_data);
        let header = header::Header::construct(&header_data);
        println!("{:?}", header);

        if let Ok(header) = header {
            println!("length: {:?}", header.frame_length());
        }
    }*/

    frame_reader.read();
    println!("----------------------------------------");
}
