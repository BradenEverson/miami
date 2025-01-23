//! Meta Event Structs and Parsing

use super::{event::IteratorWrapper, TrackError};
use crate::{chunk::parsed::track::MTrkEvent, reader::Yieldable};

/// A meta level event
#[derive(Debug, Clone, PartialEq)]
pub enum MetaEvent {
    /// Sequence Number, tag 0x00
    SequenceNumber(u16),
    /// Text metadata, tag 0x01
    Text(String),
    /// Copyright, tag 0x02
    Copyright(String),
    /// Track name, tag 0x03
    TrackName(String),
    /// Instrucment name, tag 0x04
    InstrumentName(String),
    /// Lyric, tag 0x05
    Lyric(String),
    /// Marker, tag 0x06
    Marker(String),
    /// Cue Point, tag 0x07
    CuePoint(Vec<u8>),
    /// Midi Channel Prefix, tag 0x20
    MidiChannelPrefix(u8),
    /// End of Track Identifier, tag 0x2F
    EndOfTrack,
    /// Tempo, tag 0x51
    Tempo(u32),
    /// Smpte Offset, tag 0x54
    SmpteOffset(SmpteOffset),
    /// Time signature, tag 0x58
    TimeSignature(TimeSignature),
    /// Key Signature, tag 0x59
    KeySignature(KeySignature),
    /// Sequencer Specific, tag 0x7f
    SequencerSpecific(Vec<u8>),
    /// An unknown meta event
    UnknownRaw(u8, Vec<u8>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A key signature
pub struct KeySignature {
    sharps_flats: i8,
    major_minor: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// An SMPTE Offset
pub struct SmpteOffset {
    /// Hours of offset
    hours: u8,
    /// Minutes of offset
    minutes: u8,
    /// Seconds of offset
    seconds: u8,
    /// Frames of offset
    frames: u8,
    /// Subframes of offset
    subframes: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A Time Signature
pub struct TimeSignature {
    /// The time signature's numerator
    numerator: u8,
    /// The time signature's denominator
    denominator: u32,
    /// Clocks per tick
    clocks_per_tick: u8,
    /// Thirty second notes per quarter
    thirty_second_notes_per_quarter: u8,
}

impl<ITER> TryFrom<IteratorWrapper<&mut ITER>> for MetaEvent
where
    ITER: Iterator<Item = u8>,
{
    type Error = TrackError;
    fn try_from(value: IteratorWrapper<&mut ITER>) -> Result<Self, Self::Error> {
        let prefix = value.0.next().ok_or(TrackError::OutOfSpace)?;
        if prefix != 0xFF {
            return Err(TrackError::InvalidMetaEventData);
        }

        let event_tag = value.0.next().ok_or(TrackError::OutOfSpace)?;

        let length = MTrkEvent::try_get_delta_time(value.0).ok_or(TrackError::OutOfSpace)?;

        let data = value.0.get(length as usize);

        macro_rules! meta_event {
            ($len: expr_2021, $name: expr_2021, $value: expr_2021) => {{
                if data.len() != $len {
                    return Err(TrackError::InvalidMetaEventData);
                }
                Ok($name($value))
            }};
        }

        match event_tag {
            0x00 => meta_event!(
                2,
                MetaEvent::SequenceNumber,
                u16::from_be_bytes([data[0], data[1]])
            ),
            0x01 => Ok(MetaEvent::Text(String::from_utf8(data)?)),
            0x02 => Ok(MetaEvent::Copyright(String::from_utf8(data)?)),
            0x03 => Ok(MetaEvent::TrackName(String::from_utf8(data)?)),
            0x04 => Ok(MetaEvent::InstrumentName(String::from_utf8(data)?)),
            0x05 => Ok(MetaEvent::Lyric(String::from_utf8(data)?)),
            0x06 => Ok(MetaEvent::Marker(String::from_utf8(data)?)),
            0x07 => Ok(MetaEvent::CuePoint(data)),

            0x20 => meta_event!(1, MetaEvent::MidiChannelPrefix, data[0]),
            0x2F => Ok(MetaEvent::EndOfTrack),

            0x51 => meta_event!(
                3,
                MetaEvent::Tempo,
                ((data[0] as u32) << 16) | ((data[1] as u32) << 8) | (data[2] as u32)
            ),
            0x54 => meta_event!(
                5,
                MetaEvent::SmpteOffset,
                SmpteOffset {
                    hours: data[0],
                    minutes: data[1],
                    seconds: data[2],
                    frames: data[3],
                    subframes: data[4]
                }
            ),
            0x58 => meta_event!(
                4,
                MetaEvent::TimeSignature,
                TimeSignature {
                    numerator: data[0],
                    denominator: 2u32.pow(data[1] as u32),
                    clocks_per_tick: data[2],
                    thirty_second_notes_per_quarter: data[3],
                }
            ),
            0x59 => meta_event!(
                2,
                MetaEvent::KeySignature,
                KeySignature {
                    sharps_flats: data[0] as i8,
                    major_minor: data[1] != 0
                }
            ),

            0x7F => Ok(MetaEvent::SequencerSpecific(data)),

            _ => Ok(MetaEvent::UnknownRaw(event_tag, data)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::chunk::parsed::track::{
        event::IteratorWrapper,
        meta::{KeySignature, MetaEvent, SmpteOffset, TimeSignature},
        TrackError,
    };

    #[test]
    fn test_sequence_number() {
        let data = vec![0xFF, 0x00, 0x02, 0x00, 0x01]; // Tag: 0x00, Length: 2, Value: [0x00, 0x01]
        let result = MetaEvent::try_from(IteratorWrapper(&mut data.into_iter())).unwrap();
        assert_eq!(result, MetaEvent::SequenceNumber(1));
    }

    #[test]
    fn test_text_event() {
        let data = vec![0xFF, 0x01, 0x05, b'H', b'e', b'l', b'l', b'o']; // Tag: 0x01, Length: 5, Value: "Hello"
        let result = MetaEvent::try_from(IteratorWrapper(&mut data.into_iter())).unwrap();
        assert_eq!(result, MetaEvent::Text("Hello".to_string()));
    }

    #[test]
    fn test_copyright_event() {
        let data = vec![
            0xFF, 0x02, 0x0A, b'C', b'o', b'p', b'y', b'r', b'i', b'g', b'h', b't',
        ];
        let result = MetaEvent::try_from(IteratorWrapper(&mut data.into_iter())).unwrap();
        assert_eq!(result, MetaEvent::Copyright("Copyright".to_string()));
    }

    #[test]
    fn test_tempo_event() {
        let data = vec![0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]; // Tag: 0x51, Length: 3, Tempo: 500,000 microseconds/quarter note
        let result = MetaEvent::try_from(IteratorWrapper(&mut data.into_iter())).unwrap();
        assert_eq!(result, MetaEvent::Tempo(500_000));
    }

    #[test]
    fn test_time_signature_event() {
        let data = vec![0xFF, 0x58, 0x04, 0x04, 0x02, 0x18, 0x08]; // Tag: 0x58, Length: 4
        let result = MetaEvent::try_from(IteratorWrapper(&mut data.into_iter())).unwrap();
        assert_eq!(
            result,
            MetaEvent::TimeSignature(TimeSignature {
                numerator: 4,
                denominator: 4, // 2^2 = 4
                clocks_per_tick: 24,
                thirty_second_notes_per_quarter: 8,
            })
        );
    }

    #[test]
    fn test_key_signature_event() {
        let data = vec![0xFF, 0x59, 0x02, 0x00, 0x00]; // Tag: 0x59, Length: 2, C Major
        let result = MetaEvent::try_from(IteratorWrapper(&mut data.into_iter())).unwrap();
        assert_eq!(
            result,
            MetaEvent::KeySignature(KeySignature {
                sharps_flats: 0,
                major_minor: false,
            })
        );
    }

    #[test]
    fn test_smpte_offset_event() {
        let data = vec![0xFF, 0x54, 0x05, 0x01, 0x20, 0x15, 0x10, 0x00]; // Tag: 0x54, Length: 5
        let result = MetaEvent::try_from(IteratorWrapper(&mut data.into_iter())).unwrap();
        assert_eq!(
            result,
            MetaEvent::SmpteOffset(SmpteOffset {
                hours: 1,
                minutes: 32,
                seconds: 21,
                frames: 16,
                subframes: 0,
            })
        );
    }

    #[test]
    fn test_end_of_track_event() {
        let data = vec![0xFF, 0x2F, 0x00]; // Tag: 0x2F, Length: 0
        let result = MetaEvent::try_from(IteratorWrapper(&mut data.into_iter())).unwrap();
        assert_eq!(result, MetaEvent::EndOfTrack);
    }

    #[test]
    fn test_unknown_event() {
        let data = vec![0xFF, 0x99, 0x03, 0x01, 0x02, 0x03]; // Unknown Tag: 0x99
        let result = MetaEvent::try_from(IteratorWrapper(&mut data.into_iter())).unwrap();
        assert_eq!(result, MetaEvent::UnknownRaw(0x99, vec![0x01, 0x02, 0x03]));
    }

    #[test]
    fn test_invalid_length() {
        let data = vec![0xFF, 0x00, 0x02, 0x02]; // Tag: 0x00, Length: 3, but only 2 bytes provided
        let result = MetaEvent::try_from(IteratorWrapper(&mut data.into_iter()));
        assert_eq!(result, Err(TrackError::InvalidMetaEventData));
    }

    #[test]
    fn test_out_of_space() {
        let data = vec![]; // Empty data
        let result = MetaEvent::try_from(IteratorWrapper(&mut data.into_iter()));
        assert_eq!(result, Err(TrackError::OutOfSpace));
    }
}
