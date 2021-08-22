//! Common API related to errors in amount

// Standard paths
use std::error;
use std::fmt;

// Crate paths
use crate::api::currency::error::CurrencyError;

#[derive(Debug)]
pub enum TransactionsProcessorError {
    CannotReadInputFile(String, csv::Error),
    CannotReadInputFileHeaders(String, csv::Error),
    CannotReadInputFileRecord(String, csv::Error),
    CannotDeserializeRecord(String, csv::Error),
    CannotBuildCurrencyValue(CurrencyError),
}

// Add empty Error trait
impl error::Error for TransactionsProcessorError {}

fn desc(amount_error: &TransactionsProcessorError) -> String {
    use self::TransactionsProcessorError::*;
    match *amount_error {
        CannotReadInputFile(ref file, ref err) => {
            format!("cannot read input file: {}, reason: {}", file, err)
        }
        CannotReadInputFileHeaders(ref file, ref err) => format!(
            "cannot read required csv header in input file: {}, reason: {}",
            file, err
        ),
        CannotReadInputFileRecord(ref file, ref err) => format!(
            "cannot read csv record in input file: {}, reason: {}",
            file, err
        ),
        CannotDeserializeRecord(ref file, ref err) => format!(
            "cannot deserialize csv record in input file: {}, reason: {}",
            file, err
        ),
        CannotBuildCurrencyValue(ref err) => {
            format!("cannot build currency value, reason: {}", err)
        }
    }
}

// Implement Display trait
impl fmt::Display for TransactionsProcessorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", desc(&self))
    }
}

impl From<CurrencyError> for TransactionsProcessorError {
    fn from(err: CurrencyError) -> TransactionsProcessorError {
        TransactionsProcessorError::CannotBuildCurrencyValue(err)
    }
}
