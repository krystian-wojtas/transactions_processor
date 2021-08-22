// Standard paths
use std::convert::TryFrom;

// Crate paths
use api::currency::Currency;
use api::engine::Engine;
use api::error::TransactionsProcessorError;
use api::transactions::Transaction;
use api::transactions::Type;

// Crate modules
pub mod api;

fn dispatch(
    engine: &mut Engine,
    transaction: &Transaction,
) -> Result<(), TransactionsProcessorError> {
    match transaction.type_ {
        Type::Deposit => {
            // Ensure required field is provided
            let amount = transaction.amount.ok_or_else(|| {
                TransactionsProcessorError::MissedMandatoryAmountInInputRecordDeposit
            })?;
            // Parse input string into Currency type
            let amount = Currency::try_from(amount).map_err(|err| {
                TransactionsProcessorError::CannotParseMandatoryInputAmountInInputRecordDeposit(
                    amount.to_string(),
                    err,
                )
            })?;

            engine.deposit(transaction.client, transaction.tx, amount)?;

            Ok(())
        }
        _ => unimplemented!(),
    }
}

pub fn process(file: &str) -> Result<(), TransactionsProcessorError> {
    let mut engine = Engine::new();

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

        dispatch(&mut engine, &transaction)?;
    }

    Ok(())
}
