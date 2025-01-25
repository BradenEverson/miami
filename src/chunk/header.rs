//! Header Chunk Enum and Struct Definitions

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::writer::MidiWriteable;

/// Header chunk data, including format, ntrks and division as 3 16 bit unsigned integers
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderChunk {
    /// The MIDI format
    format: Format,
    /// Number of tracks
    ntrks: u16,
    /// Time signature/division
    division: Division,
}

impl MidiWriteable for HeaderChunk {
    fn to_midi_bytes(self) -> Vec<u8> {
        let mut bytes = self.format.to_midi_bytes();
        let ntrks = self.ntrks.to_midi_bytes();
        let division = self.division.to_midi_bytes();

        bytes.extend(ntrks.iter());
        bytes.extend(division.iter());

        bytes
    }
}

impl TryFrom<(u16, u16, u16)> for HeaderChunk {
    type Error = InvalidFormat;
    fn try_from(value: (u16, u16, u16)) -> Result<Self, Self::Error> {
        let (format, ntrks, division) = value;

        Ok(Self {
            format: format.try_into()?,
            ntrks,
            division: division.into(),
        })
    }
}

/// The overall organization of the MIDI file. Only three values are valid, making most of the 16
/// bits irrelevant
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Format {
    /// The file contains a single multi-channel track
    Zero,
    /// The file contains one or more simultaneous tracks (or MIDI outputs) of a sequence
    One,
    /// The file contains one or more sequentially independent single-track patterns
    Two,
}

impl MidiWriteable for Format {
    fn to_midi_bytes(self) -> Vec<u8> {
        vec![
            0,
            match self {
                Format::Zero => 0,
                Format::One => 1,
                Format::Two => 2,
            },
        ]
    }
}

/// Error struct representing an invalid format specifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidFormat;

impl core::error::Error for InvalidFormat {}
impl core::fmt::Display for InvalidFormat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write![f, "Invalid header format"]
    }
}

impl TryFrom<u16> for Format {
    type Error = InvalidFormat;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Format::Zero),
            1 => Ok(Format::One),
            2 => Ok(Format::Two),
            _ => Err(InvalidFormat),
        }
    }
}

/// The meaning of the delta-times in the MIDI sequence,
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Division {
    /// When bit 15 is a 0, bits 14-0 represent ticks per quarter note
    Metrical(u16),
    /// When bit 15 is 1, bits 14-8 represent the negative SMPTE format,
    /// and bits 7-0 represent ticks per frame
    TimeCodeBased(SmpteTicks),
}

/// Division defined by time-code-based time
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SmpteTicks {
    /// 7 bits of negative timecode
    smpte: i8,
    /// 8 bits of ticks per frame
    tpf: u8,
}

impl MidiWriteable for SmpteTicks {
    fn to_midi_bytes(self) -> Vec<u8> {
        const MASK: u8 = 0x80;
        let smpte = self.smpte.to_be_bytes()[0] | MASK;

        vec![smpte, self.tpf]
    }
}

impl From<u16> for Division {
    fn from(value: u16) -> Self {
        const MASK: u16 = 0x7FFF;
        let msb = value >> 15;
        let remaining = value & MASK;

        match msb {
            0 => Division::Metrical(remaining),
            1 => {
                // Time Code Based
                let tpf = remaining as u8;
                let smpte = (remaining >> 8) as i8;

                // Explicit sign extension for SMPTE
                let smpte = if smpte & 0x8 != 0 {
                    smpte | !0x7F
                } else {
                    smpte
                };

                let ticks = SmpteTicks { smpte, tpf };

                Division::TimeCodeBased(ticks)
            }
            _ => unreachable!("Only msb is checked and can therefore only be 1 or 0"),
        }
    }
}

impl MidiWriteable for Division {
    fn to_midi_bytes(self) -> Vec<u8> {
        match self {
            Self::Metrical(metrical) => metrical.to_midi_bytes(),
            Self::TimeCodeBased(smpte) => smpte.to_midi_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    const HEADER_CHUNK_RAW: Chunk = Chunk {
        chunk_type: HEADER_CHUNK,
        length: 6,
    };

    use crate::{
        chunk::{
            chunk_types::HEADER_CHUNK,
            header::{Division, Format, HeaderChunk, SmpteTicks},
        },
        reader::{MidiReadable, MidiStream},
        Chunk,
    };

    #[test]
    fn parsing_division_to_metrical_works() {
        let test: Division = (0x000au16).into();
        let expected = Division::Metrical(10);

        assert_eq!(test, expected)
    }

    #[test]
    fn parsing_division_to_timecode_works() {
        let test: Division = (0x80FFu16).into();
        let expected = Division::TimeCodeBased(SmpteTicks { smpte: 0, tpf: 255 });

        assert_eq!(test, expected);

        let test: Division = (0xFFE8u16).into();
        let expected = Division::TimeCodeBased(SmpteTicks {
            smpte: -1,
            tpf: 232,
        });

        assert_eq!(test, expected);

        let test: Division = (0x8bFFu16).into();
        let expected = Division::TimeCodeBased(SmpteTicks {
            smpte: -117,
            tpf: 255,
        });

        assert_eq!(test, expected)
    }

    #[test]
    fn header_chunk_reads_properly() {
        let mut data = "test/run.mid"
            .get_midi_bytes()
            .expect("Get `run.midi` file and stream bytes");

        let (header, payload) = data.read_chunk_data_pair().expect("Get chunk and data");

        let header: Chunk = header.into();
        assert_eq!(header, HEADER_CHUNK_RAW);

        // Now we try reading the next 6 bytes as [u16; 3]
        let mut payload = payload.iter();
        let mut packets = vec![];
        while let Some(first) = payload.next() {
            if let Some(second) = payload.next() {
                let bytes = [*first, *second];
                let packet = u16::from_be_bytes(bytes);
                packets.push(packet);
            }
        }

        assert!(packets.len() == 3);

        let header_chunk = HeaderChunk::try_from((packets[0], packets[1], packets[2]))
            .expect("Parse header chunk from payload packets");
        let expected = HeaderChunk {
            format: Format::One,
            ntrks: 10,
            division: Division::Metrical(384),
        };

        assert_eq!(expected, header_chunk)
    }
}
