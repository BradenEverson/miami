//! Parsed Chunk Enum type

use header::HeaderChunk;

pub mod header;

/// A chunk's type paired with it's data
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParsedChunk {
    /// A header chunk
    Header(HeaderChunk),
}
