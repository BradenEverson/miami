//! Parsed Chunk Enum type

/// A chunk's type paired with it's data
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParsedChunk {
    /// A header chunk
    Header(HeaderChunk),
}

/// Header chunk data, including format, ntrks and division as 3 16 bit unsigned integers
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeaderChunk {
    format: Format,
    ntrks: u16,
    division: u16,
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
pub enum Format {
    /// The file contains a single multi-channel track
    Zero,
    /// The file contains one or more simultaneous tracks (or MIDI outputs) of a sequence
    One,
    /// The file contains one or more sequentially independent single-track patterns
    Two,
}

/// Error struct representing an invalid format specifier
pub struct InvalidFormat;

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
pub enum Division {
    /// When bit 15 is a 0, bits 14-0 represent ticks per quarter note
    Metrical(u16),
    /// When bit 15 is 1, bits 14-8 represent the negative SMPTE format,
    /// and bits 7-0 represent ticks per frame
    TimeCodeBased(SmpteTicks),
}

/// Division defined by time-code-based time
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SmpteTicks {
    /// 7 bits of negative timecode
    smpte: i8,
    /// 8 bits of ticks per frame
    tpf: u8,
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

#[cfg(test)]
mod tests {
    use crate::chunk::parsed::SmpteTicks;

    use super::Division;

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
}
