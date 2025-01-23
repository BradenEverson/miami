# Miami 🌴📼
*parsing midi files is not as scary as it sounds.* 🐔


A lightweight, zero-dependency MIDI file parser designed for resource-constrained environments. This works on no-std targets, making it great for embedded systems and WebAssembly applications.

## Features

- **No-std Compatibility**: Fully functional in environments without a standard library.
- **Efficient MIDI Parsing**: Parse MIDI chunks and their associated data with minimal overhead.
- **Error Handling**: Comprehensive error reporting for invalid or unsupported chunks.

## Getting Started

### Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
miami = "0.1"
```

For serde support include the `serde` feature flag ;)

### Example Usage

The following example demonstrates how to read and process MIDI chunks from a file:

```rust
use miami::{
    chunk::ParsedChunk,
    reader::{MidiReadable, MidiStream},
};

fn main() {
    let mut data = "path/to/midi/file.mid"
        .get_midi_bytes()
        .expect("Failed to load MIDI file");

    while let Some(parsed) = data
        .read_chunk_data_pair()
        .map(|val| ParsedChunk::try_from(val))
    {
        println!("{parsed:?}");
    }
}
```

## Core Concepts

### Raw MIDI Chunk

A raw MIDI chunk consists of a 4-character ASCII type identifier and a 32-bit unsigned integer specifying the length of its data:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chunk {
    pub chunk_type: [char; 4],
    pub length: u32,
}
```

### Parsed MIDI Chunk

Parsed chunks are categorized into meaningful types such as `HeaderChunk` and `TrackChunk`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedChunk {
    Header(HeaderChunk),
    Track(TrackChunk),
}
```

### Header Chunk

The `HeaderChunk` struct stores essential MIDI metadata:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeaderChunk {
    format: Format,
    ntrks: u16,
    division: Division,
}
```

## The Future

There is currently still a lot of work I want to do with `miami`, mainly MIDI file generation and a MIDI runtime potentially based in WASM. I'm planning to continue maintaining and working on this for the foreseeable future.

## Contributions

Contributions are welcome! If you find a bug or have a feature request, feel free to open an issue or submit a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
