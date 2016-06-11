## 0.35.0

BACKWARDS INCOMPATIBILITIES / NOTES:

* Update to rustc 1.11.0-nightly (7d2f75a95 2016-06-09).

## 0.34.0

BACKWARDS INCOMPATIBILITIES / NOTES:

* Sync with rustc 1.11.0-nightly (ec872dc8a 2016-06-07).
* Removed `syntex_syntax::owned_slice`
  [#53](https://github.com/serde-rs/syntex/pull/53).
* Restore use of various Rust 1.6-compatible APIs:
  [#54](https://github.com/serde-rs/syntex/pull/54)
  [#55](https://github.com/serde-rs/syntex/pull/55)
  [#56](https://github.com/serde-rs/syntex/pull/56)
  [#57](https://github.com/serde-rs/syntex/pull/57)
* Remove unused `ty_param_to_string`
  [#59](https://github.com/serde-rs/syntex/pull/59)
* Accept AsRef<Path> instead of &Path in Registry::expand
  [#65](https://github.com/serde-rs/syntex/pull/65)

BUG FIXES:

* Fix accidental removal of question-marks
  [#58](https://github.com/serde-rs/syntex/pull/58)
  [#60](https://github.com/serde-rs/syntex/pull/60)

## 0.33.0

BACKWARDS INCOMPATIBILITIES / NOTES:

* Update to rustc 1.10.0-nightly (7bddce693 2016-05-27).
* Support for Rust 1.5.0 is no longer guaranteed.

BUG FIXES:

* Increase pretty printer ring buffer size to pretty print large code outputs
  [#47](https://github.com/serde-rs/syntex/pull/47).
