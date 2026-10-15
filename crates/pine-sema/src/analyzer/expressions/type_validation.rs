use crate::PineDialect;
use crate::prelude::*;

impl Analyzer {
    pub(crate) fn validate_history_offset(&mut self, offset: &Expr, offset_type: Option<PineType>) {
        if let Some(value) = self.known_history_offset_int_value(offset) {
            if value < 0 {
                self.unsupported(
                    "negative_history_offset",
                    "history offsets must be non-negative in the current supported subset",
                    offset.span,
                );
            }
            return;
        }

        let Some(offset_type) = offset_type else {
            self.unsupported(
                "dynamic_history_offset",
                "dynamic history offsets require a numeric expression in the current supported subset",
                offset.span,
            );
            return;
        };

        if matches!(offset_type.kind, ValueKind::Int | ValueKind::Float) {
            return;
        }

        let actual = pine_type_name(offset_type);
        self.unsupported(
            "dynamic_history_offset",
            &format!(
                "dynamic history offsets require a numeric expression in the current supported subset; got {actual}"
            ),
            offset.span,
        );
    }

    pub(crate) fn validate_assignment(
        &mut self,
        name: &str,
        target_type: PineType,
        value_type: PineType,
        span: Span,
    ) {
        if !can_assign(target_type, value_type) {
            self.diagnostics.push(Diagnostic::error(
                "E_ASSIGN_TYPE",
                format!(
                    "cannot assign {} to `{}` of type {}",
                    pine_type_name(value_type),
                    name,
                    pine_type_name(target_type)
                ),
                span,
            ));
        }
    }

    pub(crate) fn infer_unary(
        &mut self,
        op: UnaryOp,
        expr_type: PineType,
        span: Span,
    ) -> Option<PineType> {
        match op {
            UnaryOp::Plus | UnaryOp::Minus if is_numeric(expr_type.kind) => Some(expr_type),
            UnaryOp::Not if expr_type.kind == ValueKind::Bool => Some(expr_type),
            _ => {
                let expected = match op {
                    UnaryOp::Plus | UnaryOp::Minus => "numeric",
                    UnaryOp::Not => "bool",
                };
                self.unary_operator_error(op, expected, expr_type, span);
                None
            }
        }
    }

    pub(crate) fn infer_binary(
        &mut self,
        op: BinaryOp,
        left_type: PineType,
        right_type: PineType,
        span: Span,
    ) -> Option<PineType> {
        if matches!(op, BinaryOp::Add | BinaryOp::Sub)
            && left_type.kind == ValueKind::PlotDisplay
            && right_type.kind == ValueKind::PlotDisplay
        {
            return Some(PineType::new(
                strongest_qualifier(left_type.qualifier, right_type.qualifier),
                ValueKind::PlotDisplay,
            ));
        }
        match op {
            BinaryOp::Add => {
                if left_type.kind == ValueKind::String && right_type.kind == ValueKind::String {
                    Some(PineType::new(
                        strongest_qualifier(left_type.qualifier, right_type.qualifier),
                        ValueKind::String,
                    ))
                } else if is_numeric(left_type.kind) && is_numeric(right_type.kind) {
                    Some(PineType::new(
                        strongest_qualifier(left_type.qualifier, right_type.qualifier),
                        numeric_result_kind(op, left_type.kind, right_type.kind),
                    ))
                } else {
                    self.operator_error(
                        op,
                        "numeric or string operands",
                        left_type,
                        right_type,
                        span,
                    );
                    None
                }
            }
            BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if is_numeric(left_type.kind) && is_numeric(right_type.kind) {
                    Some(PineType::new(
                        strongest_qualifier(left_type.qualifier, right_type.qualifier),
                        numeric_result_kind(op, left_type.kind, right_type.kind),
                    ))
                } else {
                    self.operator_error(op, "numeric operands", left_type, right_type, span);
                    None
                }
            }
            BinaryOp::Eq | BinaryOp::NotEq => {
                let legacy_numeric_bool =
                    matches!(self.legacy.dialect(), PineDialect::V4 | PineDialect::V5)
                        && matches!(
                            (left_type.kind, right_type.kind),
                            (ValueKind::Bool, ValueKind::Int | ValueKind::Float)
                                | (ValueKind::Int | ValueKind::Float, ValueKind::Bool)
                        );
                if common_kind(left_type.kind, right_type.kind).is_some() || legacy_numeric_bool {
                    Some(PineType::new(
                        strongest_qualifier(left_type.qualifier, right_type.qualifier),
                        ValueKind::Bool,
                    ))
                } else {
                    self.operator_error(op, "comparable operands", left_type, right_type, span);
                    None
                }
            }
            BinaryOp::Gt | BinaryOp::Gte | BinaryOp::Lt | BinaryOp::Lte => {
                if is_numeric(left_type.kind) && is_numeric(right_type.kind) {
                    Some(PineType::new(
                        strongest_qualifier(left_type.qualifier, right_type.qualifier),
                        ValueKind::Bool,
                    ))
                } else {
                    self.operator_error(op, "numeric operands", left_type, right_type, span);
                    None
                }
            }
            BinaryOp::And | BinaryOp::Or => {
                if left_type.kind == ValueKind::Bool && right_type.kind == ValueKind::Bool {
                    Some(PineType::new(
                        strongest_qualifier(left_type.qualifier, right_type.qualifier),
                        ValueKind::Bool,
                    ))
                } else {
                    self.operator_error(op, "bool operands", left_type, right_type, span);
                    None
                }
            }
        }
    }

    pub(crate) fn unary_operator_error(
        &mut self,
        op: UnaryOp,
        expected: &str,
        expr_type: PineType,
        span: Span,
    ) {
        self.diagnostics.push(Diagnostic::error(
            "E_OPERATOR_TYPE",
            format!(
                "operator `{}` expects {}, got {}",
                unary_operator_label(op),
                expected,
                pine_type_name(expr_type)
            ),
            span,
        ));
    }

    pub(crate) fn operator_error(
        &mut self,
        op: BinaryOp,
        expected: &str,
        left_type: PineType,
        right_type: PineType,
        span: Span,
    ) {
        self.diagnostics.push(Diagnostic::error(
            "E_OPERATOR_TYPE",
            format!(
                "operator `{}` expects {}, got {} and {}",
                binary_operator_label(op),
                expected,
                pine_type_name(left_type),
                pine_type_name(right_type)
            ),
            span,
        ));
    }

    pub(crate) fn expect_bool(&mut self, pine_type: PineType, span: Span) {
        if pine_type.kind == ValueKind::Bool {
            return;
        }
        if self.legacy.dialect() != PineDialect::V6
            && matches!(
                pine_type.kind,
                ValueKind::Int | ValueKind::Float | ValueKind::Na
            )
        {
            self.record_numeric_to_bool_coercion(span);
        } else {
            self.diagnostics.push(Diagnostic::error(
                "E_CONDITION_TYPE",
                format!("condition must be bool, got {}", pine_type_name(pine_type)),
                span,
            ));
        }
    }

    pub(crate) fn expect_numeric_loop_bound(&mut self, pine_type: PineType, span: Span) {
        if !matches!(pine_type.kind, ValueKind::Int | ValueKind::Float) {
            self.diagnostics.push(Diagnostic::error(
                "E_LOOP_RANGE_TYPE",
                format!(
                    "for loop range must be int or float, got {}",
                    pine_type_name(pine_type)
                ),
                span,
            ));
        }
    }

    pub(crate) fn expect_loop_step_type(
        &mut self,
        from_type: Option<PineType>,
        step_type: PineType,
        span: Span,
    ) {
        self.expect_numeric_loop_bound(step_type, span);
        if from_type.is_some_and(|value| value.kind == ValueKind::Int)
            && step_type.kind == ValueKind::Float
        {
            self.diagnostics.push(Diagnostic::error(
                "E_LOOP_RANGE_TYPE",
                "for loop with an int counter requires an int step",
                span,
            ));
        }
    }

    pub(crate) fn expect_non_zero_loop_step(&mut self, step: &Expr) {
        if self.known_const_numeric_value(step) == Some(0.0) {
            self.diagnostics.push(Diagnostic::error(
                "E_LOOP_STEP",
                "for loop step cannot be zero",
                step.span,
            ));
        }
    }

    pub(crate) fn merge_branch_types(
        &mut self,
        condition_type: PineType,
        then_type: PineType,
        else_type: PineType,
        condition_value: Option<bool>,
        span: Span,
    ) -> Option<PineType> {
        let Some(kind) = common_kind(then_type.kind, else_type.kind) else {
            self.diagnostics.push(Diagnostic::error(
                "E_BRANCH_TYPE",
                format!(
                    "ternary branches have incompatible types {} and {}",
                    value_kind_name(then_type.kind),
                    value_kind_name(else_type.kind)
                ),
                span,
            ));
            return None;
        };
        let branch_qualifier = match condition_value {
            Some(true) => then_type.qualifier,
            Some(false) => else_type.qualifier,
            None => strongest_qualifier(then_type.qualifier, else_type.qualifier),
        };

        Some(PineType::new(
            strongest_qualifier(condition_type.qualifier, branch_qualifier),
            kind,
        ))
    }
}

fn unary_operator_label(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Plus => "+",
        UnaryOp::Minus => "-",
        UnaryOp::Not => "not",
    }
}

fn binary_operator_label(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "%",
        BinaryOp::Eq => "==",
        BinaryOp::NotEq => "!=",
        BinaryOp::Gt => ">",
        BinaryOp::Gte => ">=",
        BinaryOp::Lt => "<",
        BinaryOp::Lte => "<=",
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
    }
}
