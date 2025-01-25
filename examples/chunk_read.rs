//! Example program that reads the entirety of a MIDI file as raw chunks

use miami::{reader::MidiReadable, Midi, RawMidi};

fn main() {
    let data = "test/test.mid"
        .get_midi_bytes()
        .expect("Get `run.midi` file and stream bytes");

    let midi = RawMidi::try_from_midi_stream(data).expect("Parse data as a MIDI stream");
    let sanitized_midi: Midi = midi.try_into().expect("Upgrade into sanitized format");

    println!("Header: {:?}", sanitized_midi.header);
    for chunk in sanitized_midi.tracks.iter() {
        println!("Track: {chunk:?}");
    }
}
