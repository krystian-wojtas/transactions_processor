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
    #[error("cannot deposit: client: {0}, transaction: {1}, amount: {2}, reason: {3}")]
    CannotDeposit(u16, u32, Currency, CurrencyError),
    #[error("cannot withdrawal: client: {0}, transaction: {1}, amount: {2}, reason: {3}")]
    CannotWithdrawal(u16, u32, Currency, CurrencyError),
    #[error("account for client: {0} does not exist")]
    AccountDoesNotExist(u16),
    #[error("deposit transaction should be uniqe but already exist: {0}")]
    DepositTransactionNotUnique(u32),
    #[error("deposit transaction failed due to high concurency, try again: {0}")]
    DepositTryAgain(u32),
    #[error("withdrawal transaction should be uniqe but already exist: {0}")]
    WithdrawalTransactionNotUnique(u32),
    #[error("transaction already disputed: {0}")]
    DisputeAlreadyDisputed(u32),
    #[error("cannot find transaction to dispute: {0}")]
    DisputeCannotFindTransaction(u32),
    #[error("cannot find account to dispute: {0}")]
    DisputeCannotFindAccount(u16),
    #[error("cannot substract available funds to dispute: {0}")]
    DisputeCannotSubstractAvailable(CurrencyError),
    #[error("cannot add held funds: {0} to dispute")]
    DisputeCannotAddHeld(CurrencyError),
    #[error("cannot resolve transaction which was not disputed: {0}")]
    ResolveTransactionNotDisputed(u32),
    #[error("cannot find transaction to resolve: {0}")]
    ResolveCannotFindTransaction(u32),
    #[error("cannot find account to chargeback: {0}")]
    ResolveCannotFindAccount(u16),
    #[error("cannot substract available funds: {0} to resolve")]
    ResolveCannotAddAvailable(CurrencyError),
    #[error("cannot add held funds: {0} to resolve")]
    ResolveCannotSubstractHeld(CurrencyError),
    #[error("cannot chargeback transaction which was not disputed: {0}")]
    ChargebackTransactionNotDisputed(u32),
    #[error("cannot find transaction to chargeback: {0}")]
    ChargebackCannotFindTransaction(u32),
    #[error("cannot find account to chargeback: {0}")]
    ChargebackCannotFindAccount(u16),
    #[error("cannot substract available funds: {0} to chargeback")]
    ChargebackCannotAddAvailable(CurrencyError),
    #[error("cannot add held funds: {0} to chargeback")]
    ChargebackCannotSubstractHeld(CurrencyError),
}
