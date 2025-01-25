//! Track chunk data enums and structs

use std::string::FromUtf8Error;

use event::{IteratorWrapper, MidiEvent, UnsupportedStatusCode};
use meta::MetaEvent;
use sysex::SysexEvent;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::writer::MidiWriteable;

pub mod event;
pub mod meta;
pub mod sysex;

/// Error types from parsing a track
#[derive(Debug, Clone, PartialEq)]
pub enum TrackError {
    /// End of File Marker, ends the iterator
    EOF,
    /// End of file while parsing Marker
    OutOfSpace,
    /// Invalid chunk format
    InvalidFormat,
    /// MIDI Channel Event status code is invalid
    UnsupportedStatusCode(UnsupportedStatusCode),
    /// Meta Event is in an invalid format
    InvalidMetaEventData,
    /// Invalid start tag for sysex message
    InvalidSysExMessage,
    /// Missing ending to exclusive message
    MissingEndOfExclusive,
    /// Error while parsing a UTF8 String for metadata
    UtfParseError(FromUtf8Error),
}

impl core::error::Error for TrackError {}
impl core::fmt::Display for TrackError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EOF => write![f, "Reached end of line at end of parsing"],
            Self::OutOfSpace => write![f, "Reached end of chunk before done parsing"],
            Self::InvalidFormat => write![f, "Invalid Track Format"],
            Self::UnsupportedStatusCode(e) => {
                write![f, "Invalid Status Code for MIDI Channel Event {e}"]
            }
            Self::InvalidMetaEventData => write![f, "Meta Event data is in an invalid format"],
            Self::InvalidSysExMessage => write![f, "Invalid SysEx Message Start"],
            Self::MissingEndOfExclusive => {
                write![f, "Missing end of System Exclusive Message 0xF7 byte"]
            }
            Self::UtfParseError(_) => write![
                f,
                "Failed to parse utf-8 encoded string in the meta track event"
            ],
        }
    }
}
impl From<UnsupportedStatusCode> for TrackError {
    fn from(f: UnsupportedStatusCode) -> Self {
        Self::UnsupportedStatusCode(f)
    }
}
impl From<FromUtf8Error> for TrackError {
    fn from(f: FromUtf8Error) -> Self {
        Self::UtfParseError(f)
    }
}

/// A track chunk, containing one or more MTrk events
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TrackChunk {
    /// All associated track events to this chunk
    pub(crate) mtrk_events: Vec<MTrkEvent>,
}

impl TryFrom<Vec<u8>> for TrackChunk {
    type Error = TrackError;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut value = value.into_iter();
        let mut mtrk_events = vec![];

        loop {
            match MTrkEvent::try_from(IteratorWrapper(&mut value)) {
                Ok(new_track) => mtrk_events.push(new_track),
                Err(TrackError::EOF) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(Self { mtrk_events })
    }
}

/// A MIDI Event with a DeltaTime and an attached Event
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MTrkEvent {
    /// Delta time is a variable-length representation of how much time to wait in ticks before the
    /// event follows.
    delta_time: u32,
    /// The event that occurs after the delta time is waited for
    event: Event,
}

impl MidiWriteable for MTrkEvent {
    fn to_midi_bytes(self) -> Vec<u8> {
        let mut bytes = MTrkEvent::to_midi_vlq(self.delta_time);
        let event_bytes = self.event.to_midi_bytes();

        bytes.extend(event_bytes.iter());

        bytes
    }
}

impl<ITER> TryFrom<IteratorWrapper<&mut ITER>> for MTrkEvent
where
    ITER: Iterator<Item = u8>,
{
    type Error = TrackError;
    fn try_from(value: IteratorWrapper<&mut ITER>) -> Result<Self, Self::Error> {
        let value = value.0;

        if let Some(dt) = MTrkEvent::try_get_delta_time(value) {
            Ok(MTrkEvent {
                delta_time: dt,
                event: Event::try_from(IteratorWrapper(value))?,
            })
        } else {
            Err(TrackError::EOF)
        }
    }
}

impl MTrkEvent {
    /// Gets the delta time as a variable length
    pub fn try_get_delta_time<ITER: Iterator<Item = u8>>(iter: &mut ITER) -> Option<u32> {
        let mut time_bytes = vec![];

        // Collect from iterator until delta time bytes are done
        for byte in iter.by_ref() {
            // Check if msb is 1, if not then this is the last delta time
            let msb_one = MTrkEvent::msb_is_one(byte);

            time_bytes.push(byte);

            if !msb_one {
                break;
            }
        }

        const MASK: u8 = 0x7F;

        if time_bytes.is_empty() {
            return None;
        }

        // Concat all bytes together
        let mut result: u32 = 0;
        for byte in time_bytes {
            result <<= 7;
            result |= (byte & MASK) as u32;
        }

        Some(result)
    }

    /// Goes backwards from length to variable length vector of bytes
    pub fn to_midi_vlq(mut value: u32) -> Vec<u8> {
        let mut bytes = Vec::new();

        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;

            if !bytes.is_empty() {
                byte |= 0x80;
            }

            bytes.push(byte);

            if value == 0 {
                break;
            }
        }

        bytes.reverse();
        bytes
    }

    /// Returns true if the msb of a byte is 1
    fn msb_is_one(byte: u8) -> bool {
        byte >> 7 == 1
    }
}

/// Any event that may occur
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Event {
    /// A midi event
    MidiEvent(MidiEvent),
    /// A system exclusive event
    SysexEvent(SysexEvent),
    /// Specifies non-MIDI information useful to this format or to sequencers
    MetaEvent(MetaEvent),
}

impl MidiWriteable for Event {
    fn to_midi_bytes(self) -> Vec<u8> {
        match self {
            Self::MidiEvent(event) => event.to_midi_bytes(),
            Self::SysexEvent(event) => event.to_midi_bytes(),
            Self::MetaEvent(event) => event.to_midi_bytes(),
        }
    }
}

impl<ITER> TryFrom<IteratorWrapper<&mut ITER>> for Event
where
    ITER: Iterator<Item = u8>,
{
    type Error = TrackError;
    fn try_from(value: IteratorWrapper<&mut ITER>) -> Result<Self, Self::Error> {
        let mut peek = value.0.peekable();

        let prefix = peek.peek().ok_or(TrackError::OutOfSpace)?;

        match prefix {
            status if (0x80..=0xEF).contains(status) => Ok(Event::MidiEvent(MidiEvent::try_from(
                IteratorWrapper(&mut peek),
            )?)),

            system if (0xF0..0xFF).contains(system) => Ok(Event::SysexEvent(SysexEvent::try_from(
                IteratorWrapper(&mut peek),
            )?)),

            0xFF => Ok(Event::MetaEvent(MetaEvent::try_from(IteratorWrapper(
                &mut peek,
            ))?)),

            _ => Err(TrackError::InvalidFormat),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MTrkEvent;

    #[test]
    fn delta_time_parsed() {
        let bytes = [0x81, 0x40];
        let mut bytes = bytes.into_iter();
        let result = MTrkEvent::try_get_delta_time(&mut bytes);

        assert_eq!(result, Some(192))
    }

    #[test]
    fn delta_time_backwards_parsed() {
        let time = 192;
        let bytes = MTrkEvent::to_midi_vlq(time);
        let expected = vec![0x81, 0x40];

        assert_eq!(bytes, expected)
    }
}
