// Standard paths
use std::convert::TryFrom;

// Crate paths
use api::currency::Currency;
use api::engine::Engine;
use api::error::TransactionsProcessorError;
use api::transactions::Transaction;
use api::transactions::Type;

// External paths
use anyhow::anyhow;
use anyhow::Result;

// Crate modules
pub mod api;

pub fn process(file: &str) -> anyhow::Result<()> {
    // Create transaction engine
    let mut engine = Engine::new();

    // Prepare input stream with transactions to process
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(file)
        .map_err(|err| TransactionsProcessorError::CannotReadInputFile {
            file: file.to_string(),
            source: err,
        })?;

    // Read first row which is supposed csv headers
    let mut raw_record = csv::ByteRecord::new();
    let headers = rdr.byte_headers().map_err(|err| {
        TransactionsProcessorError::CannotReadInputFileHeaders {
            file: file.to_string(),
            source: err,
        }
    })?;
    let headers = headers.clone();

    // Main loop to process all transactions
    loop {
        match rdr.read_byte_record(&mut raw_record) {
            // Encountered error during reading record
            Err(err) => {
                let nested_error = TransactionsProcessorError::CannotReadInputFileRecord {
                    file: file.to_string(),
                    source: err,
                };
                // Finish processing with fatal error
                return Err(anyhow!(nested_error));
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
        TransactionsProcessorError::CannotDeserializeRecord {
            file: file.to_string(),
            source: err,
        }
    })?;

    // Dispatach transaction into proper engine call
    dispatch(engine, &transaction)?;

    Ok(())
}

fn get_and_parse_amount(amount: Option<&str>) -> Result<Currency, TransactionsProcessorError> {
    // Ensure required field is provided
    let amount =
        amount.ok_or_else(|| TransactionsProcessorError::MissedMandatoryAmountInInputRecord)?;
    // Parse input string into Currency type
    let amount = Currency::try_from(amount).map_err(|err| {
        TransactionsProcessorError::CannotParseMandatoryInputAmountInInputRecord {
            amount: amount.to_string(),
            source: err,
        }
    })?;

    Ok(amount)
}

fn dispatch(
    engine: &mut Engine,
    transaction: &Transaction,
) -> Result<(), TransactionsProcessorError> {
    match transaction.type_ {
        Type::Deposit => {
            let amount = get_and_parse_amount(transaction.amount)?;

            engine.deposit(transaction.client, transaction.tx, amount)?;

            Ok(())
        }
        Type::Withdrawal => {
            let amount = get_and_parse_amount(transaction.amount)?;

            engine.withdrawal(transaction.client, transaction.tx, amount)?;

            Ok(())
        }
        Type::Dispute => {
            engine.dispute(transaction.client, transaction.tx)?;

            Ok(())
        }
        Type::Resolve => {
            engine.resolve(transaction.client, transaction.tx)?;

            Ok(())
        }
        Type::Chargeback => {
            engine.chargeback(transaction.client, transaction.tx)?;

            Ok(())
        }
    }
}

fn print_accounts(engine: &Engine) {
    // Print csv header
    println!("client, available, held, total, locked");

    let accounts = engine.accounts();

    // Limit lock time
    {
        // Panic if lock is poisoned
        let accounts_lock_read = accounts.read().unwrap();

        for (client, mutex) in accounts_lock_read.iter() {
            let account = mutex.lock().unwrap();

            // Calculate total
            let mut total = account.available.clone();
            // What is better?
            // To refuse operations which exceed total? (Then implement total field in Account)
            // Or to print inacurate total value and warning during structure dump?
            total.add(account.held).unwrap_or_else(|err| {
                eprintln!("WARNING: total is out of range: {:?}", err);
            });

            // Print data
            // To easy to serde or csv crates
            // This way is fastest
            // Speed matters
            println!(
                "{},{},{},{},{}",
                client, account.available, account.held, total, account.locked
            );
        }
    }
}

fn print_record_warning(
    optional_position: Option<&csv::Position>,
    err: TransactionsProcessorError,
) {
    match optional_position {
        Some(position) => {
            eprintln!(
                "WARNING: failed to process record:\nline: {}\nreason: {:?}",
                position.line(),
                err
            );
        }
        None => {
            eprintln!("WARNING: ignored record, reason: {:?}", err);
        }
    };
}
