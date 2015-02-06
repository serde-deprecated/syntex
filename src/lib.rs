#![feature(io)]

extern crate "syntex_syntax" as syntax;

use syntax::ast::Name;
use syntax::ext::base::SyntaxExtension;
use syntax::parse::token;

use std::old_io::{File, IoResult};

pub fn expand_str(
    crate_name: &str,
    body: String,
    syntax_exts: Vec<(&str, SyntaxExtension)>,
) -> String {
    let sess = syntax::parse::new_parse_sess();
    let cfg = vec![];

    let crate_name = crate_name.to_string();

    let krate = syntax::parse::parse_crate_from_source_str(
        crate_name.clone(),
        body,
        cfg,
        &sess);

    let krate = syntax::config::strip_unconfigured_items(&sess.span_diagnostic, krate);

    let cfg = syntax::ext::expand::ExpansionConfig {
        crate_name: crate_name.to_string(),
        enable_quotes: true,
        recursion_limit: 64,
    };

    let macros = vec![];
    let syntax_exts: Vec<(Name, SyntaxExtension)> = syntax_exts.into_iter()
        .map(|(name, ext)| (token::intern(name), ext))
        .collect();

    let krate = syntax::ext::expand::expand_crate(&sess,
                                                  cfg,
                                                  macros,
                                                  syntax_exts,
                                                  krate);

    syntax::print::pprust::to_string(|s| {
        try!(s.print_mod(&krate.module, krate.attrs.as_slice()));
        try!(s.print_remaining_comments());
        syntax::print::pp::eof(&mut s.s)
    })
}

pub fn expand_file(
    src: Path,
    dst: Path,
    crate_name: &str,
    syntax_exts: Vec<(&str, SyntaxExtension)>,
) -> IoResult<()> {
    let mut src = try!(File::open(&src));
    let src = String::from_utf8(try!(src.read_to_end())).unwrap();

    let output = expand_str(crate_name, src, syntax_exts);

    let mut dst = try!(File::create(&dst));
    dst.write_str(&output[])
}
