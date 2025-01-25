//! Chunk Definitions for parsed types and type headers

use header::{HeaderChunk, InvalidFormat};
use thiserror::Error;
use track::TrackChunk;

use crate::{
    chunk::chunk_types::{HEADER_CHUNK, TRACK_DATA_CHUNK},
    writer::MidiWriteable,
    Chunk,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod chunk_types;
pub mod header;
pub mod track;

/// Represents a parsed MIDI Chunk with its associated data.
/// A parsed chunk is classified based on its type, such as header or track.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ParsedChunk {
    /// A header chunk
    Header(HeaderChunk),
    /// A track chunk,
    Track(TrackChunk),
}

/// Error type for attempting to parse from a raw chunk to a parsed one
#[derive(Debug, Error)]
pub enum ChunkParseError {
    /// Invalid format in parsing a header
    #[error("Invalid Format Specified")]
    InvalidFormat(#[from] InvalidFormat),
    /// Type tag is not registered
    #[error("Unknown Chunk Type")]
    UnknownType,
    /// Random todo during debugging
    #[error("Development TODO")]
    Todo(&'static str),
    /// Error parsing track
    #[error("Track parsing error")]
    TrackParseError(#[from] track::TrackError),
}

impl From<ParsedChunk> for (Chunk, Vec<u8>) {
    fn from(value: ParsedChunk) -> Self {
        match value {
            ParsedChunk::Header(header) => {
                let bytes = header.to_midi_bytes();
                let chunk = Chunk {
                    chunk_type: HEADER_CHUNK,
                    length: bytes.len() as u32,
                };

                (chunk, bytes)
            }
            ParsedChunk::Track(track) => {
                let mut bytes = vec![];

                for mtrk_event in track.mtrk_events {
                    bytes.extend(mtrk_event.to_midi_bytes().iter());
                }

                let chunk = Chunk {
                    chunk_type: TRACK_DATA_CHUNK,
                    length: bytes.len() as u32,
                };
                (chunk, bytes)
            }
        }
    }
}

impl TryFrom<(Chunk, Vec<u8>)> for ParsedChunk {
    type Error = ChunkParseError;
    fn try_from(value: (Chunk, Vec<u8>)) -> Result<Self, Self::Error> {
        let (chunk, data) = value;

        match chunk.chunk_type {
            HEADER_CHUNK => {
                if chunk.len() == 6 {
                    let format = u16::from_be_bytes([data[0], data[1]]);
                    let ntrk = u16::from_be_bytes([data[2], data[3]]);
                    let division = u16::from_be_bytes([data[4], data[5]]);
                    let parsed = HeaderChunk::try_from((format, ntrk, division))?;
                    Ok(ParsedChunk::Header(parsed))
                } else {
                    Err(ChunkParseError::InvalidFormat(InvalidFormat))
                }
            }

            TRACK_DATA_CHUNK => {
                let parsed = TrackChunk::try_from(data)?;
                Ok(ParsedChunk::Track(parsed))
            }

            _ => Err(ChunkParseError::UnknownType),
        }
    }
}
