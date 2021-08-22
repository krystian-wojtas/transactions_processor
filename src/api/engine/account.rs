// Crate paths
use crate::api::currency::Currency;

pub struct Account {
    pub available: Currency,
    pub held: Currency,
    pub locked: bool,
}

impl Default for Account {
    fn default() -> Self {
        Self {
            available: Currency::new(0, 0).unwrap(),
            held: Currency::new(0, 0).unwrap(),
            locked: false,
        }
    }
}
