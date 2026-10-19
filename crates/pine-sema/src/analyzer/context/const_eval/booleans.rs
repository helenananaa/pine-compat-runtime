use super::*;
use pine_syntax::{BinaryOp, ExprKind, Literal, StmtKind, UnaryOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Identity {
    Primitive,
    LegacyConstant,
    MissingConstant(SourceContextId, usize),
}

/// A known runtime payload is distinct from a primitive condition the native
/// compiler can use to prune qualifier branches. Broker configuration and value
/// storage consume the payload. The early dynamic_requests annotation is handled
/// separately from value evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ConstantBoolean {
    payload: bool,
    identity: Identity,
    foldable: bool,
}

impl ConstantBoolean {
    pub(crate) fn primitive(payload: bool) -> Self {
        Self {
            payload,
            identity: Identity::Primitive,
            foldable: true,
        }
    }

    pub(crate) fn condition_value(self) -> Option<bool> {
        (self.foldable && self.identity == Identity::Primitive).then_some(self.payload)
    }

    pub(crate) fn materialized(mut self) -> Self {
        self.identity = Identity::Primitive;
        self
    }

    fn truthy(self) -> bool {
        self.identity != Identity::Primitive || self.payload
    }

    fn with_condition(mut self, condition: Self) -> Self {
        self.foldable &= condition.foldable;
        self
    }

    fn equals(self, other: Self) -> bool {
        self.payload == other.payload && self.identity == other.identity
    }
}

impl Analyzer {
    pub(crate) fn materializes_boolean_declaration(
        &self,
        declared_type: Option<&pine_syntax::DeclaredType>,
    ) -> bool {
        self.legacy.dialect().version() == 5
            && declared_type.and_then(pine_syntax::DeclaredType::named_type) == Some("bool")
    }

    pub(crate) fn known_const_bool_payload_value(&self, expr: &pine_syntax::Expr) -> Option<bool> {
        self.known_constant_boolean(expr).map(|value| value.payload)
    }

    pub(crate) fn known_constant_boolean(
        &self,
        expr: &pine_syntax::Expr,
    ) -> Option<ConstantBoolean> {
        let expr = expr.without_groups();
        match &expr.kind {
            ExprKind::Literal(Literal::Bool(value)) => Some(ConstantBoolean::primitive(*value)),
            ExprKind::QualifiedName(parts) => crate::legacy::strategy_direction_bool(
                &parts.join("."),
                self.legacy.dialect().version(),
            )
            .map(ConstantBoolean::primitive),
            ExprKind::Identifier(name) => {
                if let Some(ConstSwitchKey::Bool(value)) =
                    self.function_param_const_switch_key(name)
                {
                    return Some(ConstantBoolean::primitive(*value));
                }
                let symbol = self.const_lookup_symbol(name, expr.span)?;
                self.const_bool_symbols.get(&symbol.id).copied()
            }
            ExprKind::Unary {
                op: UnaryOp::Not,
                expr,
            } => {
                let value = self.known_constant_boolean(expr)?;
                Some(ConstantBoolean::primitive(!value.truthy()).with_condition(value))
            }
            ExprKind::Binary {
                op: op @ (BinaryOp::And | BinaryOp::Or),
                left,
                right,
            } => {
                let left = self.known_constant_boolean(left)?;
                let right = self.known_constant_boolean(right)?;
                let selected = match op {
                    BinaryOp::And if left.truthy() => right,
                    BinaryOp::Or if !left.truthy() => right,
                    _ => left,
                };
                Some(selected.with_condition(left).with_condition(right))
            }
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                let condition = self.known_constant_boolean(condition)?;
                self.known_constant_boolean(if condition.truthy() {
                    then_expr
                } else {
                    else_expr
                })
                .map(|value| value.with_condition(condition))
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let value = self.known_constant_boolean(condition)?;
                let version = self.legacy.dialect().version();
                if !value.truthy() && else_branch.is_empty() && version >= 6 {
                    if self.type_of_expr_with_params(expr, &HashMap::new())?.kind
                        != pine_ir::ValueKind::Bool
                    {
                        return None;
                    }
                    return Some(ConstantBoolean {
                        payload: false,
                        identity: Identity::MissingConstant(
                            self.source_context_id.get(),
                            condition.span.start,
                        ),
                        foldable: false,
                    });
                }
                let branch = if value.truthy() {
                    then_branch
                } else {
                    else_branch
                };
                let StmtKind::Expr(result) = &branch.last()?.kind else {
                    return None;
                };
                let mut result = self.known_constant_boolean(result)?.with_condition(value);
                if version == 5 {
                    result.identity = Identity::LegacyConstant;
                    result.foldable = false;
                }
                Some(result)
            }
            ExprKind::Switch { selector, arms } => self
                .known_const_bool_switch_result(selector.as_deref(), arms)
                .map(ConstantBoolean::primitive),
            ExprKind::Binary {
                op:
                    op @ (BinaryOp::Eq
                    | BinaryOp::NotEq
                    | BinaryOp::Gt
                    | BinaryOp::Gte
                    | BinaryOp::Lt
                    | BinaryOp::Lte),
                left,
                right,
            } => {
                if matches!(op, BinaryOp::Eq | BinaryOp::NotEq)
                    && let (Some(left), Some(right)) = (
                        self.known_constant_boolean(left),
                        self.known_constant_boolean(right),
                    )
                {
                    let equal = left.equals(right);
                    return Some(
                        ConstantBoolean::primitive(if *op == BinaryOp::Eq {
                            equal
                        } else {
                            !equal
                        })
                        .with_condition(left)
                        .with_condition(right),
                    );
                }
                self.known_const_numeric_comparison(*op, left, right)
                    .or_else(|| self.known_const_bool_comparison(*op, left, right))
                    .or_else(|| self.known_const_string_comparison(*op, left, right))
                    .or_else(|| self.known_const_color_comparison(*op, left, right))
                    .map(ConstantBoolean::primitive)
            }
            _ => None,
        }
    }
}
