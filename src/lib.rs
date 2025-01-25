//! # miami
//!
//! A minimal dependency MIDI file parser designed for both standard and WASM targets,
//! This crate provides core MIDI "chunks" and utilities for reading and parsing them,
//! without introducing any extra overhead or dependencies.
//!
//! ## Overview
//!
//! MIDI files are structured as a series of chunks. Each chunk contains a 4-character ASCII
//! type identifier and a 32-bit length that specifies how many bytes of data follow. The
//! `Chunk` struct and related APIs in this crate make it straightforward to inspect and
//! parse these sections of a MIDI file.
//!
//! - **Minimal dependencies**: Keeps your application lightweight and minimizes build complexity.
//!     Opt in to serde support and only require `thiserror` by default
//! - **Streaming-friendly**: Exposes traits and functions that can parse MIDI data from any
//!   implementor of [`reader::MidiStream`], making it easier to handle data on the fly.
//!
//! ## Example Usage
//!
//! ```rust
//! use miami::{
//!     chunk::ParsedChunk,
//!     reader::{MidiReadable, MidiStream},
//! };
//!
//! // Load MIDI bytes (replace with your own source as needed).
//! let mut data = "test/test.mid"
//!     .get_midi_bytes()
//!     .expect("Get `test.mid` file and read bytes");
//!
//! // Continuously read chunk-type/data pairs from the stream.
//! // Each read returns an option containing the chunk plus its raw data.
//! while let Some(parsed) = data.read_chunk_data_pair().map(ParsedChunk::try_from) {
//!     match parsed {
//!         Ok(chunk) => println!("Parsed chunk: {:?}", chunk),
//!         Err(e) => eprintln!("Failed to parse chunk: {e}"),
//!     }
//! }
//! ```
//!
//! The above example illustrates how to read chunks from a MIDI stream and use
//! [`ParsedChunk::try_from`] to parse them into known types (header or track chunks).
//!
//! ## Library Structure
//!
//! - **[`chunk`]**: Contains the [`Chunk`] struct and associated utilities for identifying
//!   chunk types and lengths.
//! - **[`reader`]**: Provides traits and types for streaming MIDI data. The [`MidiStream`]
//!   trait and related helpers allow on-the-fly parsing from any data source.
//! - **`chunk_types`, `header`, and `track`**: Provide definitions for recognized MIDI
//!   chunk types (e.g., `MThd` for the header and `MTrk` for track data) and the logic for
//!   parsing their contents.
//!
//! ## Extensibility
//!
//! While this crate focuses on parsing the structural aspects of MIDI files (chunks and headers),
//! you can use the raw track data to implement custom handling of MIDI events or other logic
//! as needed. Because `miami` exposes chunks in a straightforward format, you remain in full
//! control of the MIDI event parsing layer.
//!

pub mod chunk;
pub mod reader;
pub mod writer;

use chunk::{header::HeaderChunk, track::TrackChunk, ChunkParseError, ParsedChunk};
use reader::MidiStream;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// An entire MIDI file as a raw sequence of parsed chunks
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RawMidi {
    /// All raw chunks as ParsedChunks
    chunks: Vec<ParsedChunk>,
}

/// A MIDI File "cleaned" by enforcing a single header chunk and an arbitrary amount of Track
/// chunks
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Midi {
    /// The header chunk
    header: HeaderChunk,
    /// All subsequent track chunks
    tracks: Vec<TrackChunk>,
}

/// An error that may occur when verifying that a Raw Midi struct is sanitized into a clean MIDI
/// format
#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum MidiSanitizerError {
    /// Sequence doesn't start with a header
    #[error("First ParsedChunk in sequence isn't a header")]
    NoStartHeader,
    /// Too many headers
    #[error("More than one header chunk identified")]
    TooManyHeaders,
    /// No chunks at all
    #[error("No chunks present")]
    NoChunks,
}

impl TryFrom<RawMidi> for Midi {
    type Error = MidiSanitizerError;
    fn try_from(value: RawMidi) -> Result<Self, Self::Error> {
        let mut chunks = value.chunks.into_iter();
        let first = chunks.next().ok_or(MidiSanitizerError::NoChunks)?;
        let header = match first {
            ParsedChunk::Header(header) => header,
            _ => return Err(MidiSanitizerError::NoStartHeader),
        };
        let mut tracks = vec![];

        for track in chunks {
            match track {
                ParsedChunk::Track(track) => tracks.push(track),
                _ => return Err(MidiSanitizerError::TooManyHeaders),
            }
        }

        Ok(Self { header, tracks })
    }
}

impl RawMidi {
    /// Constructs a new MIDI instance from a stream of MIDI bytes
    pub fn try_from_midi_stream<STREAM>(stream: STREAM) -> Result<Self, ChunkParseError>
    where
        STREAM: MidiStream,
    {
        Self::try_from(StreamWrapper(stream))
    }
}

/// A wrapper to allow TryFrom implementations for `MidiStream` implementors
pub struct StreamWrapper<STREAM>(STREAM)
where
    STREAM: MidiStream;
impl<STREAM> TryFrom<StreamWrapper<STREAM>> for RawMidi
where
    STREAM: MidiStream,
{
    type Error = ChunkParseError;
    fn try_from(value: StreamWrapper<STREAM>) -> Result<Self, Self::Error> {
        let mut data = value.0;
        let mut chunks = vec![];

        while let Some(parsed) = data.read_chunk_data_pair().map(ParsedChunk::try_from) {
            let parsed = parsed?;
            chunks.push(parsed);
        }

        Ok(Self { chunks })
    }
}

/// Represents a raw MIDI Chunk.
/// A MIDI Chunk consists of a 4-character ASCII type identifier and a 32-bit unsigned integer specifying the length of its data.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Chunk {
    /// 4 character ASCII chunk type
    pub chunk_type: [char; 4],
    /// Length of the data that follows
    length: u32,
}

impl Chunk {
    /// Gets the length of the chunk as a usize
    pub fn len(&self) -> usize {
        self.length as usize
    }

    /// Returns if the chunk has no attributed data
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
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
            length: low,
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
            length: 10,
        };

        assert_eq!(expected, message.into())
    }
}
