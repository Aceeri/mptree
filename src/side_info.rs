
use ::bitcursor::BitCursor;
use ::error::MpError;
use ::header::{ChannelMode, Header};

use bitstream_io::{BitReader, BigEndian};

// Technically this is 1441 - side info size (mono) - HEADER_SIZE if you don't support MPEG 2_5 Layer 3, otherwise the maximum here is
// 2881 if you have a MPEG 2_5 Layer 3 with a bitrate of 160kbps and a sampling rate of 8000Hz.
// 
// I'm leaving a bit of wiggle room here just for weird purposes.
pub const MAX_FRAME_SIZE: u16 = 3000;

#[derive(Debug, Clone)]
pub struct SideInformation {
    main_data_size: u16, // Size in bytes how long the main data is.
    main_data_begin: u16, // Negative offset to where the audio data begins, ignore static parts of frames.
    scfsi: [[u8; 4]; 2], // SCaleFactor Selection Information.

    granules: [Granule; 2],
}

impl SideInformation {
    pub fn new(header: &Header, data: &[u8]) -> Result<SideInformation, MpError> {
        let mut reader = BitReader::endian(data, BigEndian);

        let mono = header.channel() == &ChannelMode::Mono;

        let channel_count = if mono { 1 } else { 2 };
        let private_bits = if mono { 5 } else { 3 };

        let side_info_size = if mono { 17 } else { 32 };
        let mut main_data_size = header.frame_size() - side_info_size - ::header::HEADER_SIZE as u16;
        
        if main_data_size > MAX_FRAME_SIZE {
            return Err(MpError::InvalidData(format!("Frame size too large (>2000): {}", main_data_size)));
        }

        if header.protection() {
            main_data_size -= ::header::CHECKSUM_SIZE as u16;
        }

        let mut granules = [Granule::new(); 2];
        let mut scsfi = [[0; 4]; 2];

        let main_data_begin = reader.read(9)?;

        // Skip private bits
        reader.skip(private_bits)?;

        for ch in 0..channel_count {
            for band in 0..4 {
                scsfi[ch][band] = reader.read(1)?;
            }
        }

        for gr in 0..2 {
            for ch in 0..channel_count {
                granules[gr].part2_3_length[ch] = reader.read(12)?;
                granules[gr].big_values[ch] = reader.read(9)?;
                granules[gr].global_gain[ch] = reader.read(8)?;
                granules[gr].scalefactor_compress[ch] = reader.read(4)?;
                granules[gr].windows_switching[ch] = reader.read(1)?;

                if granules[gr].windows_switching[ch] == 1 {
                    granules[gr].block_type[ch] = reader.read(2)?;
                    granules[gr].mixed_blockflag[ch] = reader.read::<u8>(1)? == 1;

                    for region in 0..2 {
                        granules[gr].table_select[ch][region] = reader.read(5)?;
                    }

                    for window in 0..3 {
                        granules[gr].subblock_gain[ch][window] = reader.read(3)?;
                    }

                    granules[gr].region0_count[ch] = if granules[gr].block_type[ch] == 2 {
                        8
                    } else {
                        7
                    };

                    // Standard is wrong here apparently...
                    granules[gr].region1_count[ch] = 20 - granules[gr].region0_count[ch];
                }
                else {
                    for region in 0..3 {
                        granules[gr].table_select[ch][region] = reader.read(5)?;
                    }

                    granules[gr].block_type[ch] = 0;
                    granules[gr].mixed_blockflag[ch] = false;
                    granules[gr].region0_count[ch] = reader.read(4)?;
                    granules[gr].region1_count[ch] = reader.read(3)?;
                }

                granules[gr].preflag[ch] = reader.read(1)?;

                granules[gr].scalefactor_scale[ch] = reader.read(1)?;

                granules[gr].count1table_select[ch] = reader.read(1)?;
            }
        }
        
        Ok(SideInformation {
            main_data_size: main_data_size,
            main_data_begin: main_data_begin,
            scfsi: scsfi,
            granules: granules,
        })
    }

    // Gets the checksum and checks if the frame is valid.
    pub fn checksum(data: &[u8]) -> bool {
        false
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Granule {
    part2_3_length: [u32; 2], // Number of bits allocated for scalefactors and Huffman encoded data.
    big_values: [u32; 2],
    global_gain: [u16; 2], // Quantization step size.
    scalefactor_compress: [u8; 2], // Number of bits used for the transmission of scalefactors.
    windows_switching: [u8; 2],
    block_type: [u8; 2],
    mixed_blockflag: [bool; 2],
    table_select: [[u32; 3]; 2],
    subblock_gain: [[u32; 3]; 2],
    region0_count: [u8; 2],
    region1_count: [u8; 2],
    preflag: [u8; 2],
    scalefactor_scale: [u8; 2],
    count1table_select: [u8; 2],  // Specifies which count1 region Huffman code table applies.
}

impl Granule {
    fn new() -> Granule {
        Granule {
            part2_3_length: [0; 2],
            big_values: [0; 2],
            global_gain: [0; 2],
            scalefactor_compress: [0; 2],
            windows_switching: [0; 2],
            block_type: [0; 2],
            mixed_blockflag: [false; 2],
            table_select: [[0; 3]; 2],
            subblock_gain: [[0; 3]; 2],
            region0_count: [0; 2],
            region1_count: [0; 2],
            preflag: [0; 2],
            scalefactor_scale: [0; 2],
            count1table_select: [0; 2],
        }
    }
}
