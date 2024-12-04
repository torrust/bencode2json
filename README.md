# Bencode2Json

[![Testing](https://github.com/torrust/bencode2json/actions/workflows/testing.yaml/badge.svg)](https://github.com/torrust/bencode2json/actions/workflows/testing.yaml) [![codecov](https://codecov.io/gh/torrust/bencode2json/branch/develop/graph/badge.svg?token=G5IK5HV2EW)](https://codecov.io/gh/torrust/bencode2json)

A lib and console command to convert from bencoded data to JSON format.

Output is similar to: <https://github.com/Chocobo1/bencode_online>. When a bencoded string (byte string) contains only valid UTF-8 chars, those chars will print to the output. If the string contains non valid UTF-8 chars, them the string will be printed in hexadecimal. For example:

Bencoded string (with 2 bytes):

```text
2:\xFF\xFE
```

JSON string:

```text
<hex>fffe</hex>
```

More info: <https://github.com/torrust/teps/discussions/15>

## Console

Run the binary with input and output file:

```console
cargo run -- -i ./tests/fixtures/sample.bencode -o output.json
```

Run the binary with stdin and stdout (UTF-8):

```console
echo "4:spam" | cargo run
"<string>spam</string>"
```

Run the binary with stdin and stdout (non UTF-8):

```console
printf "d3:bar2:\xFF\xFEe" | cargo run
{"<string>bar</string>":"<hex>fffe</hex>"}
```

```console
printf "d2:\xFF\xFE3:bare" | cargo run
{"<hex>fffe</hex>":"<string>bar</string>"}
```

> NOTICE: We need two escape the two bytes `FF` and `FE` with `\x` inside the string.

More examples:

```console
cat ./tests/fixtures/sample.bencode | cargo run
["<string>spam</string>"]
```

More examples with invalid Bencode:

```console
printf "i42" | cargo run
Error: Unexpected end of input parsing integer; read context: input pos 3, latest input bytes dump: [105, 52, 50] (UTF-8 string: `i42`); write context: output pos 2, latest output bytes dump: [52, 50] (UTF-8 string: `42`)
```

```console
printf "3:ab" | cargo run
Error: Unexpected end of input parsing string value; read context: input pos 4, latest input bytes dump: [51, 58, 97, 98] (UTF-8 string: `3:ab`)
```

```console
echo "i00e" | cargo run
Error: Leading zeros in integers are not allowed, for example b'i00e'; read context: byte `48` (char: `0`), input pos 3, latest input bytes dump: [105, 48, 48] (UTF-8 string: `i00`)
```

Generating pretty JSON with [jq][jq]:

```console
echo "d3:foold3:bari42eeee" | cargo run | jq
```

```json
{
  "<string>foo</string>": [
    {
      "<string>bar</string>": 42
    }
  ]
}
```

You can install the binary with:

```console
cargo install bencode2json
```

Or by using [cargo-binstall](https://github.com/cargo-bins/cargo-binstall):

```console
cargo binstall bencode2json
```

## Library

You can install the library with:

```console
cargo add bencode2json
```

There two ways of using the library:

- With high-level wrappers.
- With the low-level generators.

See [examples](./examples/).

## Test

Run unit and integration tests:

```console
cargo test
```

We have included a copy of another C implementation ["be2json.c"](./contrib/be2json.c). You can execute it with the following:

```console
gcc ./contrib/be2json.c -o be2json
chmod +x ./be2json
echo "4:spam" | ./be2json
```

You can generate the coverage report with:

```console
cargo cov
```

## Performance

In terms of memory usage this implementation consumes at least the size of the
biggest bencoded integer or string. The string and integer parsers keeps all the bytes in memory until
it parses the whole value.

The library also wraps the input and output streams in a [BufReader](https://doc.rust-lang.org/std/io/struct.BufReader.html)
 and [BufWriter](https://doc.rust-lang.org/std/io/struct.BufWriter.html) because it can be excessively inefficient to work directly with something that implements [Read](https://doc.rust-lang.org/std/io/trait.Read.html) or [Write](https://doc.rust-lang.org/std/io/trait.Write.html).

## TODO

- [ ] Counter for number of items in a list for debugging and errors.
- [ ] Fuzz testing: Generate random valid bencoded values.
- [ ] Install tracing crate. Add verbose mode that enables debugging.
- [ ] Option to check if the final JSON it's valid at the end of the process.
- [ ] Benchmarking for this implementation and the original C implementation.

## Alternatives

- <https://chocobo1.github.io/bencode_online/>
- <https://adrianstoll.com/post/bencoding/>
- <https://www.nayuki.io/page/bittorrent-bencode-format-tools>
- <https://gist.github.com/camilleoudot/840929699392b3d25afbec25d850c94a>
- <https://github.com/denis-selimovic/bencode>

Bencode online:

- <https://adrianstoll.com/post/bencoding/>
- <https://chocobo1.github.io/bencode_online/>

## Links

Bencode:

- <https://wiki.theory.org/BitTorrentSpecification#Bencoding>
- <https://en.wikipedia.org/wiki/Bencode>

## Credits

This implementation is basically a port to Rust from <https://gist.github.com/camilleoudot/840929699392b3d25afbec25d850c94a> with some changes like:

- It does not use magic numbers (explicit enum for states).
- It prints non UTF-8 string in hexadecimal.

The idea of using hexadecimal format `<hex>ff</hex>` for non UTF-8 string came from the
[bencode online](<https://github.com/Chocobo1/bencode_online>) repo by [@Chocobo1](https://github.com/Chocobo1).

We also want to thank [@da2ce7](https://github.com/da2ce7) for his feedback and review that has improved this project significantly.

## License

**Copyright (c) 2024 The Torrust Developers.**

This program is free software: you can redistribute it and/or modify it under the terms of the [GNU Lesser General Public License][LGPL_3_0] as published by the [Free Software Foundation][FSF], version 3.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the [GNU Lesser General Public License][LGPL_3_0] for more details.

You should have received a copy of the *GNU Lesser General Public License* along with this program. If not, see <https://www.gnu.org/licenses/>.

Some files include explicit copyright notices and/or license notices.

### Legacy Exception

For prosperity, versions of Bencode2Json that are older than five years are automatically granted the [MIT-0][MIT_0] license in addition to the existing [LGPL-3.0-only][LGPL_3_0] license.

[LGPL_3_0]: ./LICENSE
[MIT_0]: ./docs/licenses/LICENSE-MIT_0
[FSF]: https://www.fsf.org/
[jq]: https://jqlang.github.io/jq/
