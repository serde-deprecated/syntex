use ast::{self, Name};
use attr::{self, AttrMetaMethods, HasAttrs};
use codemap::{ExpnInfo, MacroAttribute, NameAndSpan};
use ext::base::*;
use ext::build::AstBuilder;
use ext::expand::{MacroExpander, expand_multi_modified};
use parse::token::intern;
use util::small_vector::SmallVector;

pub fn expand_annotatable(
    mut item: Annotatable,
    fld: &mut MacroExpander,
) -> SmallVector<Annotatable> {
    let mut out_items = SmallVector::zero();
    let mut new_attrs = Vec::new();

    item = expand_1(item, fld, &mut out_items, &mut new_attrs);

    item = item.fold_attrs(new_attrs);
    let expanded = expand_multi_modified(item, fld);
    out_items.push_all(expanded);

    out_items
}

// Responsible for expanding `cfg_attr` and delegating to expand_2.
//
// The expansion turns this:
//
//     #[cfg_attr(COND1, SPEC1)]
//     #[cfg_attr(COND2, SPEC2)]
//     struct Item { ... }
//
// into this:
//
//     #[cfg(COND1)]
//     impl Trait for Item { ... }
//     #[cfg_attr(COND2, SPEC3)]
//     struct Item { ... }
//
// In the example, SPEC1 was handled by expand_2 to create the impl, and the
// handling of SPEC2 resulted in a new attribute SPEC3 which remains
// conditional.
fn expand_1(
    mut item: Annotatable,
    fld: &mut MacroExpander,
    out_items: &mut SmallVector<Annotatable>,
    new_attrs: &mut Vec<ast::Attribute>,
) -> Annotatable {
    while !item.attrs().is_empty() {
        // Pop the first attribute.
        let mut attr = None;
        item = item.map_attrs(|mut attrs| {
            attr = Some(attrs.remove(0));
            attrs
        });
        let attr = attr.unwrap();

        match attr.node.value.node {
            // #[cfg_attr(COND, SPEC)]
            ast::MetaItemKind::List(ref word, ref vec)
                if word == "cfg_attr" && vec.len() == 2 =>
            {
                // #[cfg(COND)]
                let cond = fld.cx.attribute(
                    attr.span,
                    fld.cx.meta_list(
                        attr.node.value.span,
                        intern("cfg").as_str(),
                        vec[..1].to_vec()));
                // #[SPEC]
                let spec = fld.cx.attribute(
                    attr.span,
                    vec[1].clone());
                let mut items = SmallVector::zero();
                let mut attrs = Vec::new();
                item = expand_2(item, &spec, fld, &mut items, &mut attrs);
                for new_item in items {
                    let new_item = new_item.map_attrs(|mut attrs| {
                        attrs.push(cond.clone());
                        attrs
                    });
                    out_items.push(new_item);
                }
                for new_attr in attrs {
                    // #[cfg_attr(COND, NEW_SPEC)]
                    let new_attr = fld.cx.attribute(
                        attr.span,
                        fld.cx.meta_list(
                            attr.node.value.span,
                            word.clone(),
                            vec![vec[0].clone(), new_attr.node.value]));
                    new_attrs.push(new_attr);
                }
            }
            _ => {
                item = expand_2(item, &attr, fld, out_items, new_attrs);
            }
        }
    }
    item
}

// Responsible for expanding `derive` and delegating to expand_3.
//
// The expansion turns this:
//
//     #[derive(Serialize, Clone)]
//     #[other_attr]
//     struct Item { ... }
//
// into this:
//
//     impl Serialize for Item { ... }
//     #[derive(Clone)]
//     #[other_attr]
//     struct Item { ... }
//
// In the example, `derive_Serialize` was handled by expand_3 to create the impl
// but `derive_Clone` and `other_attr` were not handled. Attributes that are not
// handled by expand_3 are preserved.
fn expand_2(
    mut item: Annotatable,
    attr: &ast::Attribute,
    fld: &mut MacroExpander,
    out_items: &mut SmallVector<Annotatable>,
    new_attrs: &mut Vec<ast::Attribute>,
) -> Annotatable {
    let mname = intern(&attr.name());
    let mitem = &attr.node.value;
    if mname.as_str() == "derive" {
        let traits = mitem.meta_item_list().unwrap_or(&[]);
        if traits.is_empty() {
            fld.cx.span_warn(mitem.span, "empty trait list in `derive`");
        }
        let mut not_handled = Vec::new();
        for titem in traits.iter().rev() {
            let tname = match titem.node {
                ast::MetaItemKind::Word(ref tname) => tname,
                _ => {
                    fld.cx.span_err(titem.span, "malformed `derive` entry");
                    continue;
                }
            };
            let tname = intern(&format!("derive_{}", tname));
            // #[derive_Trait]
            let derive = fld.cx.attribute(
                attr.span,
                fld.cx.meta_word(titem.span, tname.as_str()));
            let (handled, out) = expand_3(item, &derive, fld, out_items, tname);
            if !handled {
                not_handled.push((*titem).clone());
            }
            item = out;
        }
        if !not_handled.is_empty() {
            // #[derive(Trait, ...)]
            let derive = fld.cx.attribute(
                attr.span,
                fld.cx.meta_list(mitem.span, mname.as_str(), not_handled));
            new_attrs.push(derive);
        }
        item
    } else {
        let (handled, out) = expand_3(item, attr, fld, out_items, mname);
        if !handled {
            new_attrs.push((*attr).clone());
        }
        out
    }
}

// Responsible for expanding attributes that match a MultiDecorator or
// MultiModifier registered in the syntax_env. Returns whether the given
// attribute was handled, along with the item to continue processing.
//
// Syntex supports only a special case of MultiModifier - those that produce
// exactly one output. If a MultiModifier produces zero or more than one output
// this function panics. The problematic case we cannot support is:
//
//     #[decorator] // not registered with Syntex
//     #[modifier] // registered
//     struct A;
fn expand_3(
    item: Annotatable,
    attr: &ast::Attribute,
    fld: &mut MacroExpander,
    out_items: &mut SmallVector<Annotatable>,
    mname: Name,
) -> (bool, Annotatable) {
    match fld.cx.syntax_env.find(mname) {
        Some(rc) => match *rc {
            MultiDecorator(ref mac) => {
                attr::mark_used(&attr);
                fld.cx.bt_push(ExpnInfo {
                    call_site: attr.span,
                    callee: NameAndSpan {
                        format: MacroAttribute(mname),
                        span: Some(attr.span),
                        // attributes can do whatever they like, for now.
                        allow_internal_unstable: true,
                    }
                });

                let mut modified = Vec::new();
                mac.expand(fld.cx, attr.span, &attr.node.value, &item,
                        &mut |item| modified.push(item));

                fld.cx.bt_pop();
                out_items.extend(modified.into_iter()
                    .flat_map(|ann| expand_annotatable(ann, fld).into_iter()));
                (true, item)
            }
            MultiModifier(ref mac) => {
                attr::mark_used(&attr);
                fld.cx.bt_push(ExpnInfo {
                    call_site: attr.span,
                    callee: NameAndSpan {
                        format: MacroAttribute(mname),
                        span: Some(attr.span),
                        // attributes can do whatever they like, for now.
                        allow_internal_unstable: true,
                    }
                });

                let mut modified = mac.expand(fld.cx,
                                              attr.span,
                                              &attr.node.value,
                                              item);
                if modified.len() != 1 {
                    panic!("expected 1 output from `#[{}]` but got {}",
                           mname, modified.len());
                }
                let modified = modified.pop().unwrap();

                fld.cx.bt_pop();

                let mut expanded = expand_annotatable(modified, fld);
                if expanded.is_empty() {
                    panic!("expected 1 output from `#[{}]` but got {}",
                           mname, expanded.len());
                }
                let last = expanded.pop().unwrap();

                out_items.extend(expanded);
                (true, last)
            }
            _ => (false, item),
        },
        _ => (false, item),
    }
}
