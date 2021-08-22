use std::convert::TryFrom;
use std::fmt;

// Crate paths
use crate::api::currency::error::CurrencyError;

// Crate modules
pub mod error;

pub const PRECISION: u64 = 10000;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Currency(u64);

// TODO generic types for decimal and fractional
impl Currency {
    pub fn new(decimal: u64, fractional: u64) -> Result<Self, CurrencyError> {
        if fractional >= PRECISION {
            return Err(CurrencyError::FractionalOutOfRange(fractional));
        }

        let value = decimal
            .checked_mul(PRECISION)
            .ok_or_else(|| CurrencyError::DecimalMultipliedByPrecisionOutOfRange(decimal))?;

        let value = value
            .checked_add(fractional)
            .ok_or_else(|| CurrencyError::DecimalAddedFractionalOutOfRange(decimal, fractional))?;

        Ok(Self(value))
    }

    pub fn add(&mut self, other: Self) -> Result<(), CurrencyError> {
        self.0 = self
            .0
            .checked_add(other.0)
            .ok_or_else(|| CurrencyError::AddingOtherOutOfRange)?;

        Ok(())
    }

    pub fn substract(&mut self, other: Self) -> Result<(), CurrencyError> {
        self.0 = self
            .0
            .checked_sub(other.0)
            .ok_or_else(|| CurrencyError::SubstractingOtherNegative)?;

        Ok(())
    }
}

impl TryFrom<&str> for Currency {
    type Error = CurrencyError;

    fn try_from(input: &str) -> Result<Self, CurrencyError> {
        let mut parts = input.split('.');

        // Even when input is empty, desimal part is read from iterator as empty
        let decimal = parts
            .next()
            .ok_or_else(|| CurrencyError::CannotGetDecimalPart)?;
        let decimal = decimal
            .parse::<u64>()
            .map_err(|err| CurrencyError::CannotParseDecimalPart(err))?;

        let fractional = parts
            .next()
            .ok_or_else(|| CurrencyError::CannotGetFractionalPart)?;
        let fractional = fractional
            .parse::<u64>()
            .map_err(|err| CurrencyError::CannotParseFractionalPart(err))?;

        Self::new(decimal, fractional)
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let decimal = self.0 / PRECISION;
        let fractional = self.0 % PRECISION;

        write!(f, "{}.{}", decimal, fractional)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn correct_min_value() {
        assert!(Currency::new(0, 0).is_ok());
    }

    #[test]
    fn correct_max_decimal_min_fractional() {
        assert!(Currency::new(u64::MAX / PRECISION, 0).is_ok());
    }

    #[test]
    fn correct_max_value() {
        assert!(Currency::new(u64::MAX / PRECISION - 1, PRECISION - 1).is_ok());
    }

    #[test]
    fn incorrect_fractional_out_of_range() -> Result<(), ()> {
        match Currency::new(0, PRECISION) {
            Err(CurrencyError::FractionalOutOfRange(_)) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    fn correct_add() {
        let mut first = Currency::new(1, 1).unwrap();
        let second = Currency::new(2, 2).unwrap();
        assert!(first.add(second).is_ok());
    }

    #[test]
    fn incorrect_add_overflow() -> Result<(), ()> {
        let mut first = Currency::new(u64::MAX / PRECISION - 1, 0).unwrap();
        let second = Currency::new(u64::MAX / PRECISION - 1, 0).unwrap();
        match first.add(second) {
            Err(CurrencyError::AddingOtherOutOfRange) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    fn correct_substract() {
        let mut first = Currency::new(1, 1).unwrap();
        let second = Currency::new(1, 1).unwrap();
        assert!(first.substract(second).is_ok());
    }

    #[test]
    fn incorrect_substract_underflow() -> Result<(), ()> {
        let mut first = Currency::new(1, 1).unwrap();
        let second = Currency::new(2, 2).unwrap();
        match first.substract(second) {
            Err(CurrencyError::SubstractingOtherNegative) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    fn cannot_multiply_precision_out_of_range() -> Result<(), ()> {
        match Currency::new(u64::MAX, 0) {
            Err(CurrencyError::DecimalMultipliedByPrecisionOutOfRange(_)) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    fn cannot_parse_empty_string() -> Result<(), ()> {
        match Currency::try_from("") {
            Err(CurrencyError::CannotParseDecimalPart(_)) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    fn cannot_parse_fractional_part_when_not_provided() {
        assert_eq!(
            Currency::try_from("0").unwrap_err(),
            CurrencyError::CannotGetFractionalPart
        );
    }

    #[test]
    fn cannot_parse_words() -> Result<(), ()> {
        match Currency::try_from("not a number") {
            Err(CurrencyError::CannotParseDecimalPart(_)) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    fn cannot_parse_words_in_fraction_part() -> Result<(), ()> {
        match Currency::try_from("0.NotANumber") {
            Err(CurrencyError::CannotParseFractionalPart(_)) => Ok(()),
            _ => Err(()),
        }
    }
}
