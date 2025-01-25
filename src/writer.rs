//! The `MidiWriteable` trait is central to translating data from miami’s internal representations
//! back into raw MIDI bytes. In other words, if `MidiReadable` is about parsing MIDI bytes *into*
//! Rust types, `MidiWriteable` does the opposite—taking those parsed types and converting them
//! into the canonical MIDI byte format. This is particularly useful when you have manipulated
//! or inspected MIDI data in your application and need to write it back to a file or stream.

use crate::Chunk;

/// A trait for types that can be encoded as MIDI-format bytes.
///
/// `MidiWriteable` is implemented by several primitive numeric types for convenience,
/// as well as by [`Chunk`] and other composite types in the `miami` library. This trait
/// makes it straightforward to transform parsed or constructed MIDI data back into a
/// binary stream for writing to a file or over a network connection.
pub trait MidiWriteable {
    /// Converts the data to a MIDI format byte sequence
    fn to_midi_bytes(self) -> Vec<u8>;
}

impl MidiWriteable for u8 {
    fn to_midi_bytes(self) -> Vec<u8> {
        vec![self]
    }
}

impl MidiWriteable for i8 {
    fn to_midi_bytes(self) -> Vec<u8> {
        vec![self.to_be_bytes()[0]]
    }
}

impl MidiWriteable for u16 {
    fn to_midi_bytes(self) -> Vec<u8> {
        let bytes = self.to_be_bytes();
        vec![bytes[0], bytes[1]]
    }
}

impl MidiWriteable for u32 {
    fn to_midi_bytes(self) -> Vec<u8> {
        let bytes = self.to_be_bytes();
        vec![bytes[0], bytes[1], bytes[2], bytes[3]]
    }
}

impl MidiWriteable for u64 {
    fn to_midi_bytes(self) -> Vec<u8> {
        let bytes = self.to_be_bytes();
        vec![
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]
    }
}

impl MidiWriteable for [char; 4] {
    fn to_midi_bytes(self) -> Vec<u8> {
        vec![self[0] as u8, self[1] as u8, self[2] as u8, self[3] as u8]
    }
}

impl MidiWriteable for Chunk {
    fn to_midi_bytes(self) -> Vec<u8> {
        let Chunk { chunk_type, length } = self;

        let mut chunk_type_bytes = chunk_type.to_midi_bytes();
        let len_bytes = length.to_midi_bytes();

        chunk_type_bytes.extend(len_bytes.iter());

        chunk_type_bytes
    }
}

impl MidiWriteable for (Chunk, Vec<u8>) {
    fn to_midi_bytes(self) -> Vec<u8> {
        let mut bytes = self.0.to_midi_bytes();
        bytes.extend(self.1.iter());

        bytes
    }
}

impl MidiWriteable for String {
    fn to_midi_bytes(self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        chunk::ParsedChunk,
        reader::{MidiReadable, MidiStream},
        Chunk,
    };

    use super::MidiWriteable;

    #[test]
    fn header_chunk_saves_as_proper_bytes() {
        let mut stream = "test/test.mid"
            .get_midi_bytes()
            .expect("Get MIDI bytes from source");
        let expected = stream
            .read_chunk_data_pair()
            .map(|val| ParsedChunk::try_from(val))
            .unwrap()
            .unwrap();

        let unparsed: (Chunk, Vec<u8>) = expected.clone().into();
        let bytes = unparsed.to_midi_bytes();

        let mut new_stream = bytes.into_iter();
        let new_header = new_stream
            .read_chunk_data_pair()
            .map(|val| ParsedChunk::try_from(val))
            .unwrap()
            .unwrap();

        assert_eq!(expected, new_header)
    }
}
