//! Bencoded integer parser.
//!
//! It reads bencoded bytes from the input and writes JSON bytes to the output.
use std::io::{self, Read};

use crate::rw::byte_reader::ByteReader;

use super::{
    error::{Error, ReadContext},
    BENCODE_END_INTEGER,
};

/// The current state parsing the integer.
#[derive(PartialEq)]
#[allow(clippy::enum_variant_names)]
enum StateExpecting {
    Start,          // S
    DigitOrSign,    // DoS
    DigitAfterSign, // DaS
    DigitOrEnd,     // DoE
}

/// It parses an integer bencoded value.
///
/// # Errors
///
/// Will return an error if it can't read from the input or write to the
/// output.
///
/// # Panics
///
/// Will panic if we reach the end of the input without completing the integer
/// (without reaching the end of the integer `e`).
pub fn parse<R: Read>(reader: &mut ByteReader<R>) -> Result<Vec<u8>, Error> {
    let mut state = StateExpecting::Start;
    let mut first_digit_is_zero = false;
    let mut value = vec![];

    loop {
        let byte = next_byte(reader)?;

        let char = byte as char;

        state = match state {
            StateExpecting::Start => {
                // Discard the 'i' byte
                StateExpecting::DigitOrSign
            }
            StateExpecting::DigitOrSign => {
                if char == '-' {
                    value.push(byte);

                    StateExpecting::DigitAfterSign
                } else if char.is_ascii_digit() {
                    value.push(byte);

                    if char == '0' {
                        first_digit_is_zero = true;
                    }

                    StateExpecting::DigitOrEnd
                } else {
                    return Err(Error::UnexpectedByteParsingInteger(ReadContext {
                        byte: Some(byte),
                        pos: reader.input_byte_counter(),
                        latest_bytes: reader.captured_bytes(),
                    }));
                }
            }
            StateExpecting::DigitAfterSign => {
                if char.is_ascii_digit() {
                    value.push(byte);

                    if char == '0' {
                        first_digit_is_zero = true;
                    }

                    StateExpecting::DigitOrEnd
                } else {
                    return Err(Error::UnexpectedByteParsingInteger(ReadContext {
                        byte: Some(byte),
                        pos: reader.input_byte_counter(),
                        latest_bytes: reader.captured_bytes(),
                    }));
                }
            }
            StateExpecting::DigitOrEnd => {
                if char.is_ascii_digit() {
                    value.push(byte);

                    if char == '0' && first_digit_is_zero {
                        return Err(Error::LeadingZerosInIntegersNotAllowed(ReadContext {
                            byte: Some(byte),
                            pos: reader.input_byte_counter(),
                            latest_bytes: reader.captured_bytes(),
                        }));
                    }

                    StateExpecting::DigitOrEnd
                } else if byte == BENCODE_END_INTEGER {
                    return Ok(value);
                } else {
                    return Err(Error::UnexpectedByteParsingInteger(ReadContext {
                        byte: Some(byte),
                        pos: reader.input_byte_counter(),
                        latest_bytes: reader.captured_bytes(),
                    }));
                }
            }
        };
    }
}

/// It reads the next byte from the input.
///
/// # Errors
///
/// Will return an error if the end of input was reached.
fn next_byte<R: Read>(reader: &mut ByteReader<R>) -> Result<u8, Error> {
    match reader.read_byte() {
        Ok(byte) => Ok(byte),
        Err(err) => {
            if err.kind() == io::ErrorKind::UnexpectedEof {
                return Err(Error::UnexpectedEndOfInputParsingInteger(ReadContext {
                    byte: None,
                    pos: reader.input_byte_counter(),
                    latest_bytes: reader.captured_bytes(),
                }));
            }
            Err(err.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{error::Error, rw::byte_reader::ByteReader};

    use super::parse;

    fn bencode_to_json_unchecked(input_buffer: &[u8]) -> Vec<u8> {
        parse_bencode(input_buffer).expect("Bencode to JSON conversion failed")
    }

    fn try_bencode_to_json(input_buffer: &[u8]) -> Result<Vec<u8>, Error> {
        parse_bencode(input_buffer)
    }

    fn parse_bencode(input_buffer: &[u8]) -> Result<Vec<u8>, Error> {
        let mut reader = ByteReader::new(input_buffer);

        parse(&mut reader)
    }

    mod for_helpers {
        use crate::tokenizer::integer::tests::try_bencode_to_json;

        #[test]
        fn bencode_to_json_wrapper_succeeds() {
            assert_eq!(try_bencode_to_json(b"i0e").unwrap(), "0".as_bytes());
        }

        #[test]
        fn bencode_to_json_wrapper_fails() {
            assert!(try_bencode_to_json(b"i").is_err());
        }
    }

    #[test]
    fn zero() {
        assert_eq!(bencode_to_json_unchecked(b"i0e"), "0".as_bytes());
    }

    #[test]
    fn one_digit_integer() {
        assert_eq!(bencode_to_json_unchecked(b"i1e"), "1".as_bytes());
    }

    #[test]
    fn two_digits_integer() {
        assert_eq!(bencode_to_json_unchecked(b"i42e"), "42".as_bytes());
    }

    #[test]
    fn negative_integer() {
        assert_eq!(bencode_to_json_unchecked(b"i-1e"), "-1".as_bytes());
    }

    mod it_should_fail {
        use std::io::{self, Read};

        use crate::{
            error::Error,
            rw::byte_reader::ByteReader,
            tokenizer::integer::{parse, tests::try_bencode_to_json},
        };

        #[test]
        fn when_it_cannot_read_more_bytes_from_input() {
            let unfinished_int = b"i42";

            let result = try_bencode_to_json(unfinished_int);

            assert!(matches!(
                result,
                Err(Error::UnexpectedEndOfInputParsingInteger { .. })
            ));
        }

        #[test]
        fn when_it_finds_an_invalid_byte() {
            let int_with_invalid_byte = b"iae";

            let result = try_bencode_to_json(int_with_invalid_byte);

            assert!(matches!(
                result,
                Err(Error::UnexpectedByteParsingInteger { .. })
            ));
        }

        #[test]
        fn when_it_finds_leading_zeros() {
            // Leading zeros are not allowed.Only the zero integer can start with zero.

            let int_with_invalid_byte = b"i00e";

            let result = try_bencode_to_json(int_with_invalid_byte);

            assert!(matches!(
                result,
                Err(Error::LeadingZerosInIntegersNotAllowed { .. })
            ));
        }

        #[test]
        fn when_it_finds_leading_zeros_in_a_negative_integer() {
            // Leading zeros are not allowed.Only the zero integer can start with zero.

            let int_with_invalid_byte = b"i-00e";

            let result = try_bencode_to_json(int_with_invalid_byte);

            assert!(matches!(
                result,
                Err(Error::LeadingZerosInIntegersNotAllowed { .. })
            ));
        }

        mod when_it_receives_a_unexpected_byte {
            use crate::{error::Error, tokenizer::integer::tests::try_bencode_to_json};

            #[test]
            fn while_expecting_a_digit_or_sign() {
                let int_with_invalid_byte = b"ia";

                let result = try_bencode_to_json(int_with_invalid_byte);

                assert!(matches!(
                    result,
                    Err(Error::UnexpectedByteParsingInteger { .. })
                ));
            }

            #[test]
            fn while_expecting_digit_after_the_sign() {
                let int_with_invalid_byte = b"i-a";

                let result = try_bencode_to_json(int_with_invalid_byte);

                assert!(matches!(
                    result,
                    Err(Error::UnexpectedByteParsingInteger { .. })
                ));
            }

            #[test]
            fn while_expecting_digit_or_end() {
                let int_with_invalid_byte = b"i-1a";

                let result = try_bencode_to_json(int_with_invalid_byte);

                assert!(matches!(
                    result,
                    Err(Error::UnexpectedByteParsingInteger { .. })
                ));
            }
        }

        #[test]
        fn when_it_receives_a_non_eof_io_error() {
            struct FaultyReader;

            impl Read for FaultyReader {
                fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
                    Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "Permission denied",
                    ))
                }
            }

            let mut reader = ByteReader::new(FaultyReader);

            let result = parse(&mut reader);

            assert!(matches!(result, Err(Error::Io(_))));
        }
    }
}
