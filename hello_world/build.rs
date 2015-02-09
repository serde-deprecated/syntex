#![feature(env, path)]

extern crate syntex;
extern crate hello_world_macros;

use std::env;

fn main() {
    let mut registry = syntex::Registry::new();
    hello_world_macros::register(&mut registry);

    let src = Path::new("src/main.rss");
    let dst = Path::new(env::var_string("OUT_DIR").unwrap()).join("main.rs");

    registry.expand("hello_world", &src, &dst).unwrap();
}
