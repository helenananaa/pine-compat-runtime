use crate::{Diagnostic, Span};

use super::Parser;

impl Parser {
    pub(super) fn reject_reserved_destructuring_binding(&mut self, name: &str, span: Span) {
        // The `in` grammar keyword is rejected even by tuple/for-in name
        // positions that admit the other future reserved names on native Pine.
        if name == "in" {
            self.reject_reserved_binding(name, span);
        }
    }

    /// These words became reserved in v5. Check declaration sites rather than
    /// lexing them as keywords: named arguments and qualified members remain
    /// legal, as do the native tuple/for-in binding exceptions.
    pub(super) fn reject_reserved_binding(&mut self, name: &str, span: Span) {
        if self.source_version >= 5
            && matches!(
                name,
                "catch"
                    | "class"
                    | "do"
                    | "ellipse"
                    | "in"
                    | "is"
                    | "polygon"
                    | "range"
                    | "return"
                    | "struct"
                    | "text"
                    | "throw"
                    | "try"
            )
        {
            self.diagnostics.push(Diagnostic::error(
                "E_RESERVED_IDENTIFIER",
                format!("reserved identifier `{name}` cannot be declared here in Pine v5 or later"),
                span,
            ));
        }
    }
}
