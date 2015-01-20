extern crate syntex;
extern crate "syntex_syntax" as syntax;
extern crate hello_world_macros;

use syntax::parse::token;
use syntax::ext::base::SyntaxExtension;
use std::os;

fn main() {
    syntex::expand_file(
        Path::new("src/hello_world.rs.syntex"),
        Path::new(os::getenv("OUT_DIR").unwrap()).join("hello_world.rs"),
        "hello_world",
        vec![
            (
                "my_syntax",
                SyntaxExtension::NormalTT(Box::new(hello_world_macros::expand_my_syntax), None),
            )
        ]).unwrap();
}
