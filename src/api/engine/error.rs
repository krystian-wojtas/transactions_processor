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
    CannotWithdrawal(u16, u32, Currency, CurrencyError),
    AccountDoesNotExist(u16),
    DepositTransactionNotUnique(u32),
    WithdrawalTransactionNotUnique(u32),
    DisputeAlreadyDisputed(u32),
    DisputeCannotFindTransaction(u32),
    DisputeCannotFindAccount(u16),
    DisputeCannotSubstractAvailable(CurrencyError),
    DisputeCannotAddHeld(CurrencyError),
    ResolveTransactionNotDisputed(u32),
    ResolveCannotFindTransaction(u32),
    ResolveCannotFindAccount(u16),
    ResolveCannotAddAvailable(CurrencyError),
    ResolveCannotSubstractHeld(CurrencyError),
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
        CannotWithdrawal(client, tx, amount, ref err) => format!(
            "cannot withdrawal: client: {}, transaction: {}, amount: {}, reason: {}",
            client, tx, amount, err
        ),
        AccountDoesNotExist(client) => format!("account for client: {} does not exist", client),
        DepositTransactionNotUnique(tx) => {
            format!(
                "deposit transaction should be uniqe but already exist: {}",
                tx
            )
        }
        WithdrawalTransactionNotUnique(tx) => {
            format!(
                "withdrawal transaction should be uniqe but already exist: {}",
                tx
            )
        }
        DisputeAlreadyDisputed(tx) => {
            format!("transaction already disputed: {}", tx)
        }
        DisputeCannotFindTransaction(tx) => {
            format!("cannot find transaction to dispute: {}", tx)
        }
        DisputeCannotFindAccount(tx) => {
            format!("cannot find account to dispute: {}", tx)
        }
        DisputeCannotSubstractAvailable(ref err) => {
            format!("cannot substract available funds to dispute: {}", err)
        }
        DisputeCannotAddHeld(ref err) => {
            format!("cannot add held funds: {} to dispute", err)
        }
        ResolveTransactionNotDisputed(tx) => {
            format!("cannot resolve transaction which was not disputed: {}", tx)
        }
        ResolveCannotFindTransaction(tx) => {
            format!("cannot find transaction to resolve: {}", tx)
        }
        ResolveCannotFindAccount(tx) => {
            format!("cannot find account to resolve: {}", tx)
        }
        ResolveCannotAddAvailable(ref err) => {
            format!("cannot substract available funds: {} to resolve", err)
        }
        ResolveCannotSubstractHeld(ref err) => {
            format!("cannot add held funds: {} to resolve", err)
        }
    }
}

// Implement Display trait
impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", desc(&self))
    }
}
