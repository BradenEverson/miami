//! System Exclusive Messages

use super::{event::IteratorWrapper, TrackError};

/// A midi system exclusize event message
#[derive(Debug, Clone, PartialEq)]
pub struct SysexEvent {
    /// The manufacture ID of the System Exclusize message
    manufacture_id: ManufactureId,
    /// Data payload to be parsed on a per-system basis
    payload: Vec<u8>,
}

/// A manufacturer's ID. Can be either a 1 byte variant or 3 bytes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ManufactureId {
    /// One byte ID
    OneByte(u8),
    /// Three byte ID
    ThreeByte([u8; 3]),
}

impl<ITER> TryFrom<&mut IteratorWrapper<&mut ITER>> for ManufactureId
where
    ITER: Iterator<Item = u8>,
{
    type Error = TrackError;
    fn try_from(value: &mut IteratorWrapper<&mut ITER>) -> Result<Self, Self::Error> {
        let first_byte = value.0.next().ok_or(TrackError::OutOfSpace)?;
        if first_byte == 0x00 {
            let second_byte = value.0.next().ok_or(TrackError::OutOfSpace)?;
            let third_byte = value.0.next().ok_or(TrackError::OutOfSpace)?;

            Ok(ManufactureId::ThreeByte([
                first_byte,
                second_byte,
                third_byte,
            ]))
        } else {
            Ok(ManufactureId::OneByte(first_byte))
        }
    }
}

impl<ITER> TryFrom<IteratorWrapper<&mut ITER>> for SysexEvent
where
    ITER: Iterator<Item = u8>,
{
    type Error = TrackError;
    fn try_from(mut value: IteratorWrapper<&mut ITER>) -> Result<Self, Self::Error> {
        let prefix = value.0.next().ok_or(TrackError::OutOfSpace)?;
        if prefix != 0xF0 {
            return Err(TrackError::InvalidSysExMessage);
        }

        let manufacture_id = ManufactureId::try_from(&mut value)?;
        let mut payload = vec![];

        loop {
            let byte = value.0.next().ok_or(TrackError::MissingEndOfExclusive)?;
            if byte == 0xF7 {
                break;
            } else {
                payload.push(byte);
            }
        }

        Ok(Self {
            manufacture_id,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::chunk::track::{event::IteratorWrapper, TrackError};

    use super::{ManufactureId, SysexEvent};

    #[test]
    fn one_byte_manufature_id() {
        let mut data = [0x01, 0x02, 0xFF, 0xFF].into_iter();
        let mut wrapper = IteratorWrapper(&mut data);

        let id = ManufactureId::try_from(&mut wrapper).expect("Parse ID from bytes");
        assert_eq!(id, ManufactureId::OneByte(0x01))
    }

    #[test]
    fn three_byte_manufature_id() {
        let mut data = [0x00, 0x33, 0xFF, 0xFF].into_iter();
        let mut wrapper = IteratorWrapper(&mut data);

        let id = ManufactureId::try_from(&mut wrapper).expect("Parse ID from bytes");
        assert_eq!(id, ManufactureId::ThreeByte([0x00, 0x33, 0xFF]))
    }

    #[test]
    fn byte_parsing_ends_early_if_iterator_runs_out() {
        let mut data = [0x00, 0x33].into_iter();
        let mut wrapper = IteratorWrapper(&mut data);

        let id = ManufactureId::try_from(&mut wrapper);
        assert_eq!(id, Err(TrackError::OutOfSpace))
    }

    #[test]
    fn sys_ex_message_valid_parse() {
        let mut data = [0xF0, 0x01, 0xFF, 0x00, 0x21, 0xF7].into_iter();
        let wrapper = IteratorWrapper(&mut data);

        let sysex = SysexEvent::try_from(wrapper).expect("Parse sysex message from bytes");
        let expected = SysexEvent {
            manufacture_id: ManufactureId::OneByte(0x01),
            payload: vec![0xFF, 0x00, 0x21],
        };

        assert_eq!(sysex, expected)
    }

    #[test]
    fn sys_ex_message_invalid_parse_failes() {
        let mut data = [0xF0, 0x01, 0xFF, 0x00, 0x21].into_iter();
        let wrapper = IteratorWrapper(&mut data);

        let sysex = SysexEvent::try_from(wrapper);

        assert_eq!(sysex, Err(TrackError::MissingEndOfExclusive))
    }
}
