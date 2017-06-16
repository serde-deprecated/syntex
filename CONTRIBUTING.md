How to do a Syntex release
==========================

First we need to prep the Rust repository. Check out Rust.

```
$ git clone https://github.com/rust-lang/rust
$ cd rust
```

Update the nightly version and determine it's SHA1.

```
rust$ rustup update nightly
rust$ rustc +nightly --version
rustc 1.19.0-nightly (4ed2edaaf 2017-06-01)
rust$ export RUST_SHA=4ed2edaaf
```

Check out that version.

```
rust$ git checkout $RUST_SHA
```

---

Check out Syntex.

```
rust$ cd ..
$ git clone https://github.com/serde-rs/syntex
$ cd syntex
```

Check out the `rust` branch, which tracks the upstream Rust `libsyntax`. Delete
the syntex source directories and replace them with the ones from upstream.

```
syntex$ git checkout origin/rust
syntex$ rm -r syntex_syntax/src syntex_pos/src syntex_errors/src
syntex$ cp -r ../rust/src/libsyntax syntex_syntax/src
syntex$ cp -r ../rust/src/libsyntax_pos syntex_pos/src
syntex$ cp -r ../rust/src/librustc_errors syntex_errors/src
syntex$ git add .
syntex$ git commit -m "Sync with $(rustc +nightly --version)"
syntex$ git push origin HEAD:rust
```

Switch back to the master branch, merge it in, and resolve any conflicts.

```
syntex$ git checkout origin/master
syntex$ git merge origin/rust
# ... conflict resolution :-)
```

Confirm that everything compiles on stable Rust and newer.

```
syntex$ cd syntex_syntax
syntex/syntex_syntax$ cargo +stable build
syntex/syntex_syntax$ cargo +beta build
syntex/syntex_syntax$ cargo +nightly build
```

Once it works locally, push the `master` branch for CI.

```
syntex$ git push origin HEAD:refs/heads/up
```

Resolve any build issues, bump the version number, tag it, and publish.

```
syntex$ GIT_COMMITTER_DATE="$(git show --format=%aD | head -1)" git tag -s -m "Release 0.59.0" v0.59.0
syntex$ git push origin --tags
syntex$ cd syntex_pos
syntex/syntex_pos$ cargo publish
syntex/syntex_pos$ cd ../syntex_errors
syntex/syntex_errors$ cargo publish
syntex/syntex_errors$ cd ../syntex_syntax
syntex/syntex_syntax$ cargo publish
```
