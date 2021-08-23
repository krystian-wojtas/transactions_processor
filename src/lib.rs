// Standard paths
use std::convert::TryFrom;

// Crate paths
use api::currency::Currency;
use api::engine::account::Account;
use api::engine::Engine;
use api::error::TransactionsProcessorError;
use api::transactions::Transaction;
use api::transactions::Type;

// Crate modules
pub mod api;

pub fn process(file: &str) -> Result<(), TransactionsProcessorError> {
    // Create transaction engine
    let mut engine = Engine::new();

    // Prepare input stream with transactions to process
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(file)
        .map_err(|err| TransactionsProcessorError::CannotReadInputFile(file.to_string(), err))?;

    // Read first row which is supposed csv headers
    let mut raw_record = csv::ByteRecord::new();
    let headers = rdr.byte_headers().map_err(|err| {
        TransactionsProcessorError::CannotReadInputFileHeaders(file.to_string(), err)
    })?;
    let headers = headers.clone();

    // Main loop to process all transactions
    loop {
        match rdr.read_byte_record(&mut raw_record) {
            // Encountered error during reading record
            Err(err) => {
                let nested_error =
                    TransactionsProcessorError::CannotReadInputFileRecord(file.to_string(), err);
                // Finish processing with fatal error
                return Err(nested_error);
                // Or only print warning if error is not considered fatal
                // and continue processing any following records
                // print_record_warning(&raw_record, nested_error);
            }
            // End of input csv file, finish
            Ok(false) => break,
            // Record is read into buffer
            Ok(true) => {
                // Process record
                // If any errors, then print them as warnings and continue with others
                process_record(&mut engine, &raw_record, &headers, file).unwrap_or_else(|err| {
                    print_record_warning(raw_record.position(), err);
                });
            }
        }
    }

    print_accounts(&engine);

    Ok(())
}

fn process_record(
    engine: &mut Engine,
    raw_record: &csv::ByteRecord,
    headers: &csv::ByteRecord,
    file: &str,
) -> Result<(), TransactionsProcessorError> {
    // Try to deserialize record into assumed structure
    let transaction: Transaction = raw_record.deserialize(Some(&headers)).map_err(|err| {
        TransactionsProcessorError::CannotDeserializeRecord(file.to_string(), err)
    })?;

    // Dispatach transaction into proper engine call
    dispatch(engine, &transaction)?;

    Ok(())
}

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
        Type::Withdrawal => {
            // Ensure required field is provided
            let amount = transaction.amount.ok_or_else(|| {
                TransactionsProcessorError::MissedMandatoryAmountInInputRecordWithdrawal
            })?;
            // Parse input string into Currency type
            let amount = Currency::try_from(amount).map_err(|err| {
                TransactionsProcessorError::CannotParseMandatoryInputAmountInInputRecordWithdrawal(
                    amount.to_string(),
                    err,
                )
            })?;

            engine.withdrawal(transaction.client, transaction.tx, amount)?;

            Ok(())
        }
        // Type::Dispute => {
        //     engine.dispute(transaction.client, transaction.tx)?;

        //     Ok(())
        // }
        _ => unimplemented!(),
    }
}

fn print_accounts(engine: &Engine) {
    // Print csv header
    println!("client, available, held, total, locked");

    for (
        client,
        Account {
            available,
            held,
            locked,
        },
    ) in engine.iter()
    {
        // Calculate total
        let mut total = available.clone();
        // What is better?
        // To refuse operations which exceed total? (Then implement total field in Account)
        // Or to print inacurate total value and warning during structure dump?
        total.add(*held).unwrap_or_else(|err| {
            eprintln!("WARNING: total is out of range: {}", err);
        });

        // Print data
        println!("{},{},{},{},{}", client, available, held, total, locked);
    }
}

fn print_record_warning(
    optional_position: Option<&csv::Position>,
    err: TransactionsProcessorError,
) {
    match optional_position {
        Some(position) => {
            eprintln!(
                "WARNING: ignored record in line: {}, reason: {}",
                position.line(),
                err
            );
        }
        None => {
            eprintln!("WARNING: ignored record, reason: {}", err);
        }
    };
}
