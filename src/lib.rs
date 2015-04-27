extern crate syntex_syntax;

use std::fs::File;
use std::io;
use std::path::Path;

use syntex_syntax::ast::{MacroDef, Name};
use syntex_syntax::config;
use syntex_syntax::ext::base::{SyntaxExtension, TTMacroExpander};
use syntex_syntax::ext::expand;
use syntex_syntax::parse::{self, token};
use syntex_syntax::print::{pp, pprust};

pub struct Registry {
    macros: Vec<MacroDef>,
    syntax_exts: Vec<(Name, SyntaxExtension)>,
}

impl Registry {
    pub fn new() -> Registry {
        Registry {
            macros: Vec::new(),
            syntax_exts: Vec::new(),
        }
    }

    pub fn with_standard_macros() -> Registry {
        let registry = Registry::new();
        registry
    }

    pub fn register_fn<F>(&mut self, name: &str, extension: F)
        where F: TTMacroExpander + 'static
    {
        let name = token::intern(name);
        let syntax_extension = SyntaxExtension::NormalTT(Box::new(extension), None, false);
        self.syntax_exts.push((name, syntax_extension));
    }

    pub fn expand(self, crate_name: &str, src: &Path, dst: &Path) -> io::Result<()> {
        let sess = parse::new_parse_sess();
        let cfg = vec![];

        let krate = parse::parse_crate_from_file(
            src,
            cfg,
            &sess);

        let krate = config::strip_unconfigured_items(
            &sess.span_diagnostic,
            krate);

        let cfg = expand::ExpansionConfig::default(crate_name.to_string());

        let krate = expand::expand_crate(
            &sess,
            cfg,
            self.macros,
            self.syntax_exts,
            krate);

        let dst = try!(File::create(dst));

        let mut printer = pprust::rust_printer(Box::new(dst));
        try!(printer.print_mod(&krate.module, &krate.attrs[..]));
        try!(printer.print_remaining_comments());
        pp::eof(&mut printer.s)
    }
}
