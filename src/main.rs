//! Converts Bencode to JSON.
//!
//! Usage:
//!
//! Using stdin and stdout:
//!
//! ```text
//! echo "i42e" | cargo run
//! ```
//!
//! Using files:
//!
//! ```text
//! cargo run -- -i ./tests/fixtures/sample.bencode -o output.json
//! ```
use bencode2json::generators::json::Generator;
use clap::{Arg, Command};
use std::fs::File;
use std::io::{self, Read, Write};

fn main() {
    run();
}

fn run() {
    let matches = Command::new("bencode2json")
        .version("0.1.0")
        .author("Torrust Organization")
        .about("Converts Bencode to JSON")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .default_value(None)
                .help("Optional input file (defaults to stdin)"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .default_value(None)
                .help("Optional output file (defaults to stdout)"),
        )
        .get_matches();

    // Handle input stream (file or stdin)
    let input: Box<dyn Read> = if let Some(input_path) = matches.get_one::<String>("input") {
        match File::open(input_path) {
            Ok(file) => Box::new(file),
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    } else {
        Box::new(io::stdin())
    };

    // Handle output stream (file or stdout)
    let mut output: Box<dyn Write> = if let Some(output_path) = matches.get_one::<String>("output")
    {
        match File::create(output_path) {
            Ok(file) => Box::new(file),
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    } else {
        Box::new(io::stdout())
    };

    if let Err(e) = Generator::new(input).write_bytes(&mut output) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
