//! Bencoded string parser.
//!
//! It reads bencoded bytes from the input and writes JSON bytes to the output.
use std::io::{self, Read};

use crate::rw::byte_reader::ByteReader;

/* todo: Optimize UTF-8 conversion. Try to convert to string partially and stop
    converting if we reach a point when input is not valid UTF-8 anymore. This
    way we don't consume more memory and we can print the bytes directly to the
    output from that point on.
*/

use core::str;

use super::error::{Error, ReadContext};

/// It parses a string bencoded value.
///
/// # Errors
///
/// Will return an error if it can't read from the input.
///
/// # Panics
///
/// Will panic if we reach the end of the input without completing the string.
pub fn parse<R: Read>(reader: &mut ByteReader<R>) -> Result<Vec<u8>, Error> {
    let mut string_parser = StringParser::default();
    string_parser.parse(reader)
}

/// Strings bencode format have two parts: `length:value`.
///
/// - Length is a sequence of bytes (only digits 0..9).
/// - Value is an arbitrary sequence of bytes (not only valid UTF-8).
#[derive(Default, Debug)]
#[allow(clippy::module_name_repetitions)]
struct StringParser {
    /// The final parsed string.
    parsed_value: String,
}

impl StringParser {
    fn parse<R: Read>(&mut self, reader: &mut ByteReader<R>) -> Result<Vec<u8>, Error> {
        let mut length = Length::default();

        length.parse(reader)?;

        let mut value = Value::new(length.number);

        let value_bytes = value.parse(reader)?;

        self.parsed_value = value.utf8();

        Ok(value_bytes)
    }
}

#[derive(Default, Debug)]
struct Length {
    /// A list of parsed bytes. It's only for debugging.
    bytes: Vec<u8>,

    /// The parsed length at the current read digit.
    number: usize,
}

impl Length {
    const END_OF_STRING_LENGTH_BYTE: u8 = b':';

    fn parse<R: Read>(&mut self, reader: &mut ByteReader<R>) -> Result<(), Error> {
        loop {
            let byte = Self::next_byte(reader)?;

            match byte {
                Self::END_OF_STRING_LENGTH_BYTE => {
                    break;
                }
                _ => {
                    self.add_byte(byte, reader)?;
                }
            }
        }

        Ok(())
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
                    return Err(Error::UnexpectedEndOfInputParsingStringLength(
                        ReadContext {
                            byte: None,
                            pos: reader.input_byte_counter(),
                            latest_bytes: reader.captured_bytes(),
                        },
                    ));
                }
                Err(err.into())
            }
        }
    }

    /// It adds a new byte (digit) to the string length.
    ///
    /// # Errors
    ///
    /// Will return an error if the byte is not a digit (0..9).
    fn add_byte<R: Read>(&mut self, byte: u8, reader: &mut ByteReader<R>) -> Result<(), Error> {
        if !byte.is_ascii_digit() {
            return Err(Error::InvalidStringLengthByte(ReadContext {
                byte: Some(byte),
                pos: reader.input_byte_counter(),
                latest_bytes: reader.captured_bytes(),
            }));
        }

        self.bytes.push(byte);

        self.add_digit_to_length(Self::byte_to_digit(byte));

        Ok(())
    }

    /// It converts a byte containing an ASCII digit into a number `usize`.
    fn byte_to_digit(byte: u8) -> usize {
        (byte - b'0') as usize
    }

    /// It adds the new digit to the number.
    fn add_digit_to_length(&mut self, digit: usize) {
        self.number = (self.number * 10) + digit;
    }
}

#[derive(Debug)]
struct Value {
    length: usize,
    bytes: Vec<u8>,
    bytes_counter: usize,
}

impl Value {
    fn new(length: usize) -> Self {
        Self {
            length,
            bytes: vec![],
            bytes_counter: 0,
        }
    }

    fn parse<R: Read>(&mut self, reader: &mut ByteReader<R>) -> Result<Vec<u8>, Error> {
        for _i in 1..=self.length {
            self.add_byte(Self::next_byte(reader)?);
        }

        Ok(self.bytes.clone())
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
                    return Err(Error::UnexpectedEndOfInputParsingStringValue(ReadContext {
                        byte: None,
                        pos: reader.input_byte_counter(),
                        latest_bytes: reader.captured_bytes(),
                    }));
                }
                Err(err.into())
            }
        }
    }

    fn add_byte(&mut self, byte: u8) {
        self.bytes.push(byte);
        self.bytes_counter += 1;
    }

    fn utf8(&self) -> String {
        match str::from_utf8(&self.bytes) {
            Ok(string) => {
                // String only contains valid UTF-8 chars -> print it as it's
                format!("<string>{}</string>", string.to_owned())
            }
            Err(_) => {
                // String contains non valid UTF-8 chars -> print it as hex bytes
                Self::bytes_to_hex(&self.bytes)
            }
        }
    }

    fn bytes_to_hex(data: &[u8]) -> String {
        format!("<hex>{}</hex>", hex::encode(data))
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
        use crate::tokenizer::string::tests::try_bencode_to_json;

        #[test]
        fn bencode_to_json_wrapper_succeeds() {
            assert_eq!(try_bencode_to_json(b"4:spam").unwrap(), r"spam".as_bytes());
        }

        #[test]
        fn bencode_to_json_wrapper_fails() {
            assert!(try_bencode_to_json(b"4:").is_err());
        }
    }

    #[test]
    fn length_can_contain_leading_zeros() {
        assert_eq!(bencode_to_json_unchecked(b"00:"), r"".as_bytes());
    }

    #[test]
    fn empty_string() {
        assert_eq!(bencode_to_json_unchecked(b"0:"), r"".as_bytes());
    }

    #[test]
    fn string_with_tags() {
        assert_eq!(
            bencode_to_json_unchecked(b"8:<string>"),
            r"<string>".as_bytes()
        );
    }

    #[test]
    fn utf8() {
        assert_eq!(bencode_to_json_unchecked(b"4:spam"), r"spam".as_bytes());
    }

    #[test]
    fn non_utf8() {
        assert_eq!(
            bencode_to_json_unchecked(b"4:\xFF\xFE\xFD\xFC"),
            vec![0xFF, 0xFE, 0xFD, 0xFC]
        );
    }

    #[test]
    fn ending_with_bencode_end_char() {
        assert_eq!(bencode_to_json_unchecked(b"1:e"), r"e".as_bytes());
    }

    #[test]
    fn containing_a_reserved_char() {
        assert_eq!(bencode_to_json_unchecked(b"1:i"), r"i".as_bytes());
        assert_eq!(bencode_to_json_unchecked(b"1:l"), r"l".as_bytes());
        assert_eq!(bencode_to_json_unchecked(b"1:d"), r"d".as_bytes());
        assert_eq!(bencode_to_json_unchecked(b"1:l"), r"l".as_bytes());
        assert_eq!(bencode_to_json_unchecked(b"1:e"), r"e".as_bytes());
    }

    #[test]
    fn containing_a_digit() {
        assert_eq!(bencode_to_json_unchecked(b"1:0"), r"0".as_bytes());
        assert_eq!(bencode_to_json_unchecked(b"1:1"), r"1".as_bytes());
        assert_eq!(bencode_to_json_unchecked(b"1:2"), r"2".as_bytes());
        assert_eq!(bencode_to_json_unchecked(b"1:3"), r"3".as_bytes());
        assert_eq!(bencode_to_json_unchecked(b"1:4"), r"4".as_bytes());
        assert_eq!(bencode_to_json_unchecked(b"1:5"), r"5".as_bytes());
        assert_eq!(bencode_to_json_unchecked(b"1:6"), r"6".as_bytes());
        assert_eq!(bencode_to_json_unchecked(b"1:7"), r"7".as_bytes());
        assert_eq!(bencode_to_json_unchecked(b"1:8"), r"8".as_bytes());
        assert_eq!(bencode_to_json_unchecked(b"1:9"), r"9".as_bytes());
    }

    mod should_escape_json {
        use crate::{test::bencode_to_json_unchecked, to_bencode};

        #[test]
        fn containing_a_double_quote() {
            assert_eq!(
                bencode_to_json_unchecked("1:\"".as_bytes()),
                r#""<string>\"</string>""#.to_string()
            );
        }

        #[test]
        fn containing_backslashes() {
            assert_eq!(
                bencode_to_json_unchecked("1:\\".as_bytes()),
                r#""<string>\\</string>""#.to_string()
            );
        }

        #[test]
        fn containing_control_characters() {
            assert_eq!(
                bencode_to_json_unchecked("1:\n".as_bytes()),
                r#""<string>\n</string>""#.to_string()
            );
            assert_eq!(
                bencode_to_json_unchecked("1:\r".as_bytes()),
                r#""<string>\r</string>""#.to_string()
            );
            assert_eq!(
                bencode_to_json_unchecked("1:\t".as_bytes()),
                r#""<string>\t</string>""#.to_string()
            );
        }

        #[test]
        fn containing_unicode_characters() {
            assert_eq!(
                bencode_to_json_unchecked(&to_bencode("ñandú")),
                r#""<string>ñandú</string>""#.to_string()
            );
        }

        #[test]
        fn containing_non_unicode_characters() {
            assert_eq!(
                bencode_to_json_unchecked(&[b'4', b':', 0x80, 0xFF, 0x00, 0xAB]),
                r#""<hex>80ff00ab</hex>""#.to_string()
            );
        }
    }

    mod it_should_fail_parsing_when {
        use std::io::{self, Read};

        use crate::{
            error::Error,
            rw::byte_reader::ByteReader,
            tokenizer::string::{parse, tests::try_bencode_to_json},
        };

        #[test]
        fn it_reaches_the_end_of_the_input_parsing_the_string_length() {
            let incomplete_string_length = b"4";

            let result = try_bencode_to_json(incomplete_string_length);

            assert!(matches!(
                result,
                Err(Error::UnexpectedEndOfInputParsingStringLength { .. })
            ));
        }

        #[test]
        fn it_reaches_the_end_of_the_input_parsing_the_string_value() {
            let incomplete_string_value = b"4:123";

            let result = try_bencode_to_json(incomplete_string_value);

            assert!(matches!(
                result,
                Err(Error::UnexpectedEndOfInputParsingStringValue { .. })
            ));
        }

        #[test]
        fn it_receives_a_non_digit_byte_in_the_string_length() {
            let incomplete_string_value = b"4a:1234";

            let result = try_bencode_to_json(incomplete_string_value);

            assert!(matches!(result, Err(Error::InvalidStringLengthByte { .. })));
        }

        /// Fake reader that fails after reading a certain number of bytes
        struct FaultyReader {
            /// The bytes the reader will return
            bytes: Vec<u8>,

            /// The position in the bytes vector where the reader will fail
            fail_in_pos: usize,

            /// The current number of bytes read
            counter: usize,
        }

        impl FaultyReader {
            fn new(bytes: Vec<u8>, fail_in_pos: usize) -> Self {
                Self {
                    bytes,
                    fail_in_pos,
                    counter: 0,
                }
            }
        }

        impl Read for FaultyReader {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                // Fail exactly at the position set by `fail_in_pos`
                if self.counter >= self.fail_in_pos {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "Permission denied",
                    ));
                }

                // Check if we have any bytes left to read
                if self.counter >= self.bytes.len() {
                    return Ok(0); // No more bytes to read (EOF)
                }

                // Write one byte at a time to the buffer
                buf[0] = self.bytes[self.counter];

                // Increment the counter to reflect one byte read
                self.counter += 1;

                // Return that we read exactly 1 byte
                Ok(1)
            }
        }

        #[test]
        fn it_cannot_read_more_bytes_without_finishing_parsing_the_string_length() {
            let mut reader = ByteReader::new(FaultyReader::new(b"4:spam".to_vec(), 1));

            let result = parse(&mut reader);

            assert!(matches!(result, Err(Error::Io(_))));
        }

        #[test]
        fn it_cannot_read_more_bytes_without_finishing_parsing_the_string_value() {
            let mut reader = ByteReader::new(FaultyReader::new(b"4:spam".to_vec(), 3));

            let result = parse(&mut reader);

            assert!(matches!(result, Err(Error::Io(_))));
        }
    }
}
