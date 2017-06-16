Rust syntax backport
====================

[![Build Status](https://api.travis-ci.org/serde-rs/syntex.png?branch=master)](https://travis-ci.org/serde-rs/syntex)
[![Latest Version](https://img.shields.io/crates/v/syntex_syntax.svg)](https://crates.io/crates/syntex_syntax)

This repository contains a backport of the following unstable crates from the
Rust compiler.

- [`libsyntax`] => [`syntex_syntax`]
- [`libsyntax_pos`] => [`syntex_pos`]
- [`librustc_errors`] => [`syntex_errors`]

[`libsyntax`]: https://github.com/rust-lang/rust/tree/master/src/libsyntax
[`syntex_syntax`]: https://docs.rs/syntex_syntax
[`libsyntax_pos`]: https://github.com/rust-lang/rust/tree/master/src/libsyntax_pos
[`syntex_pos`]: https://docs.rs/syntex_pos
[`librustc_errors`]: https://github.com/rust-lang/rust/tree/master/src/librustc_errors
[`syntex_errors`]: https://docs.rs/syntex_errors

The backported code compiles on the most recent stable release of Rust.

## License

Syntex is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Syntex by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
