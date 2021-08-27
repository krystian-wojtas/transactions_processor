//! Common API related to errors in amount

// External paths
use thiserror::Error;

// Crate paths
use crate::api::currency::error::CurrencyError;
use crate::api::currency::Currency;

#[derive(Error, Debug, PartialEq)]
pub enum EngineError {
    #[error("cannot operate as client: {0} account is locked")]
    AccountLocked(u16),
    #[error("account for client: {0} does not exist")]
    AccountDoesNotExist(u16),
    #[error("cannot find account: {0}")]
    CannotFindAccount(u16),
    #[error("transaction should be uniqe but already exist: {0}")]
    TransactionNotUnique(u32),
    #[error("cannot find transaction: {0}")]
    CannotFindTransaction(u32),
    #[error("cannot deposit: client: {client:?}, transaction: {tx:?}, amount: {amount:?}, reason: {source:?}")]
    CannotDeposit {
        client: u16,
        tx: u32,
        amount: Currency,
        source: CurrencyError,
    },
    #[error("cannot deposit because of exceeded max limit for total: client: {client:?}, transaction: {tx:?}, amount: {amount:?}, available: {available:?}, held {held:?}, reason: {source:?}")]
    CannotDepositTotalExceededMaxLimit {
        client: u16,
        tx: u32,
        amount: Currency,
        available: Currency,
        held: Currency,
        source: CurrencyError,
    },
    #[error("deposit transaction failed due to high concurency, try again: {0}")]
    DepositTryAgain(u32),
    #[error("cannot withdrawal: client: {client:?}, transaction: {tx:?}, amount: {amount:?}, reason: {source:?}")]
    CannotWithdrawal {
        client: u16,
        tx: u32,
        amount: Currency,
        source: CurrencyError,
    },
    #[error("transaction already disputed: {0}")]
    DisputeAlreadyDisputed(u32),
    #[error("cannot substract available funds to dispute: {source:?}")]
    DisputeCannotSubstractAvailable { source: CurrencyError },
    #[error("cannot add held funds: {source:?} to dispute")]
    DisputeCannotAddHeld { source: CurrencyError },
    #[error("transaction was not disputed: {0}")]
    TransactionNotDisputed(u32),
    #[error("cannot substract available funds: {source:?} to resolve")]
    ResolveCannotAddAvailable { source: CurrencyError },
    #[error("cannot add held funds: {source:?} to resolve")]
    ResolveCannotSubstractHeld { source: CurrencyError },
    #[error("cannot add held funds: {source:?} to chargeback")]
    ChargebackCannotSubstractHeld { source: CurrencyError },
}
