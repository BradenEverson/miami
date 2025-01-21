//! Track chunk data enums and structs

pub mod status;

/// A track chunk, containing one or more MTrk events
#[derive(Debug, Clone, PartialEq)]
pub struct TrackChunk {
    mtrk_events: Vec<MTrkEvent>,
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

impl<ITER> From<&mut ITER> for MTrkEvent
where
    ITER: Iterator<Item = u8>,
{
    fn from(value: &mut ITER) -> Self {
        let dt = MTrkEvent::get_delta_time(value);

        MTrkEvent {
            delta_time: dt,
            event: Event::MidiEvent(MidiEvent),
        }
    }
}

impl MTrkEvent {
    /// Gets the delta time as a variable length
    pub fn get_delta_time<ITER: Iterator<Item = u8>>(iter: &mut ITER) -> u32 {
        let mut time_bytes = vec![];

        // Collect from iterator until delta time bytes are done
        while let Some(byte) = iter.next() {
            // Check if msb is 1, if not then this is the last delta time
            let msb_one = MTrkEvent::msb_is_one(byte);

            time_bytes.push(byte);

            if !msb_one {
                break;
            }
        }

        const MASK: u8 = 0x7F;
        // Concat all bytes together
        let mut result: u32 = 0;
        for byte in time_bytes {
            result <<= 7;
            result |= (byte & MASK) as u32;
        }

        result
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
