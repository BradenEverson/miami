//! A zero dependency MIDI file parser, with the ability to target no-std targets such as embedded
//! systems and webassembly

pub mod chunk;

/// A MIDI Chunk.
/// MIDI Chunks are composed of a 4 character type and a 32-bit length
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chunk {
    /// 4 character ASCII chunk type
    pub chunk_type: [char; 4],
    /// Length of the data that follows
    pub len: u32,
}

impl From<u64> for Chunk {
    fn from(value: u64) -> Self {
        let high = (value >> 32) as u32;
        let low = value as u32;

        let a = (high >> 24) as u8 as char;
        let b = (high >> 16) as u8 as char;
        let c = (high >> 8) as u8 as char;
        let d = high as u8 as char;

        Self {
            chunk_type: [a, b, c, d],
            len: low,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Chunk;

    #[test]
    fn chunk_from_raw_u64_behaves_normally() {
        let message = 0x74657374_0000000au64;
        let expected = Chunk {
            chunk_type: ['t', 'e', 's', 't'],
            len: 10,
        };

        assert_eq!(expected, message.into())
    }
}
