//! Common API related to errors in amount

// Standard modules paths
use std::num::ParseIntError;

// External paths
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CurrencyError {
    #[error("cannot find decimal part of amount before . character")]
    CannotGetDecimalPart,
    #[error("cannot parse decimal part of amount: {source:?}")]
    CannotParseDecimalPart { source: ParseIntError },
    #[error("cannot parse fractional part of amount: {source:?}")]
    CannotParseFractionalPart { source: ParseIntError },
    #[error("cannot parse fractional parst of amount as it is too long: {0}")]
    FractionalTooLong(String),
    #[error("cannot represent amount as value: {0} is out of supported range")]
    DecimalMultipliedByPrecisionOutOfRange(u64),
    #[error(
        "cannot represent amount as decimal: {0} and fractional: {1} are out of supported range"
    )]
    DecimalAddedFractionalOutOfRange(u64, u64),
    #[error("cannot represent amount fractional: {0} is out of supported range")]
    FractionalOutOfRange(u64),
    #[error("cannot add other value as it would be out of supported range")]
    AddingOtherOutOfRange,
    #[error("cannot substract other value as it would be negative")]
    SubstractingOtherNegative,
}
