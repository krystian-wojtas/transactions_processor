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
fn parse_decimal_out_of_range() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    10000000000000000.0
";
    let output = String::from(
        "client, available, held, total, locked
",
    );
    let stderr = "DecimalMultipliedByPrecisionOutOfRange";
    run_prepared_transactions("parse_decimal_out_of_range", input, output, stderr)?;
    Ok(())
}

#[test]
fn parse_amount_fractional_too_long() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,    1.00001
";
    let output = String::from(
        "client, available, held, total, locked
",
    );
    let stderr = "FractionalTooLong";
    run_prepared_transactions("parse_amount_fractional_too_long", input, output, stderr)?;
    Ok(())
}

#[test]
fn parse_missed_decimal() -> Result<(), Box<dyn Error>> {
    let input = "type,       client,  tx, amount
deposit,         1,   1,
";
    let output = String::from(
        "client, available, held, total, locked
",
    );
    let stderr = "MissedMandatoryAmountInInputRecord";
    run_prepared_transactions("parse_missed_decimal", input, output, stderr)?;
    Ok(())
}
