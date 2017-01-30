
pub enum Version {
    Reserved,
    Version2_5, // unofficial version for very low bitrate files
    Version2, // MPEG Version 2 (ISO/IEC 13818-3)
    Version1, // MPEG Version 1 (ISO/IEC 11172-3)
}

pub enum Layer {
    Reserved,
    Layer3,
    Layer2,
    Layer1,
}

pub enum ChannelMode {
    Stereo,
    JointStereo,
    Dual, // 2 mono
    Mono,
}

pub struct Header {
    sync: Sync,
    version: Version,
    layer: Layer,
    protection: bool,
    bitrate: u16, 
    sampling_rate: u16,
    padding: bool,
    private: bool,
    channel: ChannelMode,
}
