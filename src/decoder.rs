
use std::io::{self, Read, Seek, SeekFrom, Cursor};
use byteorder::{BigEndian, ReadBytesExt};

use ::error::MpError;
use ::header::{ChannelMode, Header};

pub struct FrameReader<R: io::Read + io::Seek> {
    reader: R,
}

impl<R: io::Read + io::Seek> FrameReader<R> {
    pub fn new(reader: R) -> FrameReader<R> {
        FrameReader {
            reader: reader,
        }
    }

    fn check_sync(data: &[u8], mut offset: bool) -> (bool, Option<usize>) {
        for (index, byte) in data.iter().enumerate() {
            match offset {
                false if *byte == 0xFF => offset = true, // Check if the byte is set.
                true if 0xE0 & *byte == 0xE0 => return (true, Some(index)), // Check if the next 3 bits are set.
                _ => offset = false,
            }
        }
        (offset, None)
    }

    pub fn read(&mut self) -> Result<(), MpError> {
        let (index, header_data) = try!(self.read_until_header()); // get next header
        let header = try!(Header::construct(&header_data));
        println!("{:?}", header_data);
        println!("{:?}", header);

        let mut read = 4; // bytes ready read into a frame.
        
        let mut checksum = None;
        if header.protection() { // if the frame has a CRC-16 checksum.
            checksum = Some(try!(self.reader.read_u16::<BigEndian>()));
            read += 2;
        }

        let frame_length = header.frame_length();
        println!("frame length: {:?}", frame_length);

        let mut side_info_length = 32;
        if header.channel() == &ChannelMode::Mono {
            side_info_length = 17;
        }
        let mut side_info = vec![0x00; side_info_length];
        self.reader.read_exact(&mut side_info);

        Ok(())
    }

    // Returns the index the header was located and the header data.
    pub fn read_until_header(&mut self) -> Result<(usize, [u8; 4]), MpError> {
        let mut header_buffer: [u8; 4] = [0; 4];
        try!(self.reader.read_exact(&mut header_buffer));
        
        // early return for when the header was at the beginning of the buffer.
        if let Some(index) = FrameReader::<R>::check_sync(&header_buffer, false).1 {
            if index == 1 { // if the header was at the beginning of the buffer
                return Ok((index - 1, header_buffer));
            }
        }

        let mut read_amount = 4;
        let read_limit = 150 * 1024; // 150kb
        let mut reader_buffer: [u8; 1024] = [0; 1024];
        let mut offset = false;
        loop {
            let read = try!(self.reader.read(&mut reader_buffer));
            read_amount += read;
            
            if read == 0 {
                return Err(MpError::NoCapture);
            }
            
            match FrameReader::<R>::check_sync(&reader_buffer, offset) {
                (true, Some(index)) => { // found the header sync.
                    let mut offset_buffer: [u8; 4] = [0xFF, reader_buffer[index], 0x00, 0x00];

                    // distance from end of buffer
                    let distance = reader_buffer.len() - index;

                    use std::cmp::min;
                    // bytes remaining in in the buffer
                    let remaining = min(2, (distance - 1));
                    for offset in 1..remaining + 1 {
                        offset_buffer[offset + 1] = reader_buffer[index + offset];
                    }

                    if distance > 3 {
                        // over shot the reader buffer, should seek back
                        self.reader.seek(SeekFrom::Current( -(distance as i64 - 3)));
                    }
                    else if distance == 3 { } // distance from end of buffer is the same as the remaining length of header.
                    else {
                        // bytes not in the current buffer
                        try!(self.reader.read_exact(&mut offset_buffer[remaining + 2..]));
                    }
                    
                    return Ok((index - 1, offset_buffer));
                },
                (found, None) => {
                    // if this is true then it found the initial byte of the header sync, but we need another byte to determine validity.
                    // if this isn't true then no header sync was found in this buffer.
                    offset = found;
                },
                _ => {
                    unreachable!()
                },
            }
            
            if read_amount > read_limit {
                return Err(MpError::NoCapture);
            }
        }
    }
}
