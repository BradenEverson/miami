//! Example program that reads the entirety of a MIDI file as raw chunks

use miami::{
    chunk::ParsedChunk,
    reader::{MidiReadable, MidiStream},
};

fn main() {
    let mut data = "test/run.mid"
        .get_midi_bytes()
        .expect("Get `run.midi` file and stream bytes");

    while let Some(parsed) = data
        .read_chunk_data_pair()
        .map(|val| ParsedChunk::try_from(val))
    {
        println!("{parsed:?}")
    }
}
