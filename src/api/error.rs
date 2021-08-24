//! Common API related to errors in amount

// External paths
use thiserror::Error;

// Crate paths
use crate::api::currency::error::CurrencyError;
use crate::api::engine::error::EngineError;

#[derive(Error, Debug)]
pub enum TransactionsProcessorError {
    #[error("cannot read input file: {0}, reason: {1}")]
    CannotReadInputFile(String, csv::Error),
    #[error("cannot read required csv header in input file: {0}, reason: {1}")]
    CannotReadInputFileHeaders(String, csv::Error),
    #[error("cannot read csv record in input file: {0}, reason: {1}")]
    CannotReadInputFileRecord(String, csv::Error),
    #[error("cannot deserialize csv record in input file: {0}, reason: {1}")]
    CannotDeserializeRecord(String, csv::Error),
    #[error("cannot build currency value, reason: {0}")]
    CannotBuildCurrencyValue(CurrencyError),
    #[error("input file misses mandatory amount value")]
    MissedMandatoryAmountInInputRecord,
    #[error("cannot parse input amount: {0}, reason: {1}")]
    CannotParseMandatoryInputAmountInInputRecord(String, CurrencyError),
    #[error("engine gives error: {0}")]
    NestedEngineError(EngineError),
}

impl From<EngineError> for TransactionsProcessorError {
    fn from(err: EngineError) -> TransactionsProcessorError {
        TransactionsProcessorError::NestedEngineError(err)
    }
}
