// Standard paths
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::RwLock;

// Crate paths
use self::account::Account;
use self::error::EngineError;
use crate::api::currency::Currency;

// Crate modules
pub mod account;
pub mod error;

pub struct Engine {
    accounts: RwLock<HashMap<u16, Mutex<Account>>>,
    // Should it track client id also and verify later that disputed transactions are valid?
    transactions: RwLock<HashMap<u32, Currency>>,
    transactions_disputed: RwLock<HashSet<u32>>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            accounts: RwLock::new(HashMap::new()),
            transactions: RwLock::new(HashMap::new()),
            transactions_disputed: RwLock::new(HashSet::new()),
        }
    }

    pub fn deposit(&mut self, client: u16, tx: u32, amount: Currency) -> Result<(), EngineError> {
        // Limit lock time
        {
            // Panic if lock is poisoned
            let mut transactions_lock_write = self.transactions.write().unwrap();

            // Does it make sense to track transactions in deposit?
            // Is client going to complain about increasing his available cash?
            // If not, then getting rid of it would save memory
            //
            // Should it check if transaction is unique?
            //
            // If further deposit fails, then transaction is going to be be stored anyway
            // Then repating same transaction with same tx id will fail
            // Always should be used another unique tx id with each transaction
            if transactions_lock_write.insert(tx, amount).is_some() {
                return Err(EngineError::TransactionNotUnique(tx));
            }
        }

        // Limit lock time
        {
            // Panic if lock is poisoned
            let accounts_lock_read = self.accounts.read().unwrap();

            if let Some(mutex) = accounts_lock_read.get(&client) {
                let mut account = mutex
                    .lock()
                    // Panic if mutex is poisoned
                    .unwrap();

                if account.locked {
                    return Err(EngineError::AccountLocked(client));
                }

                account
                    .available
                    .add(amount)
                    .map_err(|source| EngineError::CannotDeposit {
                        client,
                        tx,
                        amount,
                        source,
                    })?;

                return Ok(());
            }
        }

        // Limit lock time
        {
            // Prepare new account with given deposit
            let mut account = Account::default();
            account
                .available
                .add(amount)
                .map_err(|source| EngineError::CannotDeposit {
                    client,
                    tx,
                    amount,
                    source,
                })?;

            // Panic if lock is poisoned
            let mut accounts_lock_write = self.accounts.write().unwrap();

            match accounts_lock_write.entry(client) {
                Entry::Occupied(_) => {
                    // Between getting read of read lock and before getting write lock
                    // Another thread may be lucky enough to deposit to same account
                    // Then don't overwrite already existing account
                    // Instead try deposit again
                    return Err(EngineError::DepositTryAgain(tx));
                }
                Entry::Vacant(entry) => {
                    entry.insert(Mutex::new(account));
                }
            };
        }

        Ok(())
    }

    pub fn withdrawal(
        &mut self,
        client: u16,
        tx: u32,
        amount: Currency,
    ) -> Result<(), EngineError> {
        // Limit lock time
        {
            // Panic if lock is poisoned
            let mut transactions_lock_write = self.transactions.write().unwrap();

            // Should it check if transaction is unique?
            //
            // If further deposit fails, then transaction is going to be be stored anyway
            // Then repating same transaction with same tx id will fail
            // Always should be used another unique tx id with each transaction
            if transactions_lock_write.insert(tx, amount).is_some() {
                return Err(EngineError::TransactionNotUnique(tx));
            }
        }

        // Section with accounts locks
        {
            // Panic if lock is poisoned
            let accounts_lock_read = self.accounts.read().unwrap();

            match accounts_lock_read.get(&client) {
                Some(mutex) => {
                    let mut account = mutex
                        .lock()
                        // Panic if mutex is poisoned
                        .unwrap();

                    if account.locked {
                        return Err(EngineError::AccountLocked(client));
                    }

                    account.available.substract(amount).map_err(|source| {
                        EngineError::CannotWithdrawal {
                            client,
                            tx,
                            amount,
                            source,
                        }
                    })
                }
                None => Err(EngineError::AccountDoesNotExist(client)),
            }?;
        }

        Ok(())
    }

    pub fn dispute(&mut self, client: u16, tx: u32) -> Result<(), EngineError> {
        // Limit lock time
        {
            // Panic if lock is poisoned
            let transactions_disputed_lock_read = self.transactions_disputed.read().unwrap();

            if transactions_disputed_lock_read.contains(&tx) {
                return Err(EngineError::DisputeAlreadyDisputed(tx));
            }
        }

        let amount;
        // Limit lock time
        {
            // Panic if lock is poisoned
            let transactions_lock_read = self.transactions.read().unwrap();

            let amount_ref = transactions_lock_read
                .get(&tx)
                .ok_or_else(|| EngineError::CannotFindTransaction(tx))?;

            amount = amount_ref.clone();
        }

        // Limit lock time
        {
            // Panic if lock is poisoned
            let accounts_lock_read = self.accounts.read().unwrap();
            let mutex = accounts_lock_read
                .get(&client)
                .ok_or_else(|| EngineError::CannotFindAccount(client))?;

            // Panic if lock is poisoned
            let mut account = mutex.lock().unwrap();

            account
                .available
                .substract(amount)
                .map_err(|source| EngineError::DisputeCannotSubstractAvailable { source })?;
            account
                .held
                .add(amount)
                .map_err(|source| EngineError::DisputeCannotAddHeld { source })?;
        }

        // Limit lock time
        {
            // Panic if lock is poisoned
            let mut transactions_disputed_lock_write = self.transactions_disputed.write().unwrap();
            transactions_disputed_lock_write.insert(tx);
        }

        Ok(())
    }

    pub fn resolve(&mut self, client: u16, tx: u32) -> Result<(), EngineError> {
        let amount;
        // Limit lock time
        {
            // Panic if lock is poisoned
            let transactions_lock_read = self.transactions.read().unwrap();

            let amount_ref = transactions_lock_read
                .get(&tx)
                .ok_or_else(|| EngineError::CannotFindTransaction(tx))?;

            amount = amount_ref.clone();
        }

        // Limit lock time
        {
            // Panic if lock is poisoned
            let transactions_disputed_lock_read = self.transactions_disputed.read().unwrap();

            if !transactions_disputed_lock_read.contains(&tx) {
                return Err(EngineError::ResolveTransactionNotDisputed(tx));
            }
        }

        // Limit lock time
        {
            // Panic if lock is poisoned
            let accounts_lock_read = self.accounts.read().unwrap();
            let mutex = accounts_lock_read
                .get(&client)
                .ok_or_else(|| EngineError::CannotFindAccount(client))?;

            // Panic if lock is poisoned
            let mut account = mutex.lock().unwrap();

            account
                .available
                .add(amount)
                .map_err(|source| EngineError::ResolveCannotAddAvailable { source })?;
            account
                .held
                .substract(amount)
                .map_err(|source| EngineError::ResolveCannotSubstractHeld { source })?;
        }

        // Limit lock time
        {
            // Panic if lock is poisoned
            let mut transactions_disputed_lock_write = self.transactions_disputed.write().unwrap();
            transactions_disputed_lock_write.remove(&tx);
        }

        Ok(())
    }

    pub fn chargeback(&mut self, client: u16, tx: u32) -> Result<(), EngineError> {
        let amount;
        // Limit lock time
        {
            // Panic if lock is poisoned
            let transactions_lock_read = self.transactions.read().unwrap();

            let amount_ref = transactions_lock_read
                .get(&tx)
                .ok_or_else(|| EngineError::CannotFindTransaction(tx))?;

            amount = amount_ref.clone();
        }

        // Limit lock time
        {
            // Panic if lock is poisoned
            let transactions_disputed_lock_read = self.transactions_disputed.read().unwrap();

            if !transactions_disputed_lock_read.contains(&tx) {
                return Err(EngineError::ChargebackTransactionNotDisputed(tx));
            }
        }

        // Limit lock time
        {
            // Panic if lock is poisoned
            let accounts_lock_read = self.accounts.read().unwrap();
            let mutex = accounts_lock_read
                .get(&client)
                .ok_or_else(|| EngineError::CannotFindAccount(client))?;

            let mut account = mutex.lock().unwrap();

            account
                .held
                .substract(amount)
                .map_err(|source| EngineError::ChargebackCannotSubstractHeld { source })?;

            account.locked = true;
        }

        // Limit lock time
        {
            // Panic if lock is poisoned
            let mut transactions_disputed_lock_write = self.transactions_disputed.write().unwrap();
            transactions_disputed_lock_write.remove(&tx);
        }

        Ok(())
    }

    pub fn accounts(&self) -> &RwLock<HashMap<u16, Mutex<Account>>> {
        &self.accounts
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::api::currency::error::CurrencyError;
    use assert_matches::assert_matches;

    #[test]
    fn correct_deposit() {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
    }

    #[test]
    fn correct_2_deposits_for_one_account() {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        assert!(engine.deposit(1, 2, amount).is_ok());
    }

    #[test]
    fn incorrect_2_deposits_for_one_account_out_of_range() {
        let mut engine = Engine::new();
        let amount = Currency::max();
        assert!(engine.deposit(1, 1, amount).is_ok());
        assert_matches!(
            engine.deposit(1, 2, amount),
            Err(EngineError::CannotDeposit {
                client: _,
                tx: _,
                amount: _,
                source: CurrencyError::AddingOtherOutOfRange
            })
        );
    }

    #[test]
    fn incorrect_2_deposits_with_same_tx() {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        assert_matches!(
            engine.deposit(1, 1, amount),
            Err(EngineError::TransactionNotUnique(..))
        );
    }

    #[test]
    fn incorrect_2_withdrawals_with_same_tx() {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        assert_matches!(
            engine.withdrawal(1, 1, amount),
            Err(EngineError::TransactionNotUnique(..))
        );
    }

    #[test]
    fn incorrect_withdrawal_from_unexisting_account() {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert_matches!(
            engine.withdrawal(1, 1, amount),
            Err(EngineError::AccountDoesNotExist(..))
        );
    }

    #[test]
    fn correct_withdrawal_from_deposited_account() {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        assert!(engine.withdrawal(1, 2, amount).is_ok());
    }

    #[test]
    fn correct_withdrawal_less_then_deposited() {
        let mut engine = Engine::new();
        let amount_more = Currency::new(2, 2).unwrap();
        let amount_less = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount_more).is_ok());
        assert!(engine.withdrawal(1, 2, amount_less).is_ok());
    }

    #[test]
    fn incorrect_withdrawal_more_then_deposited() {
        let mut engine = Engine::new();
        let amount_less = Currency::new(1, 1).unwrap();
        let amount_more = Currency::new(2, 2).unwrap();
        assert!(engine.deposit(1, 1, amount_less).is_ok());
        assert_matches!(
            engine.withdrawal(1, 2, amount_more),
            Err(EngineError::CannotWithdrawal {
                client: _,
                tx: _,
                amount: _,
                source: CurrencyError::SubstractingOtherNegative
            })
        );
    }

    #[test]
    fn correct_dispute() {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        assert!(engine.dispute(1, 1).is_ok());
    }

    #[test]
    fn incorrect_dispute_twice_some_tx() {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        assert!(engine.dispute(1, 1).is_ok());
        assert_matches!(
            engine.dispute(1, 1),
            Err(EngineError::DisputeAlreadyDisputed(..))
        );
    }

    #[test]
    fn correct_resolve() {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        assert!(engine.dispute(1, 1).is_ok());
        assert!(engine.resolve(1, 1).is_ok());
    }

    #[test]
    fn incorrect_resolve_unexisting_tx() {
        let mut engine = Engine::new();
        assert_matches!(
            engine.resolve(1, 1),
            Err(EngineError::CannotFindTransaction(..))
        );
    }

    #[test]
    fn incorrect_resolve_not_disputed_tx() {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        assert_matches!(
            engine.resolve(1, 1),
            Err(EngineError::ResolveTransactionNotDisputed(..))
        );
    }

    #[test]
    fn correct_chargeback() {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        assert!(engine.dispute(1, 1).is_ok());
        assert!(engine.chargeback(1, 1).is_ok());
    }

    #[test]
    fn incorrect_chargeback_unexisting_tx() {
        let mut engine = Engine::new();
        assert_matches!(
            engine.chargeback(1, 1),
            Err(EngineError::CannotFindTransaction(..))
        );
    }

    #[test]
    fn incorrect_chargeback_not_disputed_tx() {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        assert_matches!(
            engine.chargeback(1, 1),
            Err(EngineError::ChargebackTransactionNotDisputed(..))
        );
    }

    #[test]
    fn incorrect_deposit_on_locked_account_tx() {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        assert!(engine.dispute(1, 1).is_ok());
        assert!(engine.chargeback(1, 1).is_ok());
        assert_matches!(
            engine.deposit(1, 2, amount),
            Err(EngineError::AccountLocked(..))
        );
    }
}
