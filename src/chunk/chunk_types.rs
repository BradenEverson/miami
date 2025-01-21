//! Chunk type constants

/// Creates a chunk type identifier
macro_rules! chunk_type {
    ($const_name:ident, $a:expr_2021, $b:expr_2021, $c:expr_2021, $d:expr_2021) => {
        /// MIDI chunk type
        pub const $const_name: [char; 4] = [$a, $b, $c, $d];
    };
}

chunk_type!(HEADER_CHUNK, 'M', 'T', 'h', 'd');
chunk_type!(TRACK_DATA_CHUNK, 'M', 'T', 'r', 'k');
