// Standard paths
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Mutex;

// Crate paths
use self::account::Account;
use self::error::EngineError;
use crate::api::currency::Currency;

// Crate modules
pub mod account;
pub mod error;

pub struct Engine {
    accounts: HashMap<u16, Mutex<Account>>,
    // Should it track client id also and verify later that disputed transactions are valid?
    transactions: HashMap<u32, Currency>,
    transactions_disputed: HashSet<u32>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
            transactions_disputed: HashSet::new(),
        }
    }

    pub fn deposit(&mut self, client: u16, tx: u32, amount: Currency) -> Result<(), EngineError> {
        // Does it make sense to track transactions in deposit?
        // Is client going to complain about increasing his available cash?
        // If not, then getting rid of it would save memory
        //
        // Should it checke if transaction is unique?
        if self.transactions.insert(tx, amount).is_some() {
            return Err(EngineError::DepositTransactionNotUnique(tx));
        }

        match self.accounts.get_mut(&client) {
            Some(mutex) => {
                let mut account = mutex
                    .lock()
                    // Panic if mutex is poisoned
                    .unwrap();

                account
                    .available
                    .add(amount)
                    .map_err(|err| EngineError::CannotDeposit(client, tx, amount, err))?;
            }
            None => {
                let mut account = Account::default();
                account
                    .available
                    .add(amount)
                    .map_err(|err| EngineError::CannotDeposit(client, tx, amount, err))?;
                self.accounts.insert(client, Mutex::new(account));
            }
        };

        Ok(())
    }

    pub fn withdrawal(
        &mut self,
        client: u16,
        tx: u32,
        amount: Currency,
    ) -> Result<(), EngineError> {
        // Should it checke if transaction is unique?
        if self.transactions.insert(tx, amount).is_some() {
            return Err(EngineError::WithdrawalTransactionNotUnique(tx));
        }

        match self.accounts.get_mut(&client) {
            Some(mutex) => {
                let mut account = mutex
                    .lock()
                    // Panic if mutex is poisoned
                    .unwrap();

                account
                    .available
                    .substract(amount)
                    .map_err(|err| EngineError::CannotWithdrawal(client, tx, amount, err))
            }
            None => Err(EngineError::AccountDoesNotExist(client)),
        }?;

        Ok(())
    }

    pub fn dispute(&mut self, client: u16, tx: u32) -> Result<(), EngineError> {
        if self.transactions_disputed.contains(&tx) {
            return Err(EngineError::DisputeAlreadyDisputed(tx));
        }
        let amount = self
            .transactions
            .get(&tx)
            .ok_or_else(|| EngineError::DisputeCannotFindTransaction(tx))?;

        let mutex = self
            .accounts
            .get_mut(&client)
            .ok_or_else(|| EngineError::DisputeCannotFindAccount(client))?;

        // Limit mutex lock time
        {
            let mut account = mutex.lock().unwrap();

            account
                .available
                .substract(*amount)
                .map_err(|err| EngineError::DisputeCannotSubstractAvailable(err))?;
            account
                .held
                .add(*amount)
                .map_err(|err| EngineError::DisputeCannotAddHeld(err))?;
        }

        self.transactions_disputed.insert(tx);

        Ok(())
    }

    pub fn resolve(&mut self, client: u16, tx: u32) -> Result<(), EngineError> {
        let amount = self
            .transactions
            .get(&tx)
            .ok_or_else(|| EngineError::ResolveCannotFindTransaction(tx))?;

        if !self.transactions_disputed.contains(&tx) {
            return Err(EngineError::ResolveTransactionNotDisputed(tx));
        }

        let mutex = self
            .accounts
            .get_mut(&client)
            .ok_or_else(|| EngineError::ResolveCannotFindAccount(client))?;

        // Limit mutex lock time
        {
            let mut account = mutex.lock().unwrap();

            account
                .available
                .add(*amount)
                .map_err(|err| EngineError::ResolveCannotAddAvailable(err))?;
            account
                .held
                .substract(*amount)
                .map_err(|err| EngineError::ResolveCannotSubstractHeld(err))?;
        }

        self.transactions_disputed.remove(&tx);

        Ok(())
    }

    pub fn chargeback(&mut self, client: u16, tx: u32) -> Result<(), EngineError> {
        let amount = self
            .transactions
            .get(&tx)
            .ok_or_else(|| EngineError::ChargebackCannotFindTransaction(tx))?;

        if !self.transactions_disputed.contains(&tx) {
            return Err(EngineError::ChargebackTransactionNotDisputed(tx));
        }

        let mutex = self
            .accounts
            .get_mut(&client)
            .ok_or_else(|| EngineError::ChargebackCannotFindAccount(client))?;

        // Limit mutex lock time
        {
            let mut account = mutex.lock().unwrap();

            account
                .held
                .substract(*amount)
                .map_err(|err| EngineError::ChargebackCannotSubstractHeld(err))?;
        }

        self.transactions_disputed.remove(&tx);

        Ok(())
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<u16, Mutex<Account>> {
        self.accounts.iter()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::api::currency::error::CurrencyError;

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
    fn incorrect_2_deposits_for_one_account_out_of_range() -> Result<(), ()> {
        let mut engine = Engine::new();
        let amount = Currency::max();
        assert!(engine.deposit(1, 1, amount).is_ok());
        match engine.deposit(1, 2, amount) {
            Err(EngineError::CannotDeposit(_, _, _, CurrencyError::AddingOtherOutOfRange)) => {
                Ok(())
            }
            _ => Err(()),
        }
    }

    #[test]
    fn incorrect_2_deposits_with_same_tx() -> Result<(), ()> {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        match engine.deposit(1, 1, amount) {
            Err(EngineError::DepositTransactionNotUnique(_)) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    fn incorrect_2_withdrawals_with_same_tx() -> Result<(), ()> {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        match engine.withdrawal(1, 1, amount) {
            Err(EngineError::WithdrawalTransactionNotUnique(_)) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    fn incorrect_withdrawal_from_unexisting_account() -> Result<(), ()> {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        match engine.withdrawal(1, 1, amount) {
            Err(EngineError::AccountDoesNotExist(_)) => Ok(()),
            _ => Err(()),
        }
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
    fn incorrect_withdrawal_more_then_deposited() -> Result<(), ()> {
        let mut engine = Engine::new();
        let amount_less = Currency::new(1, 1).unwrap();
        let amount_more = Currency::new(2, 2).unwrap();
        assert!(engine.deposit(1, 1, amount_less).is_ok());
        match engine.withdrawal(1, 2, amount_more) {
            Err(EngineError::CannotWithdrawal(
                _,
                _,
                _,
                CurrencyError::SubstractingOtherNegative,
            )) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    fn correct_dispute() {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        assert!(engine.dispute(1, 1).is_ok());
    }

    #[test]
    fn incorrect_dispute_twice_some_tx() -> Result<(), ()> {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        assert!(engine.dispute(1, 1).is_ok());
        match engine.dispute(1, 1) {
            Err(EngineError::DisputeAlreadyDisputed(_)) => Ok(()),
            _ => Err(()),
        }
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
    fn incorrect_resolve_unexisting_tx() -> Result<(), ()> {
        let mut engine = Engine::new();
        match engine.resolve(1, 1) {
            Err(EngineError::ResolveCannotFindTransaction(_)) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    fn incorrect_resolve_not_disputed_tx() -> Result<(), ()> {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        match engine.resolve(1, 1) {
            Err(EngineError::ResolveTransactionNotDisputed(_)) => Ok(()),
            _ => Err(()),
        }
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
    fn incorrect_chargeback_unexisting_tx() -> Result<(), ()> {
        let mut engine = Engine::new();
        match engine.chargeback(1, 1) {
            Err(EngineError::ChargebackCannotFindTransaction(_)) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    fn incorrect_chargeback_not_disputed_tx() -> Result<(), ()> {
        let mut engine = Engine::new();
        let amount = Currency::new(1, 1).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        match engine.chargeback(1, 1) {
            Err(EngineError::ChargebackTransactionNotDisputed(_)) => Ok(()),
            _ => Err(()),
        }
    }
}
