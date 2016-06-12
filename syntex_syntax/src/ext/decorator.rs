use ast::{self, Name};
use attr::{self, AttrMetaMethods};
use codemap::{ExpnInfo, MacroAttribute, NameAndSpan};
use ext::base::*;
use ext::build::AstBuilder;
use ext::expand::{expand_annotatable, MacroExpander};
use parse::token::intern;
use util::small_vector::SmallVector;

// Responsible for expanding `cfg_attr` and delegating to expand_decorators_2.
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
// In the example, SPEC1 was handled by expand_decorators_2 to create the impl,
// and the handling of SPEC2 resulted in a new attribute SPEC3 which remains
// conditional.
pub fn expand_decorators(a: Annotatable,
                     fld: &mut MacroExpander,
                     decorator_items: &mut SmallVector<Annotatable>,
                     new_attrs: &mut Vec<ast::Attribute>)
{
    for attr in a.attrs() {
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
                expand_decorators_2(&a, &spec, fld, &mut items, &mut attrs);
                for item in items {
                    let item = annotatable_with_attr(item, cond.clone());
                    decorator_items.push(item);
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
                expand_decorators_2(&a, attr, fld, decorator_items, new_attrs);
            }
        }
    }
}

fn annotatable_with_attr(
    a: Annotatable,
    attr: ast::Attribute,
) -> Annotatable {
    fn extend<T>(mut v: Vec<T>, t: T) -> Vec<T> { v.push(t); v }
    match a {
        Annotatable::Item(i) => Annotatable::Item(i.map(|i| {
            ast::Item { attrs: extend(i.attrs, attr), ..i }
        })),
        Annotatable::TraitItem(i) => Annotatable::TraitItem(i.map(|ti| {
            ast::TraitItem { attrs: extend(ti.attrs, attr), ..ti }
        })),
        Annotatable::ImplItem(i) => Annotatable::ImplItem(i.map(|ii| {
            ast::ImplItem { attrs: extend(ii.attrs, attr), ..ii }
        })),
    }
}

// Responsible for expanding `derive` and delegating to expand_decorators_3.
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
// In the example, `derive_Serialize` was handled by expand_decorators_3 to
// create the impl but `derive_Clone` and `other_attr` were not handled.
// Attributes that are not handled by expand_decorators_3 are preserved.
fn expand_decorators_2(
    a: &Annotatable,
    attr: &ast::Attribute,
    fld: &mut MacroExpander,
    decorator_items: &mut SmallVector<Annotatable>,
    new_attrs: &mut Vec<ast::Attribute>,
) {
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
            let handled = expand_decorators_3(a, &derive, fld, decorator_items, tname);
            if !handled {
                not_handled.push((*titem).clone());
            }
        }
        if !not_handled.is_empty() {
            // #[derive(Trait, ...)]
            let derive = fld.cx.attribute(
                attr.span,
                fld.cx.meta_list(mitem.span, mname.as_str(), not_handled));
            new_attrs.push(derive);
        }
    } else {
        let handled = expand_decorators_3(a, attr, fld, decorator_items, mname);
        if !handled {
            new_attrs.push((*attr).clone());
        }
    }
}

// Responsible for expanding attributes that match a MultiDecorator registered
// in the syntax_env. Returns whether the given attribute was handled.
fn expand_decorators_3(
    a: &Annotatable,
    attr: &ast::Attribute,
    fld: &mut MacroExpander,
    decorator_items: &mut SmallVector<Annotatable>,
    mname: Name,
) -> bool {
    match fld.cx.syntax_env.find(mname) {
        Some(rc) => match *rc {
            MultiDecorator(ref dec) => {
                attr::mark_used(&attr);

                fld.cx.bt_push(ExpnInfo {
                    call_site: attr.span,
                    callee: NameAndSpan {
                        format: MacroAttribute(mname),
                        span: Some(attr.span),
                        // attributes can do whatever they like,
                        // for now.
                        allow_internal_unstable: true,
                    }
                });

                // we'd ideally decorator_items.push_all(expand_annotatable(ann, fld)),
                // but that double-mut-borrows fld
                let mut items: SmallVector<Annotatable> = SmallVector::zero();
                dec.expand(fld.cx,
                            attr.span,
                            &attr.node.value,
                            a,
                            &mut |ann| items.push(ann));
                decorator_items.extend(items.into_iter()
                    .flat_map(|ann| expand_annotatable(ann, fld).into_iter()));

                fld.cx.bt_pop();
                true
            }
            _ => false,
        },
        _ => false,
    }
}
