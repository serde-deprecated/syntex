use std::collections::HashMap;
use std::rc::Rc;

use syntex_syntax::ast::{self, Attribute};
use syntex_syntax::ext::base::{
    self,
    MultiDecorator,
    MultiModifier,
    MultiItemModifier,
    SyntaxExtension,
};
use syntex_syntax::ext::expand::{Invocation, InvocationKind, Expansion};
use syntex_syntax::ext::hygiene::Mark;
use syntex_syntax::parse::token::intern;
use syntex_syntax::parse::ParseSess;

pub struct Resolver<'a> {
    session: &'a ParseSess,
    macros: HashMap<ast::Name, Rc<SyntaxExtension>>,
}

impl<'a> Resolver<'a> {
    pub fn new(session: &'a ParseSess) -> Self {
        Resolver {
            session: session,
            macros: HashMap::new(),
        }
    }
}

impl<'a> base::Resolver for Resolver<'a> {
    fn next_node_id(&mut self) -> ast::NodeId { ast::DUMMY_NODE_ID }
    fn get_module_scope(&mut self, _id: ast::NodeId) -> Mark { Mark::root() }

    fn visit_expansion(&mut self, _invoc: Mark, _expansion: &Expansion) {}
    fn add_macro(&mut self, _scope: Mark, _def: ast::MacroDef) {}
    fn add_ext(&mut self, _ident: ast::Ident, _ext: Rc<SyntaxExtension>) {}
    fn add_expansions_at_stmt(&mut self, _id: ast::NodeId, _macros: Vec<Mark>) {}

    fn find_attr_invoc(&mut self, attrs: &mut Vec<Attribute>) -> Option<Attribute> {
        for i in 0..attrs.len() {
            let name = intern(&attrs[i].name());
            match self.macros.get(&name) {
                Some(ext) => match **ext {
                    MultiModifier(..) | MultiDecorator(..) => return Some(attrs.remove(i)),
                    _ => {}
                },
                None => {}
            }
        }
        None
    }
    fn resolve_invoc(&mut self, _scope: Mark, invoc: &Invocation) -> Option<Rc<SyntaxExtension>> {
        let (name, _span) = match invoc.kind {
            InvocationKind::Bang { ref mac, .. } => {
                let path = &mac.node.path;
                if path.segments.len() > 1 || path.global ||
                   !path.segments[0].parameters.is_empty() {
                    // NOTE: Pass macros with module separators through to the generated source.
                    self.session.span_diagnostic.span_err(path.span,
                                                          "expected macro name without module separators");
                    return None;
                }
                (path.segments[0].identifier.name, path.span)
            }
            InvocationKind::Attr { ref attr, .. } => (intern(&*attr.name()), attr.span),
        };

        if let Some(ext) = self.macros.get(&name) {
            return Some(ext.clone());
        }

        // NOTE: Pass undefined macros through to the generated source.
        /*
        let mut err =
             self.session.span_diagnostic.struct_span_err(span, &format!("macro undefined: '{}!'", name));
        // self.suggest_macro_name(&name.as_str(), &mut err);
        err.emit();
        */

        None
    }
    fn resolve_derive_mode(&mut self, _ident: ast::Ident) -> Option<Rc<MultiItemModifier>> {
        None
    }
}
