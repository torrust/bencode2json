//! Run with:
//!
//! ```not_rust
//! cargo run --example parser_file_in_file_out -- -i ./tests/fixtures/sample.bencode -o output.json
//! ```
//!
//! It should create the `output.json`  with this content: `["spam"]`.
use std::{
    fs::File,
    io::{Read, Write},
};

use bencode2json::generators::json::Generator;
use clap::{Arg, Command};

fn main() {
    let matches = Command::new("parser_file_in_file_out")
        .version("0.1.0")
        .author("Torrust Organization")
        .about("Converts Bencode to JSON")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .help("Input file"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file"),
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
        eprintln!("Error: missing input file path. Provide a file path with -i or --input");
        std::process::exit(1);
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
        eprintln!("Error: missing output file path. Provide a file path with -o or --output");
        std::process::exit(1);
    };

    if let Err(e) = Generator::new(input).write_bytes(&mut output) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
