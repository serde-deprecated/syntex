extern crate syntex_syntax;

mod squash_derive;

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use syntex_syntax::ast;
use syntex_syntax::attr;
use syntex_syntax::codemap::{DUMMY_SP, respan};
use syntex_syntax::ext::base::{
    IdentMacroExpander,
    MultiItemDecorator,
    MultiItemModifier,
    NamedSyntaxExtension,
    SyntaxExtension,
    TTMacroExpander,
};
use syntex_syntax::ext::base::ExtCtxt;
use syntex_syntax::ext::expand;
use syntex_syntax::feature_gate;
use syntex_syntax::parse::{self, token};
use syntex_syntax::print::pprust;
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
        let parse_sess = parse::ParseSess::new();
        let meta_item = parse::parse_meta_from_source_str(
            "cfgspec".to_string(),
            cfg.to_string(),
            Vec::new(),
            &parse_sess).unwrap();

        self.cfg.push(meta_item);
    }

    pub fn add_attr(&mut self, attr: &str) {
        let parse_sess = parse::ParseSess::new();
        let meta_item = parse::parse_meta_from_source_str(
            "attrspec".to_string(),
            attr.to_string(),
            Vec::new(),
            &parse_sess).unwrap();

        self.attrs.push(respan(DUMMY_SP, ast::Attribute_ {
            id: attr::mk_attr_id(),
            style: ast::AttrStyle::Outer,
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
        let sess = parse::ParseSess::new();

        let mut krate = parse::parse_crate_from_file(
            src,
            self.cfg,
            &sess).unwrap();

        krate.attrs.extend(self.attrs);

        let features = feature_gate::get_features(
            &sess.span_diagnostic,
            &krate);

        let krate = self.pre_expansion_passes.iter()
            .fold(krate, |krate, f| (f)(krate));

        let mut ecfg = expand::ExpansionConfig::default(crate_name.to_string());
        ecfg.features = Some(&features);

        let cfg = Vec::new();
        let mut gated_cfgs = Vec::new();
        let ecx = ExtCtxt::new(&sess, cfg, ecfg, &mut gated_cfgs);

        let (krate, _) = expand::expand_crate(ecx, self.macros, self.syntax_exts, krate);
        let krate = squash_derive::squash_derive(krate);

        let krate = self.post_expansion_passes.iter()
            .fold(krate, |krate, f| (f)(krate));

        let src_name = src.to_str().unwrap().to_string();
        let src = sess.codemap()
            .get_filemap(&src_name)
            .unwrap()
            .src
            .as_ref()
            .unwrap()
            .as_bytes()
            .to_vec();
        let mut rdr = &src[..];

        let mut out = Vec::new();
        let annotation = pprust::NoAnn;

        {
            let out: &mut io::Write = &mut out;

            try!(pprust::print_crate(
                sess.codemap(),
                &sess.span_diagnostic,
                &krate,
                src_name.to_string(),
                &mut rdr,
                Box::new(out),
                &annotation,
                false)
            );
        }

        let mut dst = try!(File::create(dst));
        dst.write_all(&out)
    }
}
