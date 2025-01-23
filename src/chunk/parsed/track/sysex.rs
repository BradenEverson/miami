//! System Exclusive Messages

use super::{event::IteratorWrapper, TrackError};

/// A midi system exclusize event message
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SysexEvent;

impl<ITER> TryFrom<IteratorWrapper<&mut ITER>> for SysexEvent
where
    ITER: Iterator<Item = u8>,
{
    type Error = TrackError;
    fn try_from(value: IteratorWrapper<&mut ITER>) -> Result<Self, Self::Error> {
        todo!("Parse System Exclusive Event")
    }
}
