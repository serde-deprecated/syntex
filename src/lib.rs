extern crate syntax;

use syntax::ext::base::TTMacroExpander;

fn expand_my_syntax<'cx>(
    cx: &'cx mut syntax::ext::base::ExtCtxt,
    sp: syntax::codemap::Span,
    tts: &[syntax::ast::TokenTree]
) -> Box<syntax::ext::base::MacResult + 'cx> {
    use syntax::ext::build::AstBuilder;

    let expr = cx.expr_int(sp, 5);

    syntax::ext::base::MacExpr::new(expr)
}

pub fn expand_str(crate_name: &str, body: &str) -> String {
    let sess = syntax::parse::new_parse_sess();
    let cfg = vec![];

    let crate_name = crate_name.to_string();
    let body = body.to_string();

    let krate = syntax::parse::parse_crate_from_source_str(
        crate_name.clone(),
        body,
        cfg,
        &sess);

    let krate = syntax::config::strip_unconfigured_items(&sess.span_diagnostic, krate);

    let cfg = syntax::ext::expand::ExpansionConfig {
        crate_name: crate_name.to_string(),
        deriving_hash_type_parameter: true,
        enable_quotes: true,
        recursion_limit: 64,
    };

    let macros = vec![];
    let syntax_exts = vec![
        (
            syntax::parse::token::intern("my_syntax"),
            syntax::ext::base::SyntaxExtension::NormalTT(box expand_my_syntax, None),
        )
    ];

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
