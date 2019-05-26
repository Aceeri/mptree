
use ::error::MpError;
use ::tables::{BITRATE_INDEX, SAMPLING_RATE};

use std::fmt::{self, Display, Formatter};

// MP3 headers are 4 bytes long.
pub const HEADER_SIZE: usize = 4;

#[derive(Debug, Clone, PartialEq)]
pub enum Version {
    Version2_5, // unofficial version for very low bitrate files
    Reserved,
    Version2, // MPEG Version 2 (ISO/IEC 13818-3)
    Version1, // MPEG Version 1 (ISO/IEC 11172-3)
}

impl From<u8> for Version {
    fn from(data: u8) -> Version {
        match data {
            0 => Version::Version2_5,
            1 => Version::Reserved,
            2 => Version::Version2,
            3 => Version::Version1,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Layer {
    Reserved,
    Layer3,
    Layer2,
    Layer1,
}

impl Display for Layer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Layer::Reserved => "Reserved",
            Layer::Layer3 => "Layer3",
            Layer::Layer2 => "Layer2",
            Layer::Layer1 => "Layer1",
        })
    }
}

impl From<u8> for Layer {
    fn from(data: u8) -> Layer {
        match data {
            0 => Layer::Reserved,
            1 => Layer::Layer3,
            2 => Layer::Layer2,
            3 => Layer::Layer1,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChannelMode {
    Stereo,
    JointStereo(Extension),
    Dual, // 2 mono
    Mono,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Extension {
    Bands(u8), // Bands (Minimum -> 31)
    Stereo(bool, bool), // Intensity stereo, MS stereo
}

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    version: Version, // Version 1-2 (along with the unofficial 2.5)
    layer: Layer, // Layer 1-3
    protection: bool, // Protected by a 16 bit CRC following header (0 = protected, 1 = not)
    bitrate: u16, // Bitrate (kbps)
    sampling_rate: u16, // Sampling rate frequency index (Hz)
    padding: u8, // Frame padding for one extra slot
    private: bool, // Freely used for applications
    copyright: bool, // Is copyrighted (0 = not, 1 = copyrighted)
    original: bool, // Is the original copy (0 = copy, 1 = original)
    emphasis: u8, // 00 = none, 01 = 50/15 ms, 10 = reserved, 11 = CCIT J.17, rarely used
    channel: ChannelMode, // Mono, Dual, Stereo, JointStereo
}

impl Header {
    pub fn new(data: &[u8]) -> Result<Header, MpError> {
        if data.len() != 4 {
            return Err(MpError::InvalidData(format!("Header is {} instead of {} bytes", data.len(), HEADER_SIZE)));
        }

        let version = ((data[1] & 0b0001_1000) >> 3).into();
        let layer: Layer = ((data[1] & 0b0000_0110) >> 1).into();
        let protection: bool = (data[1] & 0b0000_0001) == 0;
        let bitrate_index: u8 = (data[2] & 0b1111_0000) >> 4;
        let sampling_index: u8 = (data[2] & 0b0000_1100) >> 2;
        let padding: u8 = (data[2] & 0b0000_0010) >> 1;
        let private: bool = (data[2] & 0b0000_0001) == 1;
        let channel = match data[3] & 0b1100_0000 {
            0b00 => ChannelMode::Stereo,
            0b0100_0000 => {
                //let extension = data[3] & 0b0011_0000; // Joint Stereo extension

                let parsed_extension = match layer {
                    Layer::Layer3 => {
                        let intensity = data[3] & 0b0010_0000 == 0b0010_0000;
                        let ms = data[3] & 0b0001_0000 == 0b0001_0000;
                        Extension::Stereo(intensity, ms)
                    },
                    _ => {
                        // Just an mp3 decoder, let's not get into MPEG-1 Layer 1/2/Reserved
                        return Err(MpError::InvalidData(::error::wrong_layer(&layer)))
                        //let value = 4 + (4 * extension >> 4);
                        //Extension::Bands(value)
                    },
                };
                ChannelMode::JointStereo(parsed_extension)
            },
            0b1000_0000 => ChannelMode::Dual,
            0b1100_0000 => ChannelMode::Mono,
            channel_bits => return Err(MpError::InvalidData(format!("channel data: {:b}, got: {:b}", data[3] & 0b1100_0000, channel_bits))),
        };
        let copyright = (data[3] & 0b0000_1000) == 0b0000_1000;
        let original = (data[3] & 0b0000_0100) == 0b0000_0100;
        let emphasis = data[3] & 0b0000_0011;

        let bitrate = Header::lookup_bitrate(bitrate_index, &version, &layer, &channel)?;
        let sampling_rate = Header::lookup_sampling_rate(sampling_index, &version)?;

        Ok(Header {
            version: version,
            layer: layer,
            protection: protection,
            bitrate: bitrate,
            sampling_rate: sampling_rate,
            padding: padding,
            private: private,
            copyright: copyright,
            original: original,
            emphasis: emphasis,
            channel: channel,
        })
    }

    // Returns the bitrate of the header.
    pub fn lookup_bitrate(bit: u8, version: &Version, layer: &Layer, channel: &ChannelMode) -> Result<u16, MpError> {
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

        if bit > 14 {
            return Err(MpError::BadBit(bit as u16)); // bit index was too high
        }

        let bitrate = BITRATE_INDEX[column_index][bit as usize];
        Ok(bitrate)
    }

    // Returns the sampling rate from the version and the sampling index. Errors if takes in reserved values.
    pub fn lookup_sampling_rate(bit: u8, version: &Version) -> Result<u16, MpError> {
        if bit == 3 {
            Err(MpError::Reserved)
        } else {
            let index = match version {
                &Version::Version1 => 0,
                &Version::Version2 => 1,
                &Version::Version2_5 => 2,
                _ => return Err(MpError::Reserved),
            };
            Ok(SAMPLING_RATE[index][bit as usize])
        }
    }

    // Returns the frame length based on this header.
    pub fn frame_length(&self) -> u16 {
        (144 * (self.bitrate as u32 * 1000) / self.sampling_rate as u32 + self.padding as u32) as u16
    }

    #[inline]
    pub fn version(&self) -> &Version {
        &self.version
    }

    #[inline]
    pub fn layer(&self) -> &Layer {
        &self.layer
    }

    #[inline]
    pub fn protection(&self) -> bool {
        self.protection
    }

    #[inline]
    pub fn bitrate(&self) -> u16 {
        self.bitrate
    }

    #[inline]
    pub fn sampling_rate(&self) -> u16 {
        self.sampling_rate
    }

    #[inline]
    pub fn padding(&self) -> u8 {
        self.padding
    }

    #[inline]
    pub fn private(&self) -> bool {
        self.private
    }

    #[inline]
    pub fn copyright(&self) -> bool {
        self.copyright
    }

    #[inline]
    pub fn original(&self) -> bool {
        self.original
    }

    #[inline]
    pub fn channel(&self) -> &ChannelMode {
        &self.channel
    }
}

