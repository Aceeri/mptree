
// |----------------|-------|-------|
// | scale compress | slen1 | slen2 |
// |----------------|-------|-------|
// | 0              | 0     | 0     |
// | 1              | 0     | 1     |
// | 2              | 0     | 2     |
// | 3              | 0     | 3     |
// | 4              | 3     | 0     |
// | 5              | 1     | 1     |
// | 6              | 1     | 2     |
// | 7              | 1     | 3     |
// | 8              | 2     | 1     |
// | 9              | 2     | 2     |
// | 10             | 2     | 3     |
// | 11             | 3     | 1     |
// | 12             | 3     | 2     |
// | 13             | 3     | 3     |
// | 14             | 4     | 2     |
// | 15             | 4     | 3     |
// |----------------|-------|-------|
pub static SCALE_COMPRESS: [(u8, u8); 15] = [
    (0, 0),
    (0, 1),
    (0, 2),
    (0, 3),
    (3, 0),
    (1, 1),
    (1, 2),
    (1, 3),
    (2, 1),
    (2, 2),
    (2, 3),
    (3, 1),
    (3, 2),
    (3, 3),
    (4, 2),
    (4, 3),
];

pub struct SideInformation {
    main_data_begin: u16, // Negative offset to where the audio data begins, ignore static parts of frames.
    private_bits: u8,  // Private use bits.
    scfsi: u8, // ScaleFactor Selection Information.
    par2_3_length: u32, // Number of bits allocated for scalefactors and Huffman encoded data.
    big_values: u32,
    global_gain: u16, // Quantization step size.
    scalefactor_compress: u8, // Number of bits used for the transmission of scalefactors.
    windows_switching: u8,
    block_type: u8,
    mixed_blockflag: u8,
    table_select: u32,
    subblock_gain: u32,
    region0_count: u8,
    region1_count: u8,
    preflag: u8,
    scalefactor_scale: u8,
    count1_table_select: u8,  // Specifies which count1 region Huffman code table applies.
}
