//! MIDI file reader trait, allows for in memory byte spans to be read or files

use std::{
    convert::Infallible,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

/// Trait that allows certain amount of bytes to be yielded by an iterator
pub trait Yieldable<T> {
    /// Gets a certain number of elements while advancing the iterator
    fn get(&mut self, n: usize) -> Vec<T>;
}

impl<ITER: Iterator> Yieldable<ITER::Item> for ITER {
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
