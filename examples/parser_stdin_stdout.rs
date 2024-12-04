//! Run with:
//!
//! ```not_rust
//! echo "4:spam" | cargo run --example parser_stdin_stdout
//! ```
//!
//! It prints "spam".
use std::io;

use bencode2json::generators::json::Generator;

fn main() {
    let input = Box::new(io::stdin());
    let mut output = Box::new(io::stdout());

    if let Err(e) = Generator::new(input).write_bytes(&mut output) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
