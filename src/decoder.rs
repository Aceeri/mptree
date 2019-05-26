
use std::io;

use ::error::MpError;
use ::header::{Header, ChannelMode};
use ::side_info::SideInformation;

use byteorder::ReadBytesExt;

pub struct FrameReader<R: io::Read + io::Seek> {
    // File (preferably buffered), stream, array, etc.
    reader: R,

    // Main data bit reservoir for future frames. If we start in the middle and require
    // main data from prior frames, then we should either error or load the previous frames
    // if possible.
    main_data: VecDeque<u8>,
}

// How many bytes ahead should we check before erroring on header seeking.
const HEADER_LIMIT: usize = 1024 * 10; // 10kB

impl<R: io::Read + io::Seek> FrameReader<R> {
    pub fn new(reader: R) -> FrameReader<R> {
        FrameReader {
            reader: reader,
            main_data: VecDeque::new(),
        }
    }

    pub fn advance(&mut self) -> Result<(), MpError> {
        let header = self.find_header(HEADER_LIMIT).ok_or(MpError::NoHeaderCapture)?;
        dbg!(header.clone());

        let side_information = self.construct_side_information(&header)?;
        dbg!(side_information.clone());
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

    fn construct_side_information(&mut self, header: &Header) -> Result<SideInformation, MpError> {
        let mut mono_buffer = [0u8; 17];
        let mut dual_buffer = [0u8; 32];
        let side_info_data: &[u8] = if header.channel() == &ChannelMode::Mono {
            self.reader.read_exact(&mut mono_buffer)?;
            &mono_buffer
        } else {
            self.reader.read_exact(&mut dual_buffer)?;
            &dual_buffer
        };

        print!("side_info_data: ");
        for byte in side_info_data {
            print!("{:08b} ", byte);
        }
        println!();

        Ok(SideInformation::new(&header, &side_info_data)?)
    }
}
