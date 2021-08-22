// Standard paths
use std::convert::TryFrom;

// Crate paths
use api::currency::Currency;
use api::error::TransactionsProcessorError;
use api::transactions::Transaction;

// Crate modules
pub mod api;

pub fn process(file: &str) -> Result<(), TransactionsProcessorError> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(file)
        .map_err(|err| TransactionsProcessorError::CannotReadInputFile(file.to_string(), err))?;

    let mut raw_record = csv::ByteRecord::new();
    let headers = rdr.byte_headers().map_err(|err| {
        TransactionsProcessorError::CannotReadInputFileHeaders(file.to_string(), err)
    })?;
    let headers = headers.clone();

    while rdr.read_byte_record(&mut raw_record).map_err(|err| {
        TransactionsProcessorError::CannotReadInputFileRecord(file.to_string(), err)
    })? {
        let transaction: Transaction = raw_record.deserialize(Some(&headers)).map_err(|err| {
            TransactionsProcessorError::CannotDeserializeRecord(file.to_string(), err)
        })?;
        println!("{:?}", transaction);

        let amount = transaction.amount.unwrap();
        let amount = Currency::try_from(amount)?;
        println!("{:?}", amount);
    }

    Ok(())
}
