// Standard paths
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command; // Run programs // Used for writing assertions

// External paths
use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*;

fn run_prepared_transactions(
    testname: &str,
    input: &str,
    output: String,
    stderr: &str,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all("tmp")?;
    let file = Path::new("tmp").join(testname);
    fs::write(&file, input)?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg(file);
    cmd.assert()
        .success()
        .stdout(output)
        .stderr(predicate::str::contains(stderr));

    Ok(())
}

#[test]
fn deposit() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
";
    let output = String::from(
        "client, available, held, total, locked
1,1.0,0.0,1.0,false
",
    );
    let stderr = "";
    run_prepared_transactions("correct_1_deposit", input, output, stderr)?;
    Ok(())
}

#[test]
fn deposit_x2() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
deposit,         1,   2,    1.0
";
    let output = String::from(
        "client, available, held, total, locked
1,2.0,0.0,2.0,false
",
    );
    let stderr = "";
    run_prepared_transactions("deposit_x2", input, output, stderr)?;
    Ok(())
}

#[test]
fn deposit_withdrawal_full() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
withdrawal,      1,   2,    1.0
";
    let output = String::from(
        "client, available, held, total, locked
1,0.0,0.0,0.0,false
",
    );
    let stderr = "";
    run_prepared_transactions("deposit_withdrawal_full", input, output, stderr)?;
    Ok(())
}

#[test]
fn deposit_withdrawal_partially() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
withdrawal,      1,   2,    0.5
";
    let output = String::from(
        "client, available, held, total, locked
1,0.5000,0.0,0.5000,false
",
    );
    let stderr = "";
    run_prepared_transactions("deposit_withdrawal_partially", input, output, stderr)?;
    Ok(())
}

#[test]
fn deposit_withdrawal_too_much() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
withdrawal,      1,   2,    2.0
";
    let output = String::from(
        "client, available, held, total, locked
1,1.0,0.0,1.0,false
",
    );
    let stderr = "CannotWithdrawal";
    run_prepared_transactions("deposit_withdrawal_too_much", input, output, stderr)?;
    Ok(())
}

#[test]
fn deposit_withdrawal_x2() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
withdrawal,      1,   2,    0.5
withdrawal,      1,   3,    0.5
";
    let output = String::from(
        "client, available, held, total, locked
1,0.0,0.0,0.0,false
",
    );
    let stderr = "";
    run_prepared_transactions("deposit_withdrawal_x2", input, output, stderr)?;
    Ok(())
}

#[test]
fn deposit_x2_transaction_non_unique() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
deposit,         1,   1,    1.0
";
    let output = String::from(
        "client, available, held, total, locked
1,1.0,0.0,1.0,false
",
    );
    let stderr = "TransactionNotUnique";
    run_prepared_transactions("deposit_x2_transaction_non_unique", input, output, stderr)?;
    Ok(())
}

#[test]
fn deposit_withdrawal_transaction_non_unique() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
withdrawal,      1,   1,    1.0
";
    let output = String::from(
        "client, available, held, total, locked
1,1.0,0.0,1.0,false
",
    );
    let stderr = "TransactionNotUnique";
    run_prepared_transactions(
        "deposit_withdrawal_transaction_non_unique",
        input,
        output,
        stderr,
    )?;
    Ok(())
}

#[test]
fn withdrawal_from_non_existing_account() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
withdrawal,      1,   1,    1.0
";
    let output = String::from(
        "client, available, held, total, locked
",
    );
    let stderr = "AccountDoesNotExist";
    run_prepared_transactions(
        "withdrawal_from_non_existing_account",
        input,
        output,
        stderr,
    )?;
    Ok(())
}

#[test]
fn dispute() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
dispute,         1,   1,
";
    let output = String::from(
        "client, available, held, total, locked
1,0.0,1.0,1.0,false
",
    );
    let stderr = "";
    run_prepared_transactions("dispute", input, output, stderr)?;
    Ok(())
}

#[test]
fn dispute_non_existing_transaction() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
dispute,         1,   1,
";
    let output = String::from(
        "client, available, held, total, locked
",
    );
    let stderr = "CannotFindTransaction";
    run_prepared_transactions("dispute_non_existing_transaction", input, output, stderr)?;
    Ok(())
}

#[test]
fn dispute_x2() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
dispute,         1,   1,
dispute,         1,   1,
";
    let output = String::from(
        "client, available, held, total, locked
1,0.0,1.0,1.0,false
",
    );
    let stderr = "DisputeAlreadyDisputed";
    run_prepared_transactions("dispute_x2", input, output, stderr)?;
    Ok(())
}

#[test]
fn dispute_non_existing_account() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
dispute,         2,   1,
";
    let output = String::from(
        "client, available, held, total, locked
1,1.0,0.0,1.0,false
",
    );
    let stderr = "CannotFindAccount";
    run_prepared_transactions("dispute_non_existing_account", input, output, stderr)?;
    Ok(())
}

#[test]
fn dispute_available_too_less() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
withdrawal,      1,   2,    0.6
dispute,         1,   2,
";
    let output = String::from(
        "client, available, held, total, locked
1,0.4000,0.0,0.4000,false
",
    );
    let stderr = "DisputeCannotSubstractAvailable";
    run_prepared_transactions("dispute_available_too_less", input, output, stderr)?;
    Ok(())
}

#[test]
fn dispute_hold_too_much() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1000000000000000.0
dispute,         1,   1,
deposit,         1,   2,    1000000000000000.0
dispute,         1,   2,
";
    let output = String::from(
        "client, available, held, total, locked
1,0.0,1000000000000000.0,1000000000000000.0,false
",
    );
    let stderr = "DisputeCannotAddHeld";
    run_prepared_transactions("dispute_hold_too_much", input, output, stderr)?;
    Ok(())
}

#[test]
fn resolve() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
dispute,         1,   1,
resolve,         1,   1,
";
    let output = String::from(
        "client, available, held, total, locked
1,1.0,0.0,1.0,false
",
    );
    let stderr = "";
    run_prepared_transactions("resolve", input, output, stderr)?;
    Ok(())
}

#[test]
fn resolve_transaction_not_found() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
dispute,         1,   1,
resolve,         1,   2,
";
    let output = String::from(
        "client, available, held, total, locked
1,0.0,1.0,1.0,false
",
    );
    let stderr = "CannotFindTransaction";
    run_prepared_transactions("resolve_transaction_not_found", input, output, stderr)?;
    Ok(())
}

#[test]
fn resolve_account_not_found() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
dispute,         1,   1,
resolve,         2,   1,
";
    let output = String::from(
        "client, available, held, total, locked
1,0.0,1.0,1.0,false
",
    );
    let stderr = "CannotFindAccount";
    run_prepared_transactions("resolve_account_not_found", input, output, stderr)?;
    Ok(())
}

#[test]
fn resolve_transaction_not_disputed() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
resolve,         1,   1,
";
    let output = String::from(
        "client, available, held, total, locked
1,1.0,0.0,1.0,false
",
    );
    let stderr = "TransactionNotDisputed";
    run_prepared_transactions("resolve_transaction_not_disputed", input, output, stderr)?;
    Ok(())
}

#[test]
fn resolve_available_too_high_to_add_more() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1000000000000000.0
dispute,         1,   1,
deposit,         1,   2,    1000000000000000.0
resolve,         1,   1,
";
    // Total of available and held is also too high to successfully sum them and properly print
    let output = String::from(
        "client, available, held, total, locked
1,1000000000000000.0,1000000000000000.0,1000000000000000.0,false
",
    );
    let stderr = "ResolveCannotAddAvailable";
    run_prepared_transactions(
        "resolve_available_too_high_to_add_more",
        input,
        output,
        stderr,
    )?;
    Ok(())
}

#[test]
fn resolve_x2_mixed() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    0.4
deposit,         1,   2,    0.6
dispute,         1,   2,
dispute,         1,   1,
resolve,         1,   1,
resolve,         1,   2,
";
    // Total of available and held is also too high to successfully sum them and properly print
    let output = String::from(
        "client, available, held, total, locked
1,1.0,0.0,1.0,false
",
    );
    let stderr = "";
    run_prepared_transactions("resolve_x2_mixed", input, output, stderr)?;
    Ok(())
}

#[test]
fn chargeback() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
dispute,         1,   1,
chargeback,      1,   1,
";
    // Total of available and held is also too high to successfully sum them and properly print
    let output = String::from(
        "client, available, held, total, locked
1,0.0,0.0,0.0,true
",
    );
    let stderr = "";
    run_prepared_transactions("chargeback", input, output, stderr)?;
    Ok(())
}

#[test]
fn chargeback_withdrawal_account_locked() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.0
dispute,         1,   1,
chargeback,      1,   1,
withdrawal,      1,   2,    1.0
";
    // Total of available and held is also too high to successfully sum them and properly print
    let output = String::from(
        "client, available, held, total, locked
1,0.0,0.0,0.0,true
",
    );
    let stderr = "AccountLocked";
    run_prepared_transactions(
        "chargeback_withdrawal_account_locked",
        input,
        output,
        stderr,
    )?;
    Ok(())
}
