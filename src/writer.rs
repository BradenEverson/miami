//! Utility methods for writing a MIDI file

use crate::Chunk;

/// A chunk of data that can be converted to MIDI format bytes
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
