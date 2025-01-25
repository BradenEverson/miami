//! Chunk Definitions for parsed types and type headers

use header::{HeaderChunk, InvalidFormat};
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

impl MidiWriteable for ParsedChunk {
    fn to_midi_bytes(self) -> Vec<u8> {
        let val: (Chunk, Vec<u8>) = self.into();
        val.to_midi_bytes()
    }
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
    /// Error parsing track
    TrackParseError(track::TrackError),
}

impl core::error::Error for ChunkParseError {}
impl core::fmt::Display for ChunkParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidFormat(_) => write![f, "Invalid Format Specified"],
            Self::UnknownType => write![f, "Unknown Chunk Type"],
            Self::Todo(s) => write![f, "Development TODO: {s}"],
            Self::TrackParseError(_) => write![f, "Track parsing error"],
        }
    }
}
impl From<InvalidFormat> for ChunkParseError {
    fn from(f: InvalidFormat) -> Self {
        Self::InvalidFormat(f)
    }
}
impl From<track::TrackError> for ChunkParseError {
    fn from(f: track::TrackError) -> Self {
        Self::TrackParseError(f)
    }
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
