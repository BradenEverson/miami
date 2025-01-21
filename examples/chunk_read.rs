//! Example program that reads the entirety of a MIDI file as raw chunks

use miami::reader::{MidiReadable, MidiStream};

fn main() {
    let mut data = "test/run.mid"
        .get_midi_bytes()
        .expect("Get `run.midi` file and stream bytes");

    while let Some((chunk, data)) = data.read_chunk_data_pair() {
        println!("{:?} | {:?}", chunk, data);
    }
}
