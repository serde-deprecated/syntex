// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The compiler code necessary to implement the `#[derive]` extensions.
//!
//! FIXME (#2810): hygiene. Search for "__" strings (in other files too). We also assume "extra" is
//! the standard library, and "std" is the core library.

use ast::{MetaItem, MetaWord};
use attr::AttrMetaMethods;
use ext::base::{ExtCtxt, SyntaxEnv, MultiModifier, Annotatable};
use ext::build::AstBuilder;
use feature_gate;
use codemap::Span;
use parse::token::{intern, intern_and_get_ident};

fn expand_derive(cx: &mut ExtCtxt,
                 span: Span,
                 mitem: &MetaItem,
                 annotatable: Annotatable)
                 -> Annotatable {
    annotatable.map_item_or(|item| {
        item.map(|mut item| {
            if mitem.value_str().is_some() {
                cx.span_err(mitem.span, "unexpected value in `derive`");
            }

            let traits = mitem.meta_item_list().unwrap_or(&[]);
            if traits.is_empty() {
                cx.span_warn(mitem.span, "empty trait list in `derive`");
            }

            for titem in traits.iter().rev() {
                let tname = match titem.node {
                    MetaWord(ref tname) => tname,
                    _ => {
                        cx.span_err(titem.span, "malformed `derive` entry");
                        continue;
                    }
                };

                if !(is_builtin_trait(tname) || cx.ecfg.enable_custom_derive()) {
                    feature_gate::emit_feature_err(&cx.parse_sess.span_diagnostic,
                                                   "custom_derive",
                                                   titem.span,
                                                   feature_gate::GateIssue::Language,
                                                   feature_gate::EXPLAIN_CUSTOM_DERIVE);
                    continue;
                }

                // #[derive(Foo, Bar)] expands to #[derive_Foo] #[derive_Bar]
                item.attrs.push(cx.attribute(titem.span, cx.meta_word(titem.span,
                    intern_and_get_ident(&format!("derive_{}", tname)))));
            }

            item
        })
    }, |a| {
        cx.span_err(span, "`derive` can only be applied to items");
        a
    })
}

pub fn register_all(env: &mut SyntaxEnv) {
    env.insert(intern("derive"),
               MultiModifier(Box::new(expand_derive)));
}

fn is_builtin_trait(name: &str) -> bool {
    match name {
        "Clone" => true,

        "Hash" => true,

        "RustcEncodable" => true,

        "RustcDecodable" => true,

        "PartialEq" => true,
        "Eq" => true,
        "PartialOrd" => true,
        "Ord" => true,

        "Debug" => true,

        "Default" => true,

        "FromPrimitive" => true,

        "Send" => true,
        "Sync" => true,
        "Copy" => true,

        // deprecated
        "Encodable" => true,
        "Decodable" => true,

        _ => false,
    }
}
