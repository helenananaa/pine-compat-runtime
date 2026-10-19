use crate::prelude::*;

impl Analyzer {
    pub(crate) fn is_legacy_direction_kind(&self, kind: ValueKind) -> bool {
        self.legacy.dialect().version() <= 3 && kind == ValueKind::StrategyDirection
    }

    pub(crate) fn legacy_direction_bool_arg(
        &mut self,
        actual: PineType,
        accepts: Accepts,
        span: Span,
    ) -> bool {
        if !self.is_legacy_direction_kind(actual.kind)
            || !matches!(
                accepts,
                Accepts::BoolCompatible | Accepts::SimpleBoolCompatible
            )
            || !accepts_type(accepts, PineType::new(actual.qualifier, ValueKind::Bool))
        {
            return false;
        }
        if self
            .legacy_numeric_to_bool_exprs
            .insert(self.expr_key(span))
        {
            let version = self.legacy.dialect().version();
            self.compatibility.legacy_emulations.push(crate::compatibility::LegacyEmulation {
                feature:format!("v{version}.direction_boolean_context"),
                behavior:"Both legacy strategy direction tokens are truthy in boolean contexts; their distinct identities remain unchanged outside conversion".to_owned(),
                span,
            });
        }
        true
    }

    pub(super) fn infer_legacy_direction_binary(
        &mut self,
        op: BinaryOp,
        left: (PineType, Span),
        right: (PineType, Span),
    ) -> Option<PineType> {
        if !self.is_legacy_direction_kind(left.0.kind)
            && !self.is_legacy_direction_kind(right.0.kind)
        {
            return None;
        }
        let kinds = [left.0.kind, right.0.kind];
        let equality = matches!(op, BinaryOp::Eq | BinaryOp::NotEq)
            && kinds
                .iter()
                .all(|k| matches!(k, ValueKind::StrategyDirection | ValueKind::Bool));
        let logical = matches!(op, BinaryOp::And | BinaryOp::Or)
            && kinds.iter().all(|k| {
                matches!(
                    k,
                    ValueKind::StrategyDirection
                        | ValueKind::Bool
                        | ValueKind::Int
                        | ValueKind::Float
                        | ValueKind::Na
                )
            });
        if !equality && !logical {
            return None;
        }
        if logical {
            for (ty, span) in [left, right] {
                if !self.legacy_direction_bool_arg(ty, Accepts::BoolCompatible, span)
                    && ty.kind != ValueKind::Bool
                {
                    self.record_numeric_to_bool_coercion(span);
                }
            }
        }
        Some(PineType::new(
            strongest_qualifier(left.0.qualifier, right.0.qualifier),
            ValueKind::Bool,
        ))
    }
}
