extern crate syntex;
extern crate "syntex_syntax" as syntax;
extern crate hello_world_macros;

use syntax::parse::token;
use syntax::ext::base::SyntaxExtension;
use std::os;

fn main() {
    let mut registry = syntex::Registry::new();
    hello_world_macros::register(&mut registry);

    registry.expand(
        "hello_world",
        &Path::new("src/main.rss"),
        &Path::new(os::getenv("OUT_DIR").unwrap()).join("main.rs"));
}
