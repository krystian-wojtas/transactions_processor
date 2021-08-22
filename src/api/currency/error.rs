//! Common API related to errors in amount

// Standard modules paths
use std::error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
pub enum CurrencyError {
    CannotGetDecimalPart,
    CannotParseDecimalPart(ParseIntError),
    CannotParseFractionalPart(ParseIntError),
    FractionalTooLong(String),
    DecimalMultipliedByPrecisionOutOfRange(u64),
    DecimalAddedFractionalOutOfRange(u64, u64),
    FractionalOutOfRange(u64),
    AddingOtherOutOfRange,
    SubstractingOtherNegative,
}

// Add empty Error trait
impl error::Error for CurrencyError {}

fn desc(amount_error: &CurrencyError) -> String {
    use self::CurrencyError::*;
    match *amount_error {
        CannotGetDecimalPart => "cannot find decimal part of amount before . character".to_string(),
        CannotParseDecimalPart(ref err) => format!("cannot parse decimal part of amount: {}", err),
        CannotParseFractionalPart(ref err) => {
            format!("cannot parse fractional part of amount: {}", err)
        }
        FractionalTooLong(ref fractional) => format!(
            "cannot parse fractional parst of amount as it is too long: {}",
            fractional
        ),
        DecimalMultipliedByPrecisionOutOfRange(decimal) => format!(
            "cannot represent amount as value: {} is out of supported range",
            decimal
        ),
        DecimalAddedFractionalOutOfRange(decimal, fractional) => format!(
            "cannot represent amount as decimal: {} and fractional: {} are out of supported range",
            decimal, fractional
        ),
        FractionalOutOfRange(fractional) => format!(
            "cannot represent amount fractional: {} is out of supported range",
            fractional
        ),
        AddingOtherOutOfRange => {
            format!("cannot add other value as it would be out of supported range",)
        }
        SubstractingOtherNegative => {
            format!("cannot substract other value as it would be negative",)
        }
    }
}

// Implement Display trait
impl fmt::Display for CurrencyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", desc(&self))
    }
}
