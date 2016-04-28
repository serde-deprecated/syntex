/// This crate exposes a simple folder that squashes:
///
/// ```rust,ignore
/// #[derive_Foo]
/// #[derive_Bar]
/// struct Baz;
/// ```
///
/// Into:
///
/// ```rust,ignore
/// #[derive(Foo, Bar)]
/// struct Baz;
/// ```

use syntex_syntax::ast;
use syntex_syntax::codemap::{Span, Spanned};
use syntex_syntax::fold::{self, Folder};
use syntex_syntax::parse::token;
use syntex_syntax::ptr::P;
use syntex_syntax::util::move_map::MoveMap;

/// Squash all the `#[derive_*]` into `#[derive(*)]` together.
pub fn squash_derive(krate: ast::Crate) -> ast::Crate {
    SquashDeriveFolder.fold_crate(krate)
}

struct SquashDeriveFolder;

impl Folder for SquashDeriveFolder {
    fn fold_item_simple(&mut self, mut item: ast::Item) -> ast::Item {
        let mut attr_folder = SquashDeriveAttrFolder { derive_attr: None };
        item.attrs = item.attrs.move_flat_map(|x| attr_folder.fold_attribute(x));

        if let Some(derive_attr) = attr_folder.into_attr() {
            item.attrs.push(derive_attr);
        }

        fold::noop_fold_item_simple(item, self)
    }

    fn fold_mac(&mut self, mac: ast::Mac) -> ast::Mac {
        fold::noop_fold_mac(mac, self)
    }
}

struct SquashDeriveAttrFolder {
    derive_attr: Option<DeriveAttr>,
}

impl SquashDeriveAttrFolder {
    fn into_attr(self) -> Option<ast::Attribute> {
        match self.derive_attr {
            Some(derive_attr) => {
                let meta_item = ast::MetaItemKind::List(
                    token::intern_and_get_ident("derive"),
                    derive_attr.meta_items,
                );

                let meta_item = Spanned {
                    node: meta_item,
                    span: derive_attr.span,
                };

                Some(Spanned {
                    node: ast::Attribute_ {
                        id: derive_attr.id,
                        style: derive_attr.style,
                        value: P(meta_item),
                        is_sugared_doc: derive_attr.is_sugared_doc,
                    },
                    span: derive_attr.span,
                })
            }
            None => None,
        }
    }
}

struct DeriveAttr {
    id: ast::AttrId,
    style: ast::AttrStyle,
    is_sugared_doc: bool,
    meta_items: Vec<P<ast::MetaItem>>,
    span: Span,
}

impl Folder for SquashDeriveAttrFolder {
    fn fold_attribute(&mut self,
                      Spanned {
                          node: ast::Attribute_ { id, style, value, is_sugared_doc },
                          span,
                      }: ast::Attribute) -> Option<ast::Attribute> {
        match value.node {
            ast::MetaItemKind::Word(ref name) if name.starts_with("derive_") => {
                let (_, derive_name) = name.split_at("derive_".len());
                let derive_name = token::intern_and_get_ident(derive_name);

                let meta_word = P(Spanned {
                    node: ast::MetaItemKind::Word(derive_name),
                    span: value.span,
                });

                match self.derive_attr {
                    Some(ref mut derive_attr) => {
                        derive_attr.meta_items.push(meta_word);
                    }
                    None => {
                        self.derive_attr = Some(DeriveAttr {
                            id: id,
                            style: style,
                            is_sugared_doc: is_sugared_doc,
                            meta_items: vec![meta_word],
                            span: value.span,
                        });
                    }
                }

                return None;
            }
            _ => { }
        }

        Some(Spanned {
            node: ast::Attribute_ {
                id: id,
                style: style,
                value: value,
                is_sugared_doc: is_sugared_doc,
            },
            span: span,
        })
    }
}

#[cfg(test)]
mod tests {
    use syntex_syntax::ast;
    use syntex_syntax::codemap::{DUMMY_SP, Spanned};
    use syntex_syntax::fold::Folder;
    use syntex_syntax::parse::token;
    use syntex_syntax::ptr::P;

    fn mk_meta_word(name: &str) -> P<ast::MetaItem> {
        let name = token::intern_and_get_ident(name);

        P(Spanned {
            node: ast::MetaItemKind::Word(name),
            span: DUMMY_SP,
        })
    }

    fn mk_meta_list(name: &str,
                    meta_items: Vec<P<ast::MetaItem>>) -> P<ast::MetaItem> {
        let name = token::intern_and_get_ident(name);

        P(Spanned {
            node: ast::MetaItemKind::List(name, meta_items),
            span: DUMMY_SP,
        })
    }

    fn mk_attr(meta_item: P<ast::MetaItem>) -> ast::Attribute {
        Spanned {
            node: ast::Attribute_ {
                id: ast::AttrId(0),
                style: ast::AttrStyle::Outer,
                value: meta_item,
                is_sugared_doc: false,
            },
            span: DUMMY_SP,
        }
    }

    #[test]
    fn test_squash() {
        let variant_data = ast::VariantData::Unit(ast::DUMMY_NODE_ID);

        let generics = ast::Generics {
            lifetimes: vec![],
            ty_params: P::new(),
            where_clause: ast::WhereClause {
                id: ast::DUMMY_NODE_ID,
                predicates: vec![],
            },
        };

        let item_kind = ast::ItemKind::Struct(variant_data, generics);

        let item = ast::Item {
            id: ast::DUMMY_NODE_ID,
            ident: token::str_to_ident("Foo"),
            attrs: vec![
                mk_attr(mk_meta_word("derive_A")),
                mk_attr(mk_meta_word("derive_B")),
            ],
            node: item_kind.clone(),
            vis: ast::Visibility::Inherited,
            span: DUMMY_SP,
        };

        assert_eq!(
            super::SquashDeriveFolder.fold_item_simple(item.clone()),
            ast::Item {
                id: ast::DUMMY_NODE_ID,
                ident: token::str_to_ident("Foo"),
                attrs: vec![
                    mk_attr(mk_meta_list(
                        "derive",
                        vec![mk_meta_word("A"), mk_meta_word("B")],
                    )),
                ],
                node: item_kind,
                vis: ast::Visibility::Inherited,
                span: DUMMY_SP,
            }
        );
    }
}
