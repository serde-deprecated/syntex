extern crate syntax;
use std::io;

#[test]
fn test() {
    let sess = syntax::parse::new_parse_sess();
    let cfg = vec![];

    let krate = syntax::parse::parse_crate_from_source_str(
        "main".to_string(),
        "fn main() {}".to_string(),
        cfg,
        &sess);

    let printer = syntax::print::pprust::rust_printer(box io::stdout());

    let s = syntax::print::pprust::to_string(|s| {
        try!(s.print_mod(&krate.module, krate.attrs.as_slice()));
        try!(s.print_remaining_comments());
        syntax::print::pp::eof(&mut s.s)
    });

    println!("{}", s);
}
