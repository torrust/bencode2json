//! This lib contains functions to convert bencoded bytes into a JSON string.
//!
//! Bencode is a simple encoding format that is used to encode arbitrary
//! data structures. It is commonly used in the context of torrent files,
//! where the data structures are used to describe the contents of the torrent
//! file.
//!
//! To learn more about bencode, you can refer to the following resources:
//!
//! - <https://en.wikipedia.org/wiki/Bencode>
//! - <https://www.bittorrent.org/beps/bep_0003.html>
//!
//! Thi lib has high-level functions for common purposes that call the lower
//! level parser. You can use the low-lever parser if the high-level wrappers
//! are not suitable for your needs.
//!
//! The most straightforward way to use this lib is to use the `try_bencode_to_json`
//! function:
//!
//! ```rust
//! use bencode2json::{try_bencode_to_json};
//!
//! let result = try_bencode_to_json(b"d4:spam4:eggse").unwrap();
//!
//! assert_eq!(result, r#"{"<string>spam</string>":"<string>eggs</string>"}"#);
//! ```
//!
//! The primary goal of this lib is to provide a simple and easy-to-use API for
//! converting bencoded data into JSON. It's also designed to be flexible and
//! efficient, making it suitable for a wide range of use cases.
//!
//! A design requirement is to be able to parse bencoded data without building
//! an in-memory representation of the whole bencoded data structure.
//!
//! > __NOTICE__: In the context of this lib, parser is a function that takes an input
//! > containing bencoded data and produces a JSON output (raw bytes or UTF-8 string).
pub mod error;
pub mod generators;
pub mod rw;
pub mod tokenizer;

use error::Error;
use generators::json::Generator;
mod test;

/// It converts bencoded bytes into a JSON string.
///
/// # Errors
///
/// Will return an error if the conversion fails.
pub fn try_bencode_to_json(input_buffer: &[u8]) -> Result<String, Error> {
    let mut output = String::new();

    let mut parser = Generator::new(input_buffer);

    match parser.write_str(&mut output) {
        Ok(()) => Ok(output),
        Err(err) => Err(err),
    }
}

/// Helper to convert a string into a bencoded string.
#[must_use]
pub fn to_bencode(value: &str) -> Vec<u8> {
    let bencoded_str = format!("{}:{}", value.len(), value);
    bencoded_str.as_bytes().to_vec()
}

#[cfg(test)]
mod tests {

    mod converting_bencode_to_json {
        use crate::try_bencode_to_json;

        #[test]
        fn when_it_succeeds() {
            let result = try_bencode_to_json(b"d4:spam4:eggse").unwrap();

            assert_eq!(
                result,
                r#"{"<string>spam</string>":"<string>eggs</string>"}"#
            );
        }

        #[test]
        fn when_it_fails() {
            let result = try_bencode_to_json(b"invalid bencode value");

            assert!(result.is_err());
        }
    }

    mod converting_string_to_bencode {
        use crate::to_bencode;

        #[test]
        fn empty_string() {
            assert_eq!(to_bencode(r""), b"0:");
        }

        #[test]
        fn non_empty_string() {
            assert_eq!(to_bencode(r"alice"), b"5:alice");
        }

        mod string_with_special_chars {
            use crate::to_bencode;

            #[test]
            fn line_break() {
                assert_eq!(to_bencode(r"alice\n"), b"7:alice\x5C\x6E");
            }

            #[test]
            fn utf8_chars() {
                let word = "ñandú";
                assert_eq!(
                    to_bencode(word),
                    format!("{}:{}", word.len(), word).as_bytes()
                );
            }
        }
    }
}
