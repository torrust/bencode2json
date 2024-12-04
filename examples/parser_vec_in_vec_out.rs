//! Run with:
//!
//! ```not_rust
//! cargo run --example parser_vec_in_vec_out
//! ```
//!
//! It prints "spam".
use bencode2json::generators::json::Generator;

fn main() {
    let input = b"4:spam".to_vec();
    let mut output = Vec::new();

    if let Err(e) = Generator::new(&input[..]).write_bytes(&mut output) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    println!("{}", String::from_utf8_lossy(&output));
}
