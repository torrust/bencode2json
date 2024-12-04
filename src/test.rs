//! Helpers for testing.

/// It converts bencoded bytes into a JSON string.
///
/// # Panics
///
/// Will panic if the conversion fails.
#[cfg(test)]
#[must_use]
pub(crate) fn bencode_to_json_unchecked(input_buffer: &[u8]) -> String {
    use crate::generators::json::Generator;

    let mut output = String::new();

    let mut parser = Generator::new(input_buffer);

    parser
        .write_str(&mut output)
        .expect("Bencode to JSON conversion failed");

    output
}

/// Generates a vector of bytes representing `n` nested empty Bencode
/// lists.
///
/// This function is a helper for generating a Bencode representation
/// of a JSON array with `n` nested empty arrays. It repeats the Bencode
///  list opener `l` character `n` times, and the Bencode list closer
/// `e` character `n` times, to create the desired nested structure.
///
/// # Examples
///
/// ```rust
/// let nested_bencode = generate_n_nested_empty_bencoded_lists(2);
/// assert_eq!(nested_bencode, b"llee");
/// ```
#[cfg(test)]
pub(crate) fn generate_n_nested_empty_bencoded_lists(n: usize) -> Vec<u8> {
    let mut bencode_value = vec![b'l'; n];
    bencode_value.extend_from_slice(&vec![b'e'; n]);
    bencode_value
}

/// Generates a JSON array with `n` nested empty arrays.
///
/// This function is a helper for generating a JSON array with a
/// specific number of nested empty arrays. It repeats the opening `[`
/// character `n` times, and the closing `]` character `n` times, to
/// create the desired nested structure.
///
/// # Examples
///
/// ```rust
/// let nested_json = generate_n_nested_empty_json_arrays(2);
/// assert_eq!(nested_json, "[[]]");
/// ```
#[cfg(test)]
pub(crate) fn generate_n_nested_empty_json_arrays(n: usize) -> String {
    "[".repeat(n) + &"]".repeat(n)
}

#[cfg(test)]
/// Generates a vector of bytes representing `n` nested empty Bencode
/// dictionaries.
///
/// This function is a helper for generating a Bencode representation
/// of a JSON object with `n` nested empty objects. It repeats the Bencode
/// dictionary opener `d` character, the field key `3:foo`, and the Bencode
/// dictionary closer `e` character `n` times to create the desired nested
/// structure.
///
/// # Examples
///
/// ```rust
/// let nested_bencode = generate_n_nested_empty_bencoded_dictionaries(2);
/// assert_eq!(nested_bencode, b"d3:food3:foodeee");
/// ``````
pub(crate) fn generate_n_nested_empty_bencoded_dictionaries(n: usize) -> Vec<u8> {
    if n == 0 {
        return b"de".to_vec();
    }

    let mut dict = vec![b'd']; // Dictionary start
    dict.extend_from_slice(b"3:foo"); // Field key
    dict.extend_from_slice(&generate_n_nested_empty_bencoded_dictionaries(n - 1));
    dict.extend_from_slice(b"e"); // Dictionary end

    dict
}

#[cfg(test)]
/// Generates a JSON object with `n` nested empty objects.
///
/// This function is a helper for generating a JSON object with a
/// specific number of nested empty objects. It repeats the opening `{`
/// character `n` times, and the closing `}` character `n` times, to
/// create the desired nested structure.
///
/// # Examples
///
/// ```rust
/// let nested_json = generate_n_nested_empty_json_objects(2);
/// assert_eq!(nested_json, r#"{"foo":{"foo":{}}}"#.to_string());
/// `
pub(crate) fn generate_n_nested_empty_json_objects(n: usize) -> String {
    if n == 0 {
        return "{}".to_string();
    }

    let mut object = "{".to_string();
    object.push_str(r#""<string>foo</string>":"#);
    object.push_str(&generate_n_nested_empty_json_objects(n - 1));
    object.push('}');

    object
}

#[cfg(test)]
/// Generates a bencoded string with a repeated byte.
///
/// This function creates a bencoded string where the string value consists of a
/// repeated byte.
///
/// # Arguments
///
/// * `byte` - The byte to repeat in the string value.
/// * `n` - The number of times to repeat the byte.
///
/// # Returns
///
/// A `Vec<u8>` containing the bencoded string.
pub(crate) fn bencoded_string_with_repeated_byte(byte: u8, n: usize) -> Vec<u8> {
    let string_length = n.to_string().into_bytes();
    let string_value = vec![byte; n];

    let mut bencoded_string = Vec::new();
    bencoded_string.extend_from_slice(&string_length);
    bencoded_string.push(b':'); // Length/value separator
    bencoded_string.extend_from_slice(&string_value);

    bencoded_string
}
