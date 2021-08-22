//! Common API related to errors in amount

// Standard paths
use std::error;
use std::fmt;

// Crate paths
use crate::api::currency::error::CurrencyError;
use crate::api::engine::error::EngineError;

#[derive(Debug)]
pub enum TransactionsProcessorError {
    CannotReadInputFile(String, csv::Error),
    CannotReadInputFileHeaders(String, csv::Error),
    CannotReadInputFileRecord(String, csv::Error),
    CannotDeserializeRecord(String, csv::Error),
    CannotBuildCurrencyValue(CurrencyError),
    MissedMandatoryAmountInInputRecordDeposit,
    MissedMandatoryAmountInInputRecordWithdrawal,
    CannotParseMandatoryInputAmountInInputRecordDeposit(String, CurrencyError),
    CannotParseMandatoryInputAmountInInputRecordWithdrawal(String, CurrencyError),
    NestedEngineError(EngineError),
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
        MissedMandatoryAmountInInputRecordDeposit => {
            "input file misses mandatory amount value for deposit operation".to_string()
        }
        MissedMandatoryAmountInInputRecordWithdrawal => {
            "input file misses mandatory amount value for withdrawal operation".to_string()
        }
        CannotParseMandatoryInputAmountInInputRecordDeposit(ref amount, ref err) => format!(
            "cannot parse input amount for deposit: {}, reason: {}",
            amount, err
        ),
        CannotParseMandatoryInputAmountInInputRecordWithdrawal(ref amount, ref err) => format!(
            "cannot parse input amount for withdrawal: {}, reason: {}",
            amount, err
        ),
        NestedEngineError(ref err) => format!("enginge gives error: {}", err),
    }
}

// Implement Display trait
impl fmt::Display for TransactionsProcessorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", desc(&self))
    }
}

impl From<EngineError> for TransactionsProcessorError {
    fn from(err: EngineError) -> TransactionsProcessorError {
        TransactionsProcessorError::NestedEngineError(err)
    }
}
