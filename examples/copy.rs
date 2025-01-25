//! Example program that reads the entirety of a MIDI file as raw chunks and writes it to a second
//! file to test byte writing

use miami::{
    chunk::ParsedChunk,
    reader::{MidiReadable, MidiStream},
    writer::MidiWriteable,
};
use std::fs::File;
use std::io::Write;

fn main() {
    let mut output = File::create("test/test_run.mid").expect("Create new output file");
    let mut data = "test/run.mid"
        .get_midi_bytes()
        .expect("Get `run.midi` file and stream bytes");

    while let Some(Ok(parsed)) = data
        .read_chunk_data_pair()
        .map(|val| ParsedChunk::try_from(val))
    {
        let bytes = parsed.to_midi_bytes();
        output.write(&bytes).expect("Failed to write bytes");
    }
}
