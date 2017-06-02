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
