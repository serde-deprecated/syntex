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
    extensions: HashMap<ast::Name, Rc<SyntaxExtension>>,
    derive_modes: HashMap<ast::Name, Rc<MultiItemModifier>>,
    next_node_id: ast::NodeId,
}

impl<'a> Resolver<'a> {
    pub fn new(session: &'a ParseSess) -> Self {
        Resolver {
            session: session,
            extensions: HashMap::new(),
            derive_modes: HashMap::new(),
            next_node_id: ast::NodeId::new(1),
        }
    }
}

impl<'a> base::Resolver for Resolver<'a> {
    fn next_node_id(&mut self) -> ast::NodeId {
        let id = self.next_node_id;

        match self.next_node_id.as_usize().checked_add(1) {
            Some(next) => {
                self.next_node_id = ast::NodeId::new(next);
            }
            None => panic!("Input too large, ran out of node ids!")
        }

        id
    }
    fn get_module_scope(&mut self, _id: ast::NodeId) -> Mark { Mark::root() }

    fn visit_expansion(&mut self, _invoc: Mark, _expansion: &Expansion) {}
    fn add_macro(&mut self, _scope: Mark, def: ast::MacroDef) {
        self.session.span_diagnostic.span_bug(
            def.span,
            "add_macro is not supported yet");
    }
    fn add_ext(&mut self, ident: ast::Ident, ext: Rc<SyntaxExtension>) {
        self.extensions.insert(ident.name, ext);
    }
    fn add_expansions_at_stmt(&mut self, _id: ast::NodeId, _macros: Vec<Mark>) {}

    fn find_attr_invoc(&mut self, attrs: &mut Vec<Attribute>) -> Option<Attribute> {
        for i in 0..attrs.len() {
            let name = intern(&attrs[i].name());
            match self.extensions.get(&name) {
                Some(ext) => match **ext {
                    MultiModifier(..) | MultiDecorator(..) => return Some(attrs.remove(i)),
                    _ => {}
                },
                None => {}
            }
        }
        None
    }
    fn find_extension(&mut self, _scope: Mark, name: ast::Name) -> Option<Rc<SyntaxExtension>> {
        self.extensions.get(&name).map(|ext| ext.clone())
    }
    fn find_mac(&mut self, scope: Mark, mac: &ast::Mac) -> Option<Rc<SyntaxExtension>> {
        let path = &mac.node.path;
        if path.segments.len() > 1 || path.global ||
           !path.segments[0].parameters.is_empty() {
            // NOTE: Pass macros with module separators through to the generated source.
            self.session.span_diagnostic.span_err(path.span,
                                                  "expected macro name without module separators");
            return None;
        }
        let name = path.segments[0].identifier.name;
        self.find_extension(scope, name)
    }
    fn resolve_invoc(&mut self, scope: Mark, invoc: &Invocation) -> Option<Rc<SyntaxExtension>> {
        let (name, span) = match invoc.kind {
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

        self.find_extension(scope, name).or_else(|| {
            let mut err =
                self.session.span_diagnostic.struct_span_err(span, &format!("macro undefined: '{}!'", name));
            err.emit();
            None
        })
    }
    fn resolve_derive_mode(&mut self, ident: ast::Ident) -> Option<Rc<MultiItemModifier>> {
        self.derive_modes.get(&ident.name).cloned()
    }
}
