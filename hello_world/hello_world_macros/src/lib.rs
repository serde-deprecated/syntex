extern crate syntex;
extern crate syntex_syntax;

use syntex::Registry;

use syntex_syntax::ast::MetaItem;
use syntex_syntax::codemap::Span;
use syntex_syntax::ext::base::{Annotatable, ExtCtxt, MacEager, MacResult};
use syntex_syntax::ext::build::AstBuilder;
use syntex_syntax::parse::token::InternedString;
use syntex_syntax::ptr::P;
use syntex_syntax::tokenstream::TokenTree;

pub fn register(registry: &mut Registry) {
    registry.add_macro("syntex_macro", expand_macro);
    registry.add_modifier("syntex_modifier", expand_modifier);
}

// syntex_macro!()
// returns "hello world"
fn expand_macro<'cx>(
    cx: &'cx mut ExtCtxt,
    sp: Span,
    _: &[TokenTree]
) -> Box<MacResult + 'cx> {
    let expr = cx.expr_str(sp, InternedString::new("hello world"));
    MacEager::expr(expr)
}

// #[syntex_modifier]
// turns into #[derive(Debug)]
fn expand_modifier(cx: &mut ExtCtxt,
                   span: Span,
                   _meta_item: &MetaItem,
                   item: Annotatable) -> Annotatable {
    let item = item.expect_item();
    let mut new_item = (*item).clone();

    let debug = cx.meta_word(span, InternedString::new("Debug"));
    let attr = cx.attribute(span,
                            cx.meta_list(span,
                                         InternedString::new("derive"),
                                         vec![debug]));
    new_item.attrs.push(attr);

    Annotatable::Item(P(new_item))
}
