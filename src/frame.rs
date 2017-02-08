
use ::bitcursor::BitCursor;
use ::error::MpError;
use ::header::{ChannelMode, Header};

#[derive(Debug, Clone)]
pub struct Frame {
    header: Header, 
    size: u16, // Size in bytes how long the frame is.

    main_data_begin: u16, // Negative offset to where the audio data begins, ignore static parts of frames.
    scfsi: [[u8; 4]; 2], // ScaleFactor Selection Information.

    granules: [Granule; 2],
    /*
    // 2 granules, 2 channels
    granules: [Granule; 2],
    part2_3_length: [[u32; 2]; 2], // Number of bits allocated for scalefactors and Huffman encoded data.
    big_values: [[u32; 2]; 2],
    global_gain: [[u16; 2]; 2], // Quantization step size.
    scalefactor_compress: [[u8; 2]; 2], // Number of bits used for the transmission of scalefactors.
    windows_switching: [[u8; 2]; 2],
    block_type: [[u8; 2]; 2],
    mixed_blockflag: [[u8; 2]; 2],
    table_select: [[[u32; 3]; 2]; 2],
    subblock_gain: [[[u32; 3]; 2]; 2],
    region0_count: [[u8; 2]; 2],
    region1_count: [[u8; 2]; 2],
    preflag: [[u8; 2]; 2],
    scalefactor_scale: [[u8; 2]; 2],
    count1_table_select: [[u8; 2]; 2],  // Specifies which count1 region Huffman code table applies.
    */
}

impl Frame {
    pub fn new(header: Header, length: u16, data: &[u8]) -> Result<Frame, MpError> {
        let mut cursor = BitCursor::new(data);
        let mono = header.channel() == &ChannelMode::Mono;

        // CRC-16 protection checksum
        if header.protection() {
            /*
            if !Frame::checksum(data) {
                return Err(MpError::InvalidChecksum);
            }

            cursor.set_offset(16);
            */
        }

        // Private bits
        cursor.add_offset(if mono { 5 } else { 3 });

        let mut granules = [Granule::new(); 2];
        let mut scsfi = [[0; 4]; 2];
        let channel_range = if mono { 1 } else { 2 };

        let main_data_begin = cursor.read_bits(9) as u16;

        for ch in 0..channel_range {
            for band in 0..4 {
                scsfi[ch][band] = cursor.read_bits(1) as u8;
            }
        }

        for gr in 0..2 {
            for ch in 0..channel_range {
                // Length of the scaling factors and main data in bits.
                granules[gr].part2_3_length[ch] = cursor.read_bits(12) as u32;

                // Number of values in the big region.
                granules[gr].big_values[ch] = cursor.read_bits(9) as u32;

                // Quantization step size.
                granules[gr].global_gain[ch] = cursor.read_bits(8) as u16;
                
                granules[gr].scalefactor_compress[ch] = cursor.read_bits(4) as u8;

                granules[gr].windows_switching[ch] = cursor.read_bits(1) as u8;

                if granules[gr].windows_switching[ch] == 1 {
                    // Window type for the granules[gr]
                    granules[gr].block_type[ch] = cursor.read_bits(2) as u8;

                    // Number of scale factor bands before window switching
                    granules[gr].mixed_blockflag[ch] = cursor.read_bits(1) == 1;

                    // Regions set by default if window switching
                    granules[gr].region0_count[ch] = if granules[gr].block_type[ch] == 2 { 8 } else { 7 };
                    granules[gr].region1_count[ch] = 20 - granules[gr].region0_count[ch];
                    
                    // Huffman table number for big regions
                    for region in 0..2 {
                        granules[gr].table_select[ch][region] = cursor.read_bits(5) as u32;
                    }

                    for window in 0..3 {
                        granules[gr].subblock_gain[ch][window] = cursor.read_bits(3) as u32;
                    }
                }
                else {
                    granules[gr].block_type[ch] = 0;
                    granules[gr].mixed_blockflag[ch] = false;
                    
                    for region in 0..3 {
                        granules[gr].table_select[ch][region] = cursor.read_bits(5) as u32;
                    }

                    // Number of scale factor bands in the first big value region.
                    granules[gr].region0_count[ch] = cursor.read_bits(4) as u8;
                    
                    // Number of scale factor bands in the third big value region.
                    granules[gr].region1_count[ch] = cursor.read_bits(3) as u8;
                }

                granules[gr].preflag[ch] = cursor.read_bits(1) as u8;

                granules[gr].scalefactor_scale[ch] = cursor.read_bits(1) as u8;

                granules[gr].count1table_select[ch] = cursor.read_bits(1) as u8;
            }
        }

        /*
        // get basic side information (main_data_begin, scsfi)
        let main_data_begin: u16 = ((data[cursor] as u16) << 1) | ((data[cursor + 1] >> 7) as u16); // Concatenate 9 bits from two bytes

        let scsfi = if mono {
            let band1 = data[cursor + 1] & 0b0000_0010 >> 1;
            let band2 = data[cursor + 1] & 0b0000_0001;
            let band3 = data[cursor + 2] & 0b1000_0000 >> 7;
            let band4 = data[cursor + 2] & 0b0100_0000 >> 6;
            [[band1, band2, band3, band4], [0; 4]]
        }
        else {
            let ch1 = {
                let band1 = data[cursor + 1] & 0b0000_1000 >> 3;
                let band2 = data[cursor + 1] & 0b0000_0100 >> 2;
                let band3 = data[cursor + 1] & 0b1000_0010 >> 1;
                let band4 = data[cursor + 1] & 0b0100_0001;
                [band1, band2, band3, band4]
            };

            let ch2 = {
                let band1 = data[cursor + 2] & 0b1000_0000 >> 7;
                let band2 = data[cursor + 2] & 0b0100_0000 >> 6;
                let band3 = data[cursor + 2] & 0b0010_0000 >> 5;
                let band4 = data[cursor + 2] & 0b0001_0000 >> 4;
                [band1, band2, band3, band4]
            };

            [ch1, ch2]
        };
        */
        
        Ok(Frame {
            header: header,
            size: length,
            
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
