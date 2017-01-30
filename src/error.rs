
use std::io;

use ::header::ChannelMode;

#[derive(Debug)]
pub enum MpError {
    ReadError(io::Error),
    NoCapture, // could not capture header sync
    BadBit(u16), // bit index was non-existent or forbidden
    InvalidMode(ChannelMode, Vec<ChannelMode>), // got, expected (on of)
    Reserved, // input a reserved mode, version, or layer

}

impl From<io::Error> for MpError {
    fn from(io: io::Error) -> MpError {
        MpError::ReadError(io)
    }
}
