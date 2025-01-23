//! Meta Event Structs and Parsing

use super::{event::IteratorWrapper, TrackError};

/// A meta level event
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetaEvent;

impl<ITER> TryFrom<IteratorWrapper<&mut ITER>> for MetaEvent
where
    ITER: Iterator<Item = u8>,
{
    type Error = TrackError;
    fn try_from(value: IteratorWrapper<&mut ITER>) -> Result<Self, Self::Error> {
        let _ = value.0.next().ok_or(TrackError::OutOfSpace)?;
        let event_tag = value.0.next().ok_or(TrackError::OutOfSpace)?;

        println!("{:x}", event_tag);
        todo!("Parse Meta Event")
    }
}
