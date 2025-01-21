//! Status parsing trait and implementation

use crate::reader::Yieldable;

/// A MIDI Message Status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MidiStatus {
    /// Turn Off event
    /// This message is sent whena  note is released
    NoteOff(u8, NoteMeta),
    /// Turn On event
    /// This message is sent when a note is depressed
    NoteOn(u8, NoteMeta),
    /// Polyphonic Key Pressure
    /// This message is most often sent by pressing down a key after it "bottoms out"
    PolyphonicKeyPressure(u8, NoteMeta),
    /// Control change
    /// This message is sent when a controller value changes. Controllers include devices such as
    /// pedals and levers. Certain controller numbers are reserved.
    ControlChange(u8, ControlChange),
    /// Program change.
    /// This message is sent when the patch number changes
    ProgramChange(u8, u8),
    /// Channel Pressure
    /// This message is most often sent by pressing down on a key after it "bottoms out"
    ChannelPressure(u8, u8),
    /// Pitch Wheel Change
    /// This message is sent to indicate a change in the pitch wheel as measured by a fourteen bit
    /// value.
    PitchWheelChange(u8, u16),
}

/// Error type for an unsupported error type
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnsupportedStatusCode;

/// Wrapper around iterator to prevent trait implementation sillyness
pub struct IteratorWrapper<T>(T);
impl<ITER> TryFrom<IteratorWrapper<&mut ITER>> for MidiStatus
where
    ITER: Iterator<Item = u8>,
{
    type Error = UnsupportedStatusCode;
    fn try_from(value: IteratorWrapper<&mut ITER>) -> Result<Self, Self::Error> {
        let value = value.0;
        let status = value.get(1)[0];
        let channel = status & 0x0F;
        let status = status >> 4 as u8;

        match status {
            0b1000 => {
                let reads = value.get(2);
                Ok(Self::NoteOff(
                    channel,
                    NoteMeta {
                        key: reads[0],
                        velocity: reads[1],
                    },
                ))
            }

            0b1001 => {
                let reads = value.get(2);
                Ok(Self::NoteOn(
                    channel,
                    NoteMeta {
                        key: reads[0],
                        velocity: reads[1],
                    },
                ))
            }

            0b1011 => {
                let reads = value.get(2);
                Ok(Self::ControlChange(
                    channel,
                    ControlChange {
                        controller_number: reads[0],
                        new_value: reads[1],
                    },
                ))
            }

            0b1100 => {
                let reads = value.get(1);
                Ok(Self::ProgramChange(channel, reads[0]))
            }

            0b1101 => {
                let reads = value.get(1);
                Ok(Self::ChannelPressure(channel, reads[0]))
            }

            0b1110 => {
                let reads = value.get(2);

                const MASK: u8 = 0x7;

                let mut result: u16 = 0;
                for byte in reads.iter().rev() {
                    result <<= 7;
                    result |= (byte & MASK) as u16;
                }

                Ok(Self::PitchWheelChange(channel, result))
            }

            _ => Err(UnsupportedStatusCode),
        }
    }
}

/// Metadata for a note's relative info. Including channel, key and velocity
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoteMeta {
    /// Note key
    key: u8,
    /// Note velocity
    velocity: u8,
}

/// Metadata for changing a controller
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlChange {
    /// Controller number
    controller_number: u8,
    /// New value
    new_value: u8,
}

#[cfg(test)]
mod tests {
    use super::{IteratorWrapper, MidiStatus, NoteMeta};

    #[test]
    fn midi_event_status_parsing() {
        let status_channel = 0b10001111;
        let key = 0b01010101;
        let velocity = 0b11111111;

        let mut stream = [status_channel, key, velocity].into_iter();
        let status =
            MidiStatus::try_from(IteratorWrapper(&mut stream)).expect("Parse off note signal");

        let expected = MidiStatus::NoteOff(0x0F, NoteMeta { key, velocity });

        assert_eq!(status, expected)
    }
}
