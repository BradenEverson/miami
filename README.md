# Miami 🌴📼

*parsing midi files is not as scary as it sounds.* 🐔

[![crates.io](https://img.shields.io/crates/v/miami.svg)](https://crates.io/crates/miami)
[![Tests](https://github.com/BradenEverson/miami/actions/workflows/rust.yml/badge.svg)](https://github.com/BradenEverson/miami/actions/workflows/rust.yml)

A lightweight, minimal dependency MIDI file parser and binary writer designed for resource-constrained environments, making it great for WebAssembly applications.

## Features

- **Efficient MIDI Parsing**: Parse MIDI chunks and their associated data with minimal overhead.
- **Error Handling**: Comprehensive error reporting for invalid or unsupported chunks.
- **MIDI Format Writing**: Serialize Parsed or Generating MIDI chunks back into MIDI format binary.

## Getting Started

### Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
miami = "{whatever version you want}"
```

For serde support include the `serde` feature flag ;)

### Example Usage

The following example demonstrates how to read and process MIDI chunks from a file:

```rust
let mut data = "path/to/midi/file.mid"
    .get_midi_bytes()
    .expect("Failed to load MIDI file");

while let Some(parsed) = data
    .read_chunk_data_pair()
    .map(|val| ParsedChunk::try_from(val))
{
    println!("{parsed:?}");
}
```

Writing a set of ParsedChunks to a file:

```rust

let mut output = File::create("output.mid").unwrap();
for parsed in parsed_chunks {
    let bytes = parsed.to_midi_bytes();
    output.write(&bytes).expect("Write bytes to file");
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
## Contributions

Contributions are welcome! If you find a bug or have a feature request, feel free to open an issue or submit a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
