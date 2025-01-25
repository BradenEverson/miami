//! Meta Event Structs and Parsing

use super::{event::IteratorWrapper, TrackError};
use crate::{chunk::track::MTrkEvent, reader::Yieldable, writer::MidiWriteable};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A meta level event
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

impl MetaEvent {
    /// Returns the specific event's tag
    pub fn get_tag(&self) -> u8 {
        match self {
            Self::SequenceNumber(_) => 0x00,
            Self::Text(_) => 0x01,
            Self::Copyright(_) => 0x02,
            Self::TrackName(_) => 0x03,
            Self::InstrumentName(_) => 0x04,
            Self::Lyric(_) => 0x05,
            Self::Marker(_) => 0x06,
            Self::CuePoint(_) => 0x07,
            Self::MidiChannelPrefix(_) => 0x20,
            Self::EndOfTrack => 0x2F,
            Self::Tempo(_) => 0x51,
            Self::SmpteOffset(_) => 0x54,
            Self::TimeSignature(_) => 0x58,
            Self::KeySignature(_) => 0x59,
            Self::SequencerSpecific(_) => 0x7F,
            Self::UnknownRaw(tag, _) => *tag,
        }
    }
}

impl MidiWriteable for MetaEvent {
    fn to_midi_bytes(self) -> Vec<u8> {
        let tag_byte = self.get_tag();
        let mut bytes = vec![0xFF, tag_byte];

        let payload_bytes = match self {
            Self::SequenceNumber(val) => val.to_midi_bytes(),
            Self::Text(val) => val.to_midi_bytes(),
            Self::Copyright(val) => val.to_midi_bytes(),
            Self::TrackName(val) => val.to_midi_bytes(),
            Self::InstrumentName(val) => val.to_midi_bytes(),
            Self::Lyric(val) => val.to_midi_bytes(),
            Self::Marker(val) => val.to_midi_bytes(),
            Self::CuePoint(val) => val,
            Self::MidiChannelPrefix(val) => val.to_midi_bytes(),
            Self::EndOfTrack => vec![],
            Self::Tempo(val) => val.to_midi_bytes(),
            Self::SmpteOffset(val) => val.to_midi_bytes(),
            Self::TimeSignature(val) => val.to_midi_bytes(),
            Self::KeySignature(val) => val.to_midi_bytes(),
            Self::SequencerSpecific(val) => val,
            Self::UnknownRaw(_, val) => val,
        };

        let length = payload_bytes.len() as u32;
        let len_vlq = MTrkEvent::to_midi_vlq(length);

        bytes.extend(len_vlq.iter());
        bytes.extend(payload_bytes.iter());

        bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// A key signature
pub struct KeySignature {
    /// Sharps and flats
    sharps_flats: i8,
    /// True if in major false if in minor
    major_minor: bool,
}

impl MidiWriteable for KeySignature {
    fn to_midi_bytes(self) -> Vec<u8> {
        let KeySignature {
            sharps_flats,
            major_minor,
        } = self;

        let mut bytes = sharps_flats.to_midi_bytes();
        let major_minor_bit = if major_minor {
            // Some data may be lost here as we only know *if* major was not 0 it's true. But it's
            // only ever used for this so it's not too much of an issue
            1
        } else {
            0
        };

        bytes.push(major_minor_bit);

        bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

impl MidiWriteable for SmpteOffset {
    fn to_midi_bytes(self) -> Vec<u8> {
        let SmpteOffset {
            hours,
            minutes,
            seconds,
            frames,
            subframes,
        } = self;
        vec![hours, minutes, seconds, frames, subframes]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

impl MidiWriteable for TimeSignature {
    fn to_midi_bytes(self) -> Vec<u8> {
        let TimeSignature {
            numerator,
            denominator,
            clocks_per_tick,
            thirty_second_notes_per_quarter,
        } = self;
        let mut bytes = vec![numerator];
        bytes.extend(denominator.to_midi_bytes().iter());
        bytes.extend([clocks_per_tick, thirty_second_notes_per_quarter]);

        bytes
    }
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
    use crate::{
        chunk::track::{
            event::IteratorWrapper,
            meta::{KeySignature, MetaEvent, SmpteOffset, TimeSignature},
            TrackError,
        },
        writer::MidiWriteable,
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

    #[test]
    fn meta_event_backwards_parses_to_bytes() {
        let expected = MetaEvent::KeySignature(KeySignature {
            sharps_flats: 0,
            major_minor: false,
        });

        let bytes = expected.clone().to_midi_bytes();

        let result = MetaEvent::try_from(IteratorWrapper(&mut bytes.into_iter())).unwrap();
        assert_eq!(result, expected);
    }

    macro_rules! meta_event_test {
        ($name:ident, $event:expr_2021, $data:expr_2021) => {
            #[test]
            fn $name() {
                let data = $data;
                let expected = $event;
                let parsed =
                    MetaEvent::try_from(IteratorWrapper(&mut data.clone().into_iter())).unwrap();
                assert_eq!(parsed, expected);

                let serialized = expected.clone().to_midi_bytes();
                assert_eq!(serialized, data);
            }
        };
    }

    meta_event_test!(
        sequence_number_event,
        MetaEvent::SequenceNumber(1),
        vec![0xFF, 0x00, 0x02, 0x00, 0x01]
    );

    meta_event_test!(
        text_event,
        MetaEvent::Text("Hello".to_string()),
        vec![0xFF, 0x01, 0x05, b'H', b'e', b'l', b'l', b'o']
    );

    meta_event_test!(
        copyright_event,
        MetaEvent::Copyright("Copyright".to_string()),
        vec![0xFF, 0x02, 0x09, b'C', b'o', b'p', b'y', b'r', b'i', b'g', b'h', b't']
    );

    meta_event_test!(
        track_name_event,
        MetaEvent::TrackName("Track 1".to_string()),
        vec![0xFF, 0x03, 0x07, b'T', b'r', b'a', b'c', b'k', b' ', b'1']
    );

    meta_event_test!(
        instrument_name_event,
        MetaEvent::InstrumentName("Piano".to_string()),
        vec![0xFF, 0x04, 0x05, b'P', b'i', b'a', b'n', b'o']
    );

    meta_event_test!(
        lyric_event,
        MetaEvent::Lyric("Lyrics".to_string()),
        vec![0xFF, 0x05, 0x06, b'L', b'y', b'r', b'i', b'c', b's']
    );

    meta_event_test!(
        marker_event,
        MetaEvent::Marker("Marker".to_string()),
        vec![0xFF, 0x06, 0x06, b'M', b'a', b'r', b'k', b'e', b'r']
    );

    meta_event_test!(
        cue_point_event,
        MetaEvent::CuePoint(vec![0x01, 0x02]),
        vec![0xFF, 0x07, 0x02, 0x01, 0x02]
    );

    meta_event_test!(
        midi_channel_prefix_event,
        MetaEvent::MidiChannelPrefix(0x05),
        vec![0xFF, 0x20, 0x01, 0x05]
    );

    meta_event_test!(
        end_of_track_event,
        MetaEvent::EndOfTrack,
        vec![0xFF, 0x2F, 0x00]
    );

    meta_event_test!(
        smpte_offset_event,
        MetaEvent::SmpteOffset(SmpteOffset {
            hours: 1,
            minutes: 32,
            seconds: 21,
            frames: 16,
            subframes: 0,
        }),
        vec![0xFF, 0x54, 0x05, 0x01, 0x20, 0x15, 0x10, 0x00]
    );

    meta_event_test!(
        key_signature_event,
        MetaEvent::KeySignature(KeySignature {
            sharps_flats: 0,
            major_minor: false,
        }),
        vec![0xFF, 0x59, 0x02, 0x00, 0x00]
    );

    meta_event_test!(
        sequencer_specific_event,
        MetaEvent::SequencerSpecific(vec![0x01, 0x02, 0x03]),
        vec![0xFF, 0x7F, 0x03, 0x01, 0x02, 0x03]
    );

    meta_event_test!(
        unknown_raw_event,
        MetaEvent::UnknownRaw(0x99, vec![0x01, 0x02, 0x03]),
        vec![0xFF, 0x99, 0x03, 0x01, 0x02, 0x03]
    );
}
