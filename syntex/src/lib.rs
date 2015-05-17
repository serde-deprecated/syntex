extern crate syntex_syntax;

use std::fs::File;
use std::io;
use std::path::Path;

use syntex_syntax::ast;
use syntex_syntax::attr;
use syntex_syntax::codemap::{DUMMY_SP, respan};
use syntex_syntax::config;
use syntex_syntax::ext::base::{
    IdentMacroExpander,
    MultiItemDecorator,
    MultiItemModifier,
    NamedSyntaxExtension,
    SyntaxExtension,
    TTMacroExpander,
};
use syntex_syntax::ext::expand;
use syntex_syntax::feature_gate;
use syntex_syntax::parse::{self, token};
use syntex_syntax::print::{pp, pprust};
use syntex_syntax::ptr::P;

pub type Pass = fn(ast::Crate) -> ast::Crate;

pub struct Registry {
    macros: Vec<ast::MacroDef>,
    syntax_exts: Vec<NamedSyntaxExtension>,
    pre_expansion_passes: Vec<Box<Pass>>,
    post_expansion_passes: Vec<Box<Pass>>,
    cfg: Vec<P<ast::MetaItem>>,
    attrs: Vec<ast::Attribute>,
}

impl Registry {
    pub fn new() -> Registry {
        Registry {
            macros: Vec::new(),
            syntax_exts: Vec::new(),
            pre_expansion_passes: Vec::new(),
            post_expansion_passes: Vec::new(),
            cfg: Vec::new(),
            attrs: Vec::new(),
        }
    }

    pub fn add_cfg(&mut self, cfg: &str) {
        let meta_item = parse::parse_meta_from_source_str(
            "cfgspec".to_string(),
            cfg.to_string(),
            Vec::new(),
            &parse::new_parse_sess());

        self.cfg.push(meta_item);
    }

    pub fn add_attr(&mut self, attr: &str) {
        let meta_item = parse::parse_meta_from_source_str(
            "attrspec".to_string(),
            attr.to_string(),
            Vec::new(),
            &parse::new_parse_sess());

        self.attrs.push(respan(DUMMY_SP, ast::Attribute_ {
            id: attr::mk_attr_id(),
            style: ast::AttrOuter,
            value: meta_item,
            is_sugared_doc: false,
        }));
    }

    pub fn add_macro<F>(&mut self, name: &str, extension: F)
        where F: TTMacroExpander + 'static
    {
        let name = token::intern(name);
        let syntax_extension = SyntaxExtension::NormalTT(
            Box::new(extension),
            None,
            false
        );
        self.syntax_exts.push((name, syntax_extension));
    }

    pub fn add_ident_macro<F>(&mut self, name: &str, extension: F)
        where F: IdentMacroExpander + 'static
    {
        let name = token::intern(name);
        let syntax_extension = SyntaxExtension::IdentTT(
            Box::new(extension),
            None,
            false
        );
        self.syntax_exts.push((name, syntax_extension));
    }

    pub fn add_decorator<F>(&mut self, name: &str, extension: F)
        where F: MultiItemDecorator + 'static
    {
        let name = token::intern(name);
        let syntax_extension = SyntaxExtension::MultiDecorator(Box::new(extension));
        self.syntax_exts.push((name, syntax_extension));
    }

    pub fn add_modifier<F>(&mut self, name: &str, extension: F)
        where F: MultiItemModifier + 'static
    {
        let name = token::intern(name);
        let syntax_extension = SyntaxExtension::MultiModifier(Box::new(extension));
        self.syntax_exts.push((name, syntax_extension));
    }

    pub fn add_pre_expansion_pass(&mut self, pass: Pass) {
        self.pre_expansion_passes.push(Box::new(pass))
    }

    pub fn add_post_expansion_pass(&mut self, pass: Pass) {
        self.post_expansion_passes.push(Box::new(pass))
    }


    pub fn expand(self, crate_name: &str, src: &Path, dst: &Path) -> io::Result<()> {
        let sess = parse::new_parse_sess();

        let mut krate = parse::parse_crate_from_file(
            src,
            self.cfg,
            &sess);

        krate.attrs.extend(self.attrs);

        let krate = config::strip_unconfigured_items(
            &sess.span_diagnostic,
            krate);

        let features = feature_gate::check_crate_macros(
            &sess.span_diagnostic.cm,
            &sess.span_diagnostic,
            &krate);

        let krate = self.pre_expansion_passes.iter()
            .fold(krate, |krate, f| (f)(krate));

        let mut ecfg = expand::ExpansionConfig::default(crate_name.to_string());
        ecfg.features = Some(&features);

        let krate = expand::expand_crate(
            &sess,
            ecfg,
            self.macros,
            self.syntax_exts,
            krate);

        let krate = self.post_expansion_passes.iter()
            .fold(krate, |krate, f| (f)(krate));

        let dst = try!(File::create(dst));

        let mut printer = pprust::rust_printer(Box::new(dst));
        try!(printer.print_mod(&krate.module, &krate.attrs[..]));
        try!(printer.print_remaining_comments());
        pp::eof(&mut printer.s)
    }
}
