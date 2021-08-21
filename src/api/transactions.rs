use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize)]
pub struct Transaction<'a> {
    #[serde(rename = "type")]
    pub type_: Type,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<&'a str>,
}
