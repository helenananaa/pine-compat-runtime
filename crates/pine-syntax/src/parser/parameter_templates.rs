use crate::{FunctionParam, TokenKind};

use super::Parser;

impl Parser {
    /// Collection templates have one element type, or a key/value pair for maps.
    /// Only consume this production after its complete parameter head matches.
    pub(super) fn collection_parameter_head(&mut self) -> Option<FunctionParam> {
        let start_pos = self.pos;
        let start = self.current().span;
        let result = self.parse_collection_parameter_head();
        if result.is_none() {
            self.pos = start_pos;
        }
        result.map(|(type_name, name)| FunctionParam {
            type_name: Some(type_name),
            name,
            span: start.merge(self.previous().span),
            default_value: None,
        })
    }

    fn parse_collection_parameter_head(&mut self) -> Option<(String, String)> {
        let TokenKind::Identifier(container) = self.current().kind.clone() else {
            return None;
        };
        if !matches!(container.as_str(), "array" | "matrix" | "map")
            || !self.nth_at(1, TokenKind::Lt)
        {
            return None;
        }
        self.bump();
        self.bump();
        let mut types = vec![self.parameter_template_name()?];
        if container == "map" {
            if !self.at(TokenKind::Comma) {
                return None;
            }
            self.bump();
            types.push(self.parameter_template_name()?);
        }
        if !self.at(TokenKind::Gt) {
            return None;
        }
        self.bump();
        let TokenKind::Identifier(name) = self.current().kind.clone() else {
            return None;
        };
        self.bump();
        Some((format!("{container}<{}>", types.join(",")), name))
    }

    fn parameter_template_name(&mut self) -> Option<String> {
        let TokenKind::Identifier(mut name) = self.current().kind.clone() else {
            return None;
        };
        self.bump();
        if self.at(TokenKind::Dot) {
            self.bump();
            let TokenKind::Identifier(member) = self.current().kind.clone() else {
                return None;
            };
            self.bump();
            name.push('.');
            name.push_str(&member);
        }
        Some(name)
    }
}
