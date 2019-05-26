
use std::io::{self, Read};

use ::frame::Frame;
use ::error::MpError;
use ::header::{Header, HEADER_SIZE};

pub struct FrameReader<R: io::Read + io::Seek> {
    reader: R,
    buffer: Vec<u8>, // Keep a buffer of the currently read data.
    buffer_index: usize,
}

// How many bytes to take at once from the reader.
const BUFFER_AMOUNT: usize = 1024;

// How many bytes ahead should we check before erroring on header seeking.
const HEADER_LIMIT: usize = 1024 * 10; // 10kB

impl<R: io::Read + io::Seek> FrameReader<R> {
    pub fn new(reader: R) -> FrameReader<R> {
        FrameReader {
            reader: reader,
            buffer: Vec::new(),
            buffer_index: 0,
        }
    }

    pub fn advance(&mut self) -> Result<Frame, MpError> {
        if self.buffer_index >= self.buffer.len() {
            self.extend_buffer()?;
        }

        let header = self.find_header(HEADER_LIMIT).ok_or(MpError::NoHeaderCapture)?;
        dbg!(header);
        dbg!(self.buffer_index);
        self.buffer_index += 1;
        Err(MpError::InvalidData("Unimplemented".to_string()))
    }

    // Find the next MP3 header within a limit of bytes.
    fn find_header(&mut self, limit: usize) -> Option<Header> {
        let mut amount_read = 0;
        while amount_read < limit {
            let header_size_buffer = self.buffer_index + HEADER_SIZE;
            let buffer_length = self.buffer.len();
            if header_size_buffer >= buffer_length {
                if let Err(_) = self.strict_extend_buffer(buffer_length - header_size_buffer) {
                    return None;
                }
            }

            if self.buffer[self.buffer_index] == 0xFF && self.buffer[self.buffer_index + 1] & 0xE0 == 0xE0 {
                match Header::new(&self.buffer[self.buffer_index..self.buffer_index + HEADER_SIZE]) {
                    Ok(header) => return Some(header),
                    Err(err) => {
                        dbg!(err);

                    },
                }
                //if let Ok(header) = Header::new(&self.buffer[self.buffer_index..self.buffer_index+HEADER_SIZE]) {
                    //return Some(header);
                //}
            }

            self.buffer_index += 1;
            amount_read += 1;
        }

        None
    }

    fn strict_extend_buffer(&mut self, required: usize) -> Result<usize, MpError> {
        let amount_extended = self.extend_buffer()?;
        if amount_extended >= required {
            Ok(amount_extended)
        } else {
            Err(MpError::EOF)
        }
    }

    fn extend_buffer(&mut self) -> Result<usize, io::Error> {
        let mut buffer = [0; BUFFER_AMOUNT];
        let amount = self.reader.read(&mut buffer)?;
        self.buffer.extend(buffer.iter());
        Ok(amount)
    }
}
