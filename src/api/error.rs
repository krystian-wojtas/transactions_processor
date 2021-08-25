//! Common API related to errors in amount

// External paths
use thiserror::Error;

// Crate paths
use crate::api::currency::error::CurrencyError;
use crate::api::engine::error::EngineError;

#[derive(Error, Debug)]
pub enum TransactionsProcessorError {
    #[error("cannot read input file: {file:?}, reason: {source:?}")]
    CannotReadInputFile { file: String, source: csv::Error },
    #[error("cannot read required csv header in input file: {file:?}, reason: {source:?}")]
    CannotReadInputFileHeaders { file: String, source: csv::Error },
    #[error("cannot read csv record in input file: {file:?}, reason: {source:?}")]
    CannotReadInputFileRecord { file: String, source: csv::Error },
    #[error("cannot deserialize csv record in input file: {file:?}, reason: {source:?}")]
    CannotDeserializeRecord { file: String, source: csv::Error },
    #[error("input file misses mandatory amount value")]
    MissedMandatoryAmountInInputRecord,
    #[error("cannot parse input amount: {amount:?}, reason: {source:?}")]
    CannotParseMandatoryInputAmountInInputRecord {
        amount: String,
        source: CurrencyError,
    },
    #[error("engine gives error")]
    NestedEngineError(#[from] EngineError),
}
