//! Common API related to errors in amount

// Standard modules paths
use std::error;
use std::fmt;

// Crate paths
use crate::api::currency::error::CurrencyError;
use crate::api::currency::Currency;

#[derive(Debug, PartialEq)]
pub enum EngineError {
    CannotDeposit(u16, u32, Currency, CurrencyError),
}

// Add empty Error trait
impl error::Error for EngineError {}

fn desc(amount_error: &EngineError) -> String {
    use self::EngineError::*;
    match *amount_error {
        CannotDeposit(client, tx, amount, ref err) => format!(
            "cannot deposit: client: {}, transaction: {}, amount: {}, reason: {}",
            client, tx, amount, err
        ),
    }
}

// Implement Display trait
impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", desc(&self))
    }
}
