//! Track chunk data enums and structs

use status::IteratorWrapper;
use thiserror::Error;

pub mod status;

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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MTrkEvent {
    /// Delta time is a variable-length representation of how much time to wait in ticks before the
    /// event follows.
    delta_time: u32,
    /// The event that occurs after the delta time is waited for
    event: Event,
}

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
}

impl<ITER> TryFrom<IteratorWrapper<&mut ITER>> for MTrkEvent
where
    ITER: Iterator<Item = u8>,
{
    type Error = TrackError;
    fn try_from(value: IteratorWrapper<&mut ITER>) -> Result<Self, Self::Error> {
        let value = value.0;

        if let Some(dt) = MTrkEvent::get_delta_time(value) {
            Ok(MTrkEvent {
                delta_time: dt,
                event: Event::MidiEvent(MidiEvent),
            })
        } else {
            Err(TrackError::EOF)
        }
    }
}

impl MTrkEvent {
    /// Gets the delta time as a variable length
    pub fn get_delta_time<ITER: Iterator<Item = u8>>(iter: &mut ITER) -> Option<u32> {
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

        if time_bytes.len() == 0 {
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// A midi event
    MidiEvent(MidiEvent),
    /// A system exclusive event
    SysexEvent(SysexEvent),
    /// Specifies non-MIDI information useful to this format or to sequencers
    MetaEvent(MetaEvent),
}

/// A MIDI channel message
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MidiEvent;

/// A midi system exclusize event message
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SysexEvent;

/// A meta level event
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetaEvent;

#[cfg(test)]
mod tests {
    use super::MTrkEvent;

    #[test]
    fn delta_time_parsed() {
        let bytes = vec![0x81, 0x40];
        let mut bytes = bytes.into_iter();
        let result = MTrkEvent::get_delta_time(&mut bytes);

        assert_eq!(result, 192)
    }
}
