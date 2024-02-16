use std::num::{ParseFloatError, ParseIntError};

use genomap::GenomeMapError;
use thiserror::Error;

use crate::Position;

#[derive(Debug, Error)]
pub enum GRangesError {
    // IO related errors
    #[error("File reading eror: {0}")]
    IOError(#[from] std::io::Error),

    // File parsing related errors
    #[error("Integer parsing error: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("Float parsing error: {0}")]
    ParseFloatError(#[from] ParseFloatError),
    #[error("Bed-like file has too few columns. The first three columns must be sequence name, and start and end positions.\nLine: {0}")]
    BedlikeTooFewColumns(String),
    #[error("File has invalid column type entry: {0}")]
    InvalidColumnType(String),

    // Invalid genomic range errors
    #[error("Range invalid: start ({0}) must be greater than end ({1})")]
    InvalidGenomicRange(Position, Position),
    #[error("Range [{0}, {1}] is invalid for sequence of length {2}")]
    InvalidGenomicRangeForSequence(Position, Position, Position),
    #[error("Sequence name '{0}' is not the ranges container")]
    MissingSequence(String),
    #[error("Error encountered in genomap::GenomeMap")]
    GenomeMapError(#[from] GenomeMapError),
}