//! Parser errors.
use core::str;
use std::{
    fmt::{self},
    io,
};

use thiserror::Error;

use crate::rw;

use super::generators::BencodeType;

/// Errors that can occur while parsing a bencoded value.
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// R/W error.
    #[error("R/W error: {0}")]
    Rw(#[from] rw::error::Error),

    /// Read byte after peeking does match peeked byte.
    ///
    /// The main parser peeks one byte ahead to know what kind of bencoded value
    /// is being parsed. If the byte read after peeking does not match the
    /// peeked byte, it means the input is being consumed somewhere else.
    #[error("Read byte after peeking does match peeked byte; {0}")]
    ReadByteAfterPeekingDoesMatchPeekedByte(ReadContext),

    /// Unrecognized first byte for new bencoded value.
    ///
    /// The main parser peeks one byte ahead to know what kind of bencoded value
    /// is being parsed. This error is raised when the peeked byte is not a
    /// valid first byte for a bencoded value.
    #[error("Unrecognized first byte for new bencoded value; {0}")]
    UnrecognizedFirstBencodeValueByte(ReadContext),

    // Integers
    /// Unexpected byte parsing integer.
    ///
    /// The main parser parses integers by reading bytes until it finds the
    /// end of the integer. This error is raised when the byte read is not a
    /// valid byte for an integer bencoded value.
    #[error("Unexpected byte parsing integer; {0}")]
    UnexpectedByteParsingInteger(ReadContext),

    /// Unexpected end of input parsing integer.
    ///
    /// The input ends before the integer ends.
    #[error("Unexpected end of input parsing integer; {0}")]
    UnexpectedEndOfInputParsingInteger(ReadContext),

    /// Leading zeros in integers are not allowed, for example b'i00e'.
    #[error("Leading zeros in integers are not allowed, for example b'i00e'; {0}")]
    LeadingZerosInIntegersNotAllowed(ReadContext),

    // Strings
    /// Invalid string length byte, expected a digit.
    ///
    /// The string parser found an invalid byte for the string length. The
    /// length can only be made of digits (0-9).
    #[error("Invalid string length byte, expected a digit; {0}")]
    InvalidStringLengthByte(ReadContext),

    /// Unexpected end of input parsing string length.
    ///
    /// The input ends before the string length ends.
    #[error("Unexpected end of input parsing string length; {0}")]
    UnexpectedEndOfInputParsingStringLength(ReadContext),

    /// Unexpected end of input parsing string value.
    ///
    /// The input ends before the string value ends.
    #[error("Unexpected end of input parsing string value; {0}")]
    UnexpectedEndOfInputParsingStringValue(ReadContext),

    // Lists
    /// Unexpected end of input parsing list. Expecting first list item or list end.
    #[error(
        "Unexpected end of input parsing list. Expecting first list item or list end; {0}; {1}"
    )]
    UnexpectedEndOfInputExpectingFirstListItemOrEnd(ReadContext, WriteContext),

    /// Unexpected end of input parsing list. Expecting next list item.
    #[error("Unexpected end of input parsing list. Expecting next list item; {0}; {1}")]
    UnexpectedEndOfInputExpectingNextListItem(ReadContext, WriteContext),

    // Dictionaries
    /// Unexpected end of input parsing dictionary. Expecting first dictionary field or dictionary end.
    #[error("Unexpected end of input parsing dictionary. Expecting first dictionary field or dictionary end; {0}; {1}")]
    UnexpectedEndOfInputExpectingFirstDictFieldOrEnd(ReadContext, WriteContext),

    /// Unexpected end of input parsing dictionary. Expecting dictionary field value.
    #[error(
        "Unexpected end of input parsing dictionary. Expecting dictionary field value; {0}; {1}"
    )]
    UnexpectedEndOfInputExpectingDictFieldValue(ReadContext, WriteContext),

    /// Unexpected end of input parsing dictionary. Expecting dictionary field key or end.
    #[error(
        "Unexpected end of input parsing dictionary. Expecting dictionary field key or end; {0}; {1}"
    )]
    UnexpectedEndOfInputExpectingDictFieldKeyOrEnd(ReadContext, WriteContext),

    /// Unexpected end of dictionary. Premature end of dictionary.
    #[error("Unexpected end of dictionary. Premature end of dictionary; {0}; {1}")]
    PrematureEndOfDict(ReadContext, WriteContext),

    /// Expected string for dictionary field key.
    #[error("Expected string for dictionary field key, but got: {0}, {1}")]
    ExpectedStringForDictKeyGot(BencodeType, ReadContext, WriteContext),

    // List and dictionaries
    /// Unexpected end of list or dict. No matching start for the list or dict end.
    #[error(
        "Unexpected end of list or dict. No matching start for the list or dict end: {0}, {1}"
    )]
    NoMatchingStartForListOrDictEnd(ReadContext, WriteContext),
}

/// The reader context when the error occurred.
#[derive(Debug)]
pub struct ReadContext {
    /// The read byte that caused the error if any.
    pub byte: Option<u8>,

    /// The position of the read byte that caused the error.
    pub pos: u64,

    /// The latest bytes read from input.
    pub latest_bytes: Vec<u8>,
}

impl fmt::Display for ReadContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "read context:")?;

        match self.byte {
            None => {}
            Some(byte) => write!(f, " byte `{}` (char: `{}`),", byte, byte as char)?,
        }

        write!(
            f,
            " input pos {}, latest input bytes dump: {:?}",
            self.pos, self.latest_bytes
        )?;

        if let Ok(utf8_string) = str::from_utf8(&self.latest_bytes) {
            write!(f, " (UTF-8 string: `{utf8_string}`)")?;
        }

        Ok(())
    }
}

/// The writer context when the error occurred.
#[derive(Debug)]
pub struct WriteContext {
    /// The written byte that caused the error if any.
    pub byte: Option<u8>,

    /// The position of the written byte that caused the error.
    pub pos: u64,

    /// The latest bytes written to the output.
    pub latest_bytes: Vec<u8>,
}

impl fmt::Display for WriteContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "write context:")?;

        match self.byte {
            None => {}
            Some(byte) => write!(f, " byte `{}` (char: `{}`),", byte, byte as char)?,
        }

        write!(
            f,
            " output pos {}, latest output bytes dump: {:?}",
            self.pos, self.latest_bytes
        )?;

        if let Ok(utf8_string) = str::from_utf8(&self.latest_bytes) {
            write!(f, " (UTF-8 string: `{utf8_string}`)")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    mod for_read_context {
        use crate::error::ReadContext;

        #[test]
        fn it_should_display_the_read_context() {
            let read_context = ReadContext {
                byte: Some(b'a'),
                pos: 10,
                latest_bytes: vec![b'a', b'b', b'c'],
            };

            assert_eq!( read_context.to_string(),"read context: byte `97` (char: `a`), input pos 10, latest input bytes dump: [97, 98, 99] (UTF-8 string: `abc`)");
        }

        #[test]
        fn it_should_not_display_the_byte_if_it_is_none() {
            let read_context = ReadContext {
                byte: None,
                pos: 10,
                latest_bytes: vec![b'a', b'b', b'c'],
            };

            assert_eq!(read_context.to_string(), "read context: input pos 10, latest input bytes dump: [97, 98, 99] (UTF-8 string: `abc`)");
        }

        #[test]
        fn it_should_not_display_the_latest_bytes_as_string_if_it_is_not_a_valid_string() {
            let read_context = ReadContext {
                byte: None,
                pos: 10,
                latest_bytes: vec![b'\xFF', b'\xFE'],
            };

            assert_eq!(
                read_context.to_string(),
                "read context: input pos 10, latest input bytes dump: [255, 254]"
            );
        }
    }

    mod for_write_context {
        use crate::error::WriteContext;

        #[test]
        fn it_should_display_the_read_context() {
            let read_context = WriteContext {
                byte: Some(b'a'),
                pos: 10,
                latest_bytes: vec![b'a', b'b', b'c'],
            };

            assert_eq!( read_context.to_string(),"write context: byte `97` (char: `a`), output pos 10, latest output bytes dump: [97, 98, 99] (UTF-8 string: `abc`)");
        }

        #[test]
        fn it_should_not_display_the_byte_if_it_is_none() {
            let read_context = WriteContext {
                byte: None,
                pos: 10,
                latest_bytes: vec![b'a', b'b', b'c'],
            };

            assert_eq!(read_context.to_string(), "write context: output pos 10, latest output bytes dump: [97, 98, 99] (UTF-8 string: `abc`)");
        }

        #[test]
        fn it_should_not_display_the_latest_bytes_as_string_if_it_is_not_a_valid_string() {
            let read_context = WriteContext {
                byte: None,
                pos: 10,
                latest_bytes: vec![b'\xFF', b'\xFE'],
            };

            assert_eq!(
                read_context.to_string(),
                "write context: output pos 10, latest output bytes dump: [255, 254]"
            );
        }
    }
}
