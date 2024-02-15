//! Range and Range Containers.
//!
//!

use crate::{error::GRangesError, Position};

pub mod coitrees;
pub mod vec;
pub mod operations;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RangeEmpty {
    pub start: Position,
    pub end: Position,
}

unsafe impl Sync for RangeEmpty {}
unsafe impl Send for RangeEmpty {}

impl RangeEmpty {
    /// Create a new 0-indexed right-exclusive range.
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

/// [`RangeIndexed`] is a range with a valid
/// index to a data element in the data container.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RangeIndexed {
    pub start: Position,
    pub end: Position,
    pub index: usize,
}

unsafe impl Sync for RangeIndexed {}
unsafe impl Send for RangeIndexed {}

impl RangeIndexed {
    /// Create a new 0-indexed right-exclusive range.
    pub fn new(start: Position, end: Position, index: usize) -> Self {
        Self { start, end, index }
    }
}

/// Represents a parsed range entry, possibly containing some data.
#[derive(Debug, Clone, PartialEq)]
pub struct RangeRecord<U> {
    pub seqname: String,
    pub start: Position,
    pub end: Position,
    pub data: U,
}

/// Represents a range entry, possible with indices to sequence name and data.
#[derive(Debug, Clone, PartialEq)]
pub struct RangeIndexedRecord {
    pub seqname_index: usize,
    pub start: Position,
    pub end: Position,
    pub index: usize,
}

impl RangeIndexedRecord {
    pub fn new(seqname_index: usize, start: Position, end: Position, index: usize) -> Self {
        Self {
            seqname_index,
            start,
            end,
            index,
        }
    }
}

/// Validates whether a given range is valid for accessing a sequence of a given `length`.
///
/// # Arguments
///
/// * `range` - The range to validate.
/// * `length` - The length of the sequence.
///
/// # Returns
///
/// * `bool` - `true` if the range is valid for the sequence; otherwise, `false`.
pub fn validate_range(
    start: Position,
    end: Position,
    length: Position,
) -> Result<(), GRangesError> {
    if start > end {
        return Err(GRangesError::InvalidGenomicRange(start, end));
    }

    if end >= length {
        return Err(GRangesError::InvalidGenomicRangeForSequence(
            start, end, length,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_range;
    use crate::prelude::*;

    #[test]
    fn test_invalid_range_start_end() {
        let result = validate_range(5, 1, 10);
        assert!(matches!(
            result,
            Err(GRangesError::InvalidGenomicRange(5, 1))
        ));
    }

    #[test]
    fn test_valid_range_length() {
        let result = validate_range(1, 10, 11);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_range_length() {
        let result = validate_range(1, 10, 10);
        assert!(matches!(
            result,
            Err(GRangesError::InvalidGenomicRangeForSequence(1, 10, 10))
        ));
    }
}
