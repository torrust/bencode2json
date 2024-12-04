//! Run with:
//!
//! ```not_rust
//! cargo run --example parser_string_in_string_out
//! ```
//!
//! It prints "spam".
use bencode2json::generators::json::Generator;

fn main() {
    let input = "4:spam".to_string();
    let mut output = String::new();

    if let Err(e) = Generator::new(input.as_bytes()).write_str(&mut output) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    println!("{output}");
}
