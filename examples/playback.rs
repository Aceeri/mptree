
extern crate mptree;

use mptree::{decoder, header};
use std::io::{Read, Cursor};
use std::fs::{File};

fn main() {
    let file = File::open("mpeg_compliance_testing/layer3/sin1k0db.bit").unwrap();
    let mut frame_reader = decoder::FrameReader::new(file);

    for i in 0..5 {
        match frame_reader.advance() {
            Ok(frame) => {
                dbg!(frame);
            },
            Err(mptree::error::MpError::NoHeaderCapture) => {
                break;
            }
            Err(err) => {
                dbg!(err);
            },
        }
    }
}
