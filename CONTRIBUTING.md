How to do a syntex\_syntax Release
==================================

First, we need to prep the Rust repository. Check out Rust:

```
% git clone https://github.com/rust-lang/rust
% cd rust
```

Next, update the nightly version, and determine it's SHA1.
This is simple with [multirust](https://github.com/brson/multirust):

```
% multirust update
% multirust run nightly rustc --version
rustc 1.5.0-nightly (f93ab64d4 2015-09-21)
```

Finally, checkout that version:

```
% git checkout rust
```

---

Now that Rust is ready, get Syntex ready. First, check out Syntex:

```
% cd ..
% git clone https://github.com/serde-rs/syntex
% cd syntex
```

Check out the `rust` branch, which tracks the upstream Rust `libsyntax`. Delete
the `syntex_syntax/src` and replace it with the rust `libsyntax`.

```
% git checkout rust
% rm -r syntex_syntax/src
% cp ../rust/src/libsyntax syntex_syntax/src
% git commit -a -m "Sync with rust HEAD ($SHA)"
```

Switch back to the master branch, merge it in, and resolve any conflicts:

```
% git checkout master
% git merge rust
# ... conflict resolution
```

At this point, syntex will probably compile on Nightly, but not necessarily on
Stable. Check this by building the examples:

```
% cd hello_world
% multirust run stable cargo run
% multirust run beta cargo run
% multirust run nightly cargo run
```

Resolve any errors that come up. Once that's good, bump the version number, and
push up the `rust` and `master` branches for review. Once it lands and passes
the travis builds, tag it, then publish it:

```
% git tag -s -m "Tagging for release" v0.14.0
% git push origin v0.14.0
% cd syntex_syntax
% cargo publish
```
