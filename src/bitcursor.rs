
pub struct BitCursor<'a> {
    slice: &'a [u8],
    bit: usize
}

impl<'a> BitCursor<'a> {
    pub fn new(slice: &'a [u8]) -> BitCursor<'a> {
        BitCursor {
            slice: slice,
            bit: 0
        }
    }

    pub fn set_offset(&mut self, bit: usize) {
        self.bit = bit;
    }

    pub fn add_offset(&mut self, bit: usize) {
        self.bit += bit;
    }

    pub fn read_bits(&mut self, bits: usize) -> u64 {
        let mut output = 0;
        for i in self.bit..(self.bit + bits) {
            let byte = self.slice.get(i/8).cloned().unwrap_or(0);
            let b = (byte >> (7 - i%8)) & 1;
            output = (output << 1) | (b as u64);
        }
        self.add_offset(bits);
        output
    }
    
    pub fn range(&mut self, start: usize, end: usize) -> u64 {
        self.set_offset(start);
        self.read_bits(end - start)
    }
}

