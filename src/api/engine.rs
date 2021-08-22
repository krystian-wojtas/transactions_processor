// Standard paths
use std::collections::HashMap;

// Crate paths
use self::account::Account;
use self::error::EngineError;
use crate::api::currency::Currency;

// Crate modules
mod account;
pub mod error;

pub struct Engine {
    accounts: HashMap<u16, Account>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            accounts: HashMap::new(),
        }
    }

    pub fn deposit(&mut self, client: u16, tx: u32, amount: Currency) -> Result<(), EngineError> {
        match self.accounts.get_mut(&client) {
            Some(account) => {
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
                self.accounts.insert(client, account);
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::api::currency::error::CurrencyError;
    use crate::api::currency::PRECISION;

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
        assert!(engine.deposit(1, 1, amount).is_ok());
    }

    #[test]
    fn incorrect_2_deposits_for_one_account_out_of_range() -> Result<(), ()> {
        let mut engine = Engine::new();
        let amount = Currency::new(u64::MAX / PRECISION, 0).unwrap();
        assert!(engine.deposit(1, 1, amount).is_ok());
        match engine.deposit(1, 1, amount) {
            Err(EngineError::CannotDeposit(_, _, _, CurrencyError::AddingOtherOutOfRange)) => {
                Ok(())
            }
            _ => Err(()),
        }
    }

}