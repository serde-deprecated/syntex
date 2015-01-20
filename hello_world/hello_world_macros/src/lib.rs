extern crate "syntex_syntax" as syntax;

use syntax::ext::base::TTMacroExpander;

pub fn expand_my_syntax<'cx>(
    cx: &'cx mut syntax::ext::base::ExtCtxt,
    sp: syntax::codemap::Span,
    tts: &[syntax::ast::TokenTree]
) -> Box<syntax::ext::base::MacResult + 'cx> {
    use syntax::ext::build::AstBuilder;

    let expr = cx.expr_u8(sp, 5);

    syntax::ext::base::MacExpr::new(expr)
}
