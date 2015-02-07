extern crate syntex;
extern crate syntex_syntax;

use syntex::Registry;

use syntex_syntax::ast::TokenTree;
use syntex_syntax::codemap::Span;
use syntex_syntax::ext::base::{ExtCtxt, MacExpr, MacResult, TTMacroExpander};
use syntex_syntax::ext::build::AstBuilder;
use syntex_syntax::parse::token::InternedString;

pub fn expand_hello_world<'cx>(
    cx: &'cx mut ExtCtxt,
    sp: Span,
    tts: &[TokenTree]
) -> Box<MacResult + 'cx> {
    let expr = cx.expr_str(sp, InternedString::new("hello world"));

    MacExpr::new(expr)
}

pub fn register(registry: &mut Registry) {
    registry.register_fn("hello_world", expand_hello_world);
}
