
use std::io;

use ::header::{Layer, ChannelMode};

#[derive(Debug)]
pub enum MpError {
    IOError(io::Error),
    EOF,
    InvalidData(String),
    NoHeaderCapture, // could not capture header sync
    BadBit(u16), // bit index was non-existent or forbidden
    InvalidMode(ChannelMode, Vec<ChannelMode>), // got, expected (on of)
    Reserved, // input a reserved mode, version, or layer
}

impl From<io::Error> for MpError {
    fn from(io: io::Error) -> MpError {
        MpError::IOError(io)
    }
}

pub fn wrong_layer(layer: &Layer) -> String {
    format!("Layer {} is currently unsupported", layer)
}
