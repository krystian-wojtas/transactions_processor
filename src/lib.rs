// Standard paths
use std::convert::TryFrom;
use std::error::Error;

// Crate paths
use api::transactions::Transaction;
use api::currency::Currency;

// Crate modules
pub mod api;

pub fn process(file: &str) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(file)?;

    let mut raw_record = csv::ByteRecord::new();
    let headers = rdr.byte_headers()?.clone();

    while rdr.read_byte_record(&mut raw_record)? {
        let transaction: Transaction = raw_record.deserialize(Some(&headers))?;
        println!("{:?}", transaction);

        let amount = transaction.amount.unwrap();
        let amount = Currency::try_from(amount)?;
        println!("{:?}", amount);
    }

    Ok(())
}
