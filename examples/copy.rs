//! Example program that reads the entirety of a MIDI file as raw chunks and writes it to a second
//! file to test byte writing

use miami::{reader::MidiReadable, writer::MidiWriteable};
use miami::{Midi, RawMidi};
use std::fs::File;
use std::io::Write;

fn main() {
    let mut output = File::create("test/test_run.mid").expect("Create new output file");
    let data = "test/run.mid"
        .get_midi_bytes()
        .expect("Get `run.midi` file and stream bytes");

    let midi: Midi = RawMidi::try_from_midi_stream(data)
        .expect("Parse data as a MIDI stream")
        .check_into_midi()
        .expect("Sanitize MIDI into formatted MIDI");

    output
        .write_all(&midi.to_midi_bytes())
        .expect("Failed to write bytes");
}
