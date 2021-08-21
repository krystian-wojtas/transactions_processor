#![forbid(unsafe_code)]

// Standard paths
use std::env;
use std::process;

// Crate paths
use transactions_processor::process;

fn main() {
    let file = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: <executable> <transactions>");
        process::exit(1);
    });

    if let Err(err) = process(&file) {
        println!("Error: {}", err);
        process::exit(1);
    }
}
