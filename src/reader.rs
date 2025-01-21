//! MIDI file reader trait, allows for in memory byte spans to be read or files

use std::{
    convert::Infallible,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use crate::Chunk;

/// Trait that allows certain amount of bytes to be yielded by an iterator
pub trait Yieldable<T> {
    /// Gets a certain number of elements while advancing the iterator
    fn get(&mut self, n: usize) -> Vec<T>;
}

impl<ITER> Yieldable<ITER::Item> for ITER
where
    ITER: Iterator,
{
    #[allow(if_let_rescope)]
    fn get(&mut self, n: usize) -> Vec<ITER::Item> {
        let mut elements = Vec::with_capacity(n);
        for _ in 0..n {
            if let Some(item) = self.next() {
                elements.push(item);
            } else {
                break; // Stop if the iterator is exhausted
            }
        }
        elements
    }
}

/// Trait for reading sequential chunks from a MIDI stream
pub trait MidiStream {
    /// Reads the next chunk from the sequence and the data associated with it, fails if there
    /// isn't enough data left to read a full chunk or read a payload
    fn read_chunk_data_pair(&mut self) -> Option<(Chunk, Vec<u8>)>;
}

impl<MIDI> MidiStream for MIDI
where
    MIDI: Iterator<Item = u8>,
{
    fn read_chunk_data_pair(&mut self) -> Option<(Chunk, Vec<u8>)> {
        let chunk_packet = self.get(8);

        if chunk_packet.len() != 8 {
            return None;
        }

        // UNWRAP Safety: We verify the chunk packet is 8 bytes before
        let chunk = u64::from_be_bytes(chunk_packet.try_into().unwrap());
        let chunk: Chunk = chunk.into();

        let data = self.get(chunk.len());

        if data.len() != chunk.len() {
            return None;
        }

        Some((chunk, data))
    }
}

/// Trait that allows for different types to be translated to a MIDI parseable format
pub trait MidiReadable {
    /// Error type that may be returned from the Midi Sequence
    type Error;
    /// Creates a byte iterator from the type
    fn get_midi_bytes(self) -> Result<impl Iterator<Item = u8>, Self::Error>;
}

/// Wrapper struct to allow passing Vec<u8> to MidiReadable trait
pub struct MidiData(Vec<u8>);

impl MidiReadable for MidiData {
    type Error = Infallible;
    fn get_midi_bytes(self) -> Result<impl Iterator<Item = u8>, Self::Error> {
        Ok(self.0.into_iter())
    }
}

impl<PATH> MidiReadable for PATH
where
    PATH: AsRef<Path>,
{
    type Error = std::io::Error;
    fn get_midi_bytes(self) -> Result<impl Iterator<Item = u8>, Self::Error> {
        let path = self.as_ref();
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(reader.bytes().filter_map(Result::ok))
    }
}

#[cfg(test)]
mod tests {
    use super::MidiReadable;

    #[test]
    fn midi_files_stream() {
        let path = "test/run.mid";
        let data = path.get_midi_bytes();

        assert!(data.is_ok())
    }
}
