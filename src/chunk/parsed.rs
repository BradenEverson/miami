//! Parsed Chunk Enum type

use header::{HeaderChunk, InvalidFormat};

use crate::{
    chunk::chunk_types::{HEADER_CHUNK, TRACK_DATA_CHUNK},
    Chunk,
};

pub mod header;

/// A chunk's type paired with it's data
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParsedChunk {
    /// A header chunk
    Header(HeaderChunk),
}

/// Error type for attempting to parse from a raw chunk to a parsed one
#[derive(Debug)]
pub enum ChunkParseError {
    /// Invalid format in parsing a header
    InvalidFormat(InvalidFormat),
    /// Type tag is not registered
    UnknownType,
    /// Random todo during debugging
    Todo(&'static str),
}

impl TryFrom<(Chunk, Vec<u8>)> for ParsedChunk {
    type Error = ChunkParseError;
    fn try_from(value: (Chunk, Vec<u8>)) -> Result<Self, Self::Error> {
        let (chunk, data) = value;

        match chunk.chunk_type {
            HEADER_CHUNK => {
                if chunk.len() == 6 {
                    let format = u16::from_be_bytes([data[0], data[1]].try_into().unwrap());
                    let ntrk = u16::from_be_bytes([data[2], data[3]].try_into().unwrap());
                    let division = u16::from_be_bytes([data[4], data[5]].try_into().unwrap());
                    if let Ok(header_chunk) = HeaderChunk::try_from((format, ntrk, division)) {
                        return Ok(ParsedChunk::Header(header_chunk));
                    }
                }

                Err(ChunkParseError::InvalidFormat(InvalidFormat))
            }

            TRACK_DATA_CHUNK => Err(ChunkParseError::Todo("Parse Track Data Chunk")),

            _ => Err(ChunkParseError::UnknownType),
        }
    }
}
