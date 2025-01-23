//! Track chunk data enums and structs

use event::{IteratorWrapper, MidiEvent, UnsupportedStatusCode};
use meta::MetaEvent;
use sysex::SysexEvent;
use thiserror::Error;

pub mod event;
pub mod meta;
pub mod sysex;

/// Error types from parsing a track
#[derive(Error, Debug, Clone, Copy, PartialEq)]
pub enum TrackError {
    /// End of File Marker, ends the iterator
    #[error("Reached end of line at end of parsing")]
    EOF,
    /// End of file while parsing Marker
    #[error("Reached end of chunk before done parsing")]
    OutOfSpace,
    /// Invalid chunk format
    #[error("Invalid Track Format")]
    InvalidFormat,
    /// MIDI Channel Event status code is invalid
    #[error("Invalid Status Code for MIDI Channel Event {0}")]
    UnsupportedStatusCode(#[from] UnsupportedStatusCode),
    /// Meta Event is in an invalid format
    #[error("Meta Event data is in an invalid format")]
    InvalidMetaEventData,
    /// Invalid start tag for sysex message
    #[error("Invalid SysEx Message Start")]
    InvalidSysExMessage,
    /// Missing ending to exclusive message
    #[error("Missing end of System Exclusive Message 0xF7 byte")]
    MissingEndOfExclusive,
}

/// A track chunk, containing one or more MTrk events
#[derive(Debug, Clone, PartialEq)]
pub struct TrackChunk {
    /// All associated track events to this chunk
    mtrk_events: Vec<MTrkEvent>,
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
pub struct MTrkEvent {
    /// Delta time is a variable-length representation of how much time to wait in ticks before the
    /// event follows.
    delta_time: u32,
    /// The event that occurs after the delta time is waited for
    event: Event,
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

    /// Returns true if the msb of a byte is 1
    fn msb_is_one(byte: u8) -> bool {
        byte >> 7 == 1
    }
}

/// Any event that may occur
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A midi event
    MidiEvent(MidiEvent),
    /// A system exclusive event
    SysexEvent(SysexEvent),
    /// Specifies non-MIDI information useful to this format or to sequencers
    MetaEvent(MetaEvent),
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
                IteratorWrapper(&mut peek.into_iter()),
            )?)),

            system if (0xF0..0xFF).contains(system) => Ok(Event::SysexEvent(SysexEvent::try_from(
                IteratorWrapper(&mut peek.into_iter()),
            )?)),

            0xFF => Ok(Event::MetaEvent(MetaEvent::try_from(IteratorWrapper(
                &mut peek.into_iter(),
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
        let bytes = vec![0x81, 0x40];
        let mut bytes = bytes.into_iter();
        let result = MTrkEvent::try_get_delta_time(&mut bytes);

        assert_eq!(result, Some(192))
    }
}
