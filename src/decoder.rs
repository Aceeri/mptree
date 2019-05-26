
use std::io::{self, Read};

use ::frame::Frame;
use ::error::MpError;
use ::header::{Header, HEADER_SIZE};

use byteorder::ReadBytesExt;

pub struct FrameReader<R: io::Read + io::Seek> {
    reader: R,
}

// How many bytes to take at once from the reader.
//const BUFFER_AMOUNT: usize = 1024;

// How many bytes ahead should we check before erroring on header seeking.
const HEADER_LIMIT: usize = 1024 * 10; // 10kB

impl<R: io::Read + io::Seek> FrameReader<R> {
    pub fn new(reader: R) -> FrameReader<R> {
        FrameReader {
            reader: reader,
        }
    }

    pub fn advance(&mut self) -> Result<Frame, MpError> {
        let header = self.find_header(HEADER_LIMIT).ok_or(MpError::NoHeaderCapture)?;
        dbg!(header);
        Err(MpError::InvalidData("Unimplemented".to_string()))
    }

    // Find the next MP3 header within a limit of bytes.
    fn find_header(&mut self, limit: usize) -> Option<Header> {
        let mut header_bytes: [u8; 4] = [
            self.reader.read_u8().ok()?,
            self.reader.read_u8().ok()?,
            self.reader.read_u8().ok()?,
            self.reader.read_u8().ok()?,
        ];
        let mut amount_read = 4;

        while amount_read < limit {
            if header_bytes[0] == 0xFF && header_bytes[1] & 0xE0 == 0xE0 {
                match Header::new(&header_bytes) {
                    Ok(header) => return Some(header),
                    Err(err) => {
                        dbg!(err);
                    },
                }
            }

            header_bytes[0] = header_bytes[1];
            header_bytes[1] = header_bytes[2];
            header_bytes[2] = header_bytes[3];
            header_bytes[3] = self.reader.read_u8().ok()?;
            amount_read += 1;
        }

        None
    }
}
