use std::convert::TryFrom;
use std::fmt;

// Crate paths
use crate::api::currency::error::CurrencyError;

// Crate modules
pub mod error;

const PRECISION: usize = 4;
const BASE: u64 = 10_u64.pow(PRECISION as u32);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Currency(u64);

// TODO generic types for decimal and fractional
impl Currency {
    pub fn new(decimal: u64, fractional: u64) -> Result<Self, CurrencyError> {
        if fractional >= BASE {
            return Err(CurrencyError::FractionalOutOfRange(fractional));
        }

        let value = decimal
            .checked_mul(BASE)
            .ok_or_else(|| CurrencyError::DecimalMultipliedByPrecisionOutOfRange(decimal))?;

        let value = value
            .checked_add(fractional)
            .ok_or_else(|| CurrencyError::DecimalAddedFractionalOutOfRange(decimal, fractional))?;

        Ok(Self(value))
    }

    pub fn max() -> Self {
        // Go through checks in new to never bypass them
        // Should never panic unless logic is buggy
        Self::new(u64::MAX / BASE, u64::MAX % BASE).unwrap()
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

        let fractional = parts.next().unwrap_or("0");
        if fractional.len() > PRECISION {
            return Err(CurrencyError::FractionalTooLong(fractional.to_string()));
        }
        let fractional = String::from(fractional) + &"0".repeat(PRECISION - fractional.len());
        let fractional = fractional
            .parse::<u64>()
            .map_err(|err| CurrencyError::CannotParseFractionalPart(err))?;

        Self::new(decimal, fractional)
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let decimal = self.0 / BASE;
        let fractional = self.0 % BASE;

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
        assert!(Currency::new(u64::MAX / BASE, 0).is_ok());
    }

    #[test]
    fn correct_max_value() {
        // Should not panic
        Currency::max();
    }

    #[test]
    fn incorrect_fractional_out_of_range() -> Result<(), ()> {
        match Currency::new(0, BASE) {
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
    fn correct_add_0_to_max() {
        let mut first = Currency::max();
        let second = Currency::new(0, 0).unwrap();
        assert!(first.add(second).is_ok());
    }

    #[test]
    fn incorrect_add_overflow() -> Result<(), ()> {
        let mut first = Currency::max();
        let second = Currency::new(0, 1).unwrap();
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
    fn ok_to_parse_without_fractional_part() {
        assert!(Currency::try_from("0").is_ok());
    }

    #[test]
    fn ok_to_parse_long_fractional() {
        let amount = String::from("0.") + &"1".repeat(PRECISION);
        assert!(Currency::try_from(amount.as_str()).is_ok());
    }

    #[test]
    fn cannot_parse_too_long_fractional() -> Result<(), ()> {
        let amount = String::from("0.") + &"1".repeat(PRECISION + 1);
        match Currency::try_from(amount.as_str()) {
            Err(CurrencyError::FractionalTooLong(_)) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    fn compare_parsed_fractional_part() {
        let expected = Currency::new(0, BASE / 10).unwrap();
        let parsed = Currency::try_from("0.1").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn cannot_parse_words() -> Result<(), ()> {
        match Currency::try_from("Not a Number") {
            Err(CurrencyError::CannotParseDecimalPart(_)) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    fn cannot_parse_words_in_fraction_part() -> Result<(), ()> {
        match Currency::try_from("0.NaN") {
            Err(CurrencyError::CannotParseFractionalPart(_)) => Ok(()),
            _ => Err(()),
        }
    }
}
