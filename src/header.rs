
use ::error::MpError;

// Bitrate Lookup Table
// |----|--------|--------|--------|-------------|
// |bits| V1, L1 | V1, L2 | V1, L3 | V2, L2 & L3 |
// |----|--------|--------|--------|-------------|
// |0000|free    |free    |free    |free         |
// |0001|32      |32      |32      |8            |
// |0010|64      |48      |40      |16           |
// |0011|96      |56      |48      |24           |
// |0100|128     |64      |56      |32           |
// |0101|160     |80      |64      |40           |
// |0110|192     |96      |80      |48           |
// |0111|224     |112     |96      |56           |
// |1000|256     |128     |112     |64           |
// |1001|288     |160     |128     |80           |
// |1010|320     |192     |160     |96           |
// |1011|352     |224     |192     |112          |
// |1100|384     |256     |224     |128          |
// |1101|416     |320     |256     |144          |
// |1110|448     |384     |320     |160          |
// |1111|bad     |bad     |bad     |bad          |
// |----|--------|--------|--------|-------------|
pub static BITRATE_INDEX: [[u16; 15]; 5] = [
    [0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448], // Version 1, Layer 1
    [0, 32, 48, 56,  64,  80,  96, 112, 128, 160, 192, 224, 256, 320, 384], // Version 1, Layer 2
    [0, 32, 40, 48,  56,  64,  80,  96, 112, 128, 160, 192, 224, 256, 320], // Version 1, Layer 3
    [0, 32, 48, 56,  64,  80,  96, 112, 128, 144, 160, 176, 192, 224, 256], // Version 2, Layer 2
    [0,  8, 16, 24,  32,  40,  48,  56,  64,  80,  96, 112, 128, 144, 160], // Version 2, Layer 2 & Layer 3
];

// Sampling Rate Frequency Index (Hz)
// |------|-------|-------|---------|
// | bits | MPEG1 | MPEG2 | MPEG2.5 |
// |------|-------|-------|---------|
// |  00  | 44100 | 22050 | 11025   |
// |  01  | 48000 | 24000 | 12000   |
// |  10  | 32000 | 16000 | 8000    |
// |  11  |reserv.|reserv.| reserv. |
// |------|-------|-------|---------|
pub static SAMPLING_RATE: [[u16; 3]; 3] = [
    [44100, 48000, 32000],
    [22050, 24000, 16000],
    [11025, 12000, 8000],
];

// Returns whether the value was free in the lookup table and what the bitrate was if it exists.
pub fn bitrate(bit: u8, version: &Version, layer: &Layer, mode: &ChannelMode) -> Result<u16, MpError> {
    if version == &Version::Reserved || layer == &Layer::Reserved {
        return Err(MpError::Reserved);
    }

    let column_index = match (version, layer) {
        (&Version::Version1, &Layer::Layer1) => 0,
        (&Version::Version1, &Layer::Layer2) => 1,
        (&Version::Version1, &Layer::Layer3) => 2,
        (_, &Layer::Layer1) => 3,
        (_, _) => 4,
    };

    if bit > 15 {
        return Err(MpError::BadBit(bit as u16)); // bit index was too high
    }

    let bitrate = BITRATE_INDEX[column_index][bit as usize];
    if layer == &Layer::Layer2 {
        // if the bitrate is 32, 48, 56, or 80 and the channel is not mono, then invalid
        if ((bitrate >= 32 && bitrate <= 56) || (bitrate == 80)) && mode != &ChannelMode::Mono {
            return Err(MpError::InvalidMode(mode.clone(), vec![ChannelMode::Mono]));
        }

        if bitrate >= 224 && mode == &ChannelMode::Mono {
            return Err(MpError::InvalidMode(mode.clone(), vec![ChannelMode::Dual, ChannelMode::Stereo, ChannelMode::JointStereo]));
        }
    }

    Ok(bitrate)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Version {
    Reserved,
    Version2_5, // unofficial version for very low bitrate files
    Version2, // MPEG Version 2 (ISO/IEC 13818-3)
    Version1, // MPEG Version 1 (ISO/IEC 11172-3)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Layer {
    Reserved,
    Layer3,
    Layer2,
    Layer1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChannelMode {
    Stereo,
    JointStereo,
    Dual, // 2 mono
    Mono,
}

pub struct Header {
    version: Version, // Version 1-2 (along with the unofficial 2.5)
    layer: Layer, // Layer 1-3
    protection: bool, // Protected by a 16 bit CRC following header
    bitrate: u16, // Bitrate (kbps)
    sampling_rate: u16, // Sampling rate frequency index (Hz)
    padding: bool, // Frame padding for one extra slot
    private: bool, // Freely used for applications
    channel: ChannelMode, // Mono, Dual, Stereo, JointStereo
    channel_ext: Option<(bool, bool)>, // Intensity stereo / MS stereo 
}
