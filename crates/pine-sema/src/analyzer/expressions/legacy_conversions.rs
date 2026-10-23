use super::*;
use std::collections::HashSet;

impl Analyzer {
    pub(crate) fn ternary_common_kind(
        &self,
        left: ValueKind,
        right: ValueKind,
    ) -> Option<ValueKind> {
        if matches!(self.legacy.dialect(), PineDialect::V4 | PineDialect::V5)
            && ((left == ValueKind::Bool && matches!(right, ValueKind::Int | ValueKind::Float))
                || (right == ValueKind::Bool && matches!(left, ValueKind::Int | ValueKind::Float)))
        {
            return Some(ValueKind::Bool);
        }
        common_kind(left, right)
    }

    pub(crate) fn v4_v5_series_output_offset_arg(
        &mut self,
        call_name: &str,
        param_name: &str,
        expr: &Expr,
        actual_type: PineType,
        accepts: Accepts,
    ) -> bool {
        if !matches!(self.legacy.dialect(), PineDialect::V4 | PineDialect::V5)
            || param_name != "offset"
            || accepts != Accepts::SimpleIntCompatible
            || actual_type != PineType::new(Qualifier::Series, ValueKind::Int)
            || !matches!(
                call_name,
                "plot" | "plotchar" | "plotshape" | "plotarrow" | "bgcolor" | "barcolor"
            )
        {
            return false;
        }

        if self
            .v4_v5_series_output_offset_exprs
            .insert(self.expr_key(expr.span))
        {
            let version = self.legacy.dialect().version();
            self.compatibility
                .legacy_emulations
                .push(crate::compatibility::LegacyEmulation {
                    feature: format!("v{version}.series_output_offset"),
                    behavior: format!(
                        "Pine v{version} output offsets accept a series int and apply its final evaluated value to the complete rendered output"
                    ),
                    span: expr.span,
                });
        }
        true
    }

    pub(crate) fn legacy_numeric_bool_arg(
        &mut self,
        expr: &Expr,
        actual_type: PineType,
        accepts: Accepts,
    ) -> bool {
        if self.legacy_direction_bool_arg(actual_type, accepts, expr.span) {
            return true;
        }
        if self.legacy.dialect() == PineDialect::V6
            || !matches!(
                accepts,
                Accepts::BoolCompatible | Accepts::SimpleBoolCompatible
            )
            || !matches!(
                actual_type.kind,
                ValueKind::Int | ValueKind::Float | ValueKind::Na
            )
        {
            return false;
        }

        let coerced_type = PineType::new(actual_type.qualifier, ValueKind::Bool);
        if !accepts_type(accepts, coerced_type) {
            return false;
        }
        let key = self.expr_key(expr.span);
        if !self.legacy_numeric_to_bool_exprs.contains(&key) {
            self.record_numeric_to_bool_coercion(expr.span);
        }
        true
    }

    pub(super) fn uses_versioned_integer_division(
        &self,
        op: BinaryOp,
        left_type: PineType,
        right_type: PineType,
    ) -> bool {
        op == BinaryOp::Div
            && left_type.kind == ValueKind::Int
            && right_type.kind == ValueKind::Int
            && (self.legacy.dialect() <= PineDialect::V3
                || (matches!(self.legacy.dialect(), PineDialect::V4 | PineDialect::V5)
                    && left_type.qualifier == Qualifier::Const
                    && right_type.qualifier == Qualifier::Const))
    }

    pub(crate) fn extreme_division_length_arg(
        &mut self,
        call_name: &str,
        param_name: &str,
        expr: &Expr,
        actual_type: PineType,
    ) -> bool {
        let legacy_ema = self.legacy.dialect() >= PineDialect::V4 && call_name == "ta.ema";
        let extreme = matches!(self.legacy.dialect(), PineDialect::V5 | PineDialect::V6)
            && matches!(call_name, "ta.lowest" | "ta.highest");
        if !(legacy_ema || extreme)
            || param_name != "length"
            || actual_type.kind != ValueKind::Float
        {
            return false;
        }
        // Integer arithmetic can retain a fractional representation in Pine,
        // while remaining acceptable in an EMA integer-length context. Follow
        // aliases without accepting explicit float inputs or float operands.
        // The runtime truncates the final length, not intermediate quotients.
        if legacy_ema {
            let accepted = accepts_type(
                Accepts::SimpleIntCompatible,
                PineType::new(actual_type.qualifier, ValueKind::Int),
            ) && self.integer_arithmetic_length_expr(expr, &mut HashSet::new());
            if accepted {
                let version = self.legacy.dialect().version();
                self.compatibility.legacy_emulations.push(crate::compatibility::LegacyEmulation {
                    feature: format!("v{version}.ema_division_length"),
                    behavior: format!("Pine v{version} EMA length accepts represented integer arithmetic and truncates the final length; ordinary quotient values retain fractions"),
                    span: expr.span,
                });
            }
            return accepted;
        }
        let ExprKind::Binary {
            op: BinaryOp::Div,
            left,
            right,
        } = &expr.kind
        else {
            return false;
        };
        if ![left, right].iter().all(|operand| {
            self.analyze_expr(operand)
                .is_some_and(|ty| ty.kind == ValueKind::Int)
        }) {
            return false;
        }
        if self
            .legacy_integer_division_exprs
            .insert(self.expr_key(expr.span))
        {
            self.compatibility
                .legacy_emulations
                .push(crate::compatibility::LegacyEmulation {
                    feature: format!("v{}.{}division_length", self.legacy.dialect().version(), if legacy_ema { "ema_" } else { "extreme_" }),
                    behavior: format!("Pine v{} {call_name} length context truncates an integer-operand quotient; the ordinary quotient retains its fractional value", self.legacy.dialect().version()),
                    span: expr.span,
                });
        }
        true
    }

    fn integer_arithmetic_length_expr(
        &self,
        expr: &Expr,
        visiting: &mut HashSet<SymbolId>,
    ) -> bool {
        let expr = expr.without_groups();
        if self
            .type_of_expr_with_params(expr, &Default::default())
            .is_some_and(|ty| ty.kind == ValueKind::Int)
        {
            return true;
        }
        match &expr.kind {
            ExprKind::Identifier(name) => {
                let Some(symbol) = self
                    .bound_symbol(name, expr.span)
                    .or_else(|| self.scope.resolve(name))
                else {
                    return false;
                };
                if visiting.len() >= 64 || !visiting.insert(symbol.id) {
                    return false;
                }
                let result = self
                    .with_symbol_initializer(symbol.id, |analyzer, init| {
                        Some(analyzer.integer_arithmetic_length_expr(init, visiting))
                    })
                    .unwrap_or(false);
                visiting.remove(&symbol.id);
                result
            }
            ExprKind::Binary {
                op: BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod,
                left,
                right,
            } => {
                self.integer_arithmetic_length_expr(left, visiting)
                    && self.integer_arithmetic_length_expr(right, visiting)
            }
            ExprKind::Unary {
                op: UnaryOp::Plus | UnaryOp::Minus,
                expr,
            } => self.integer_arithmetic_length_expr(expr, visiting),
            ExprKind::Ternary {
                then_expr,
                else_expr,
                ..
            } => {
                self.integer_arithmetic_length_expr(then_expr, visiting)
                    && self.integer_arithmetic_length_expr(else_expr, visiting)
            }
            _ => false,
        }
    }

    fn record_versioned_integer_division(&mut self, span: Span, qualifier: Qualifier) -> PineType {
        if self
            .legacy_integer_division_exprs
            .insert(self.expr_key(span))
        {
            let version = self.legacy.dialect().version();
            let behavior = if matches!(self.legacy.dialect(), PineDialect::V4 | PineDialect::V5) {
                format!(
                    "Pine v{version} division of two const int values produces an int and discards the fractional remainder"
                )
            } else {
                format!(
                    "Pine v{version} division of two int values produces an int and discards the fractional remainder"
                )
            };
            self.compatibility
                .legacy_emulations
                .push(crate::compatibility::LegacyEmulation {
                    feature: format!("v{version}.integer_division"),
                    behavior,
                    span,
                });
        }
        PineType::new(qualifier, ValueKind::Int)
    }

    pub(super) fn infer_unary_with_legacy(
        &mut self,
        op: UnaryOp,
        expr_type: PineType,
        operand_span: Span,
        expression_span: Span,
    ) -> Option<PineType> {
        let dialect = self.legacy.dialect();
        if op == UnaryOp::Not
            && self.legacy_direction_bool_arg(expr_type, Accepts::BoolCompatible, operand_span)
        {
            return Some(PineType::new(expr_type.qualifier, ValueKind::Bool));
        }
        if matches!(dialect, PineDialect::V1 | PineDialect::V2)
            && matches!(op, UnaryOp::Plus | UnaryOp::Minus)
            && expr_type.kind == ValueKind::Bool
        {
            self.record_bool_to_float_coercion(operand_span);
            let result = PineType::new(expr_type.qualifier, ValueKind::Float);
            self.expr_types
                .insert(self.expr_key(expression_span), result);
            return Some(result);
        }
        if dialect != PineDialect::V6
            && op == UnaryOp::Not
            && matches!(
                expr_type.kind,
                ValueKind::Int | ValueKind::Float | ValueKind::Na
            )
        {
            self.record_numeric_to_bool_coercion(operand_span);
            let result = PineType::new(expr_type.qualifier, ValueKind::Bool);
            self.expr_types
                .insert(self.expr_key(expression_span), result);
            return Some(result);
        }
        self.infer_unary(op, expr_type, operand_span)
    }

    pub(super) fn infer_binary_with_legacy(
        &mut self,
        op: BinaryOp,
        left_type: PineType,
        right_type: PineType,
        left_span: Span,
        right_span: Span,
        span: Span,
    ) -> Option<PineType> {
        if let Some(ty) =
            self.infer_legacy_direction_binary(op, (left_type, left_span), (right_type, right_span))
        {
            return Some(ty);
        }
        let dialect = self.legacy.dialect();
        if self.uses_versioned_integer_division(op, left_type, right_type) {
            return Some(self.record_versioned_integer_division(
                span,
                strongest_qualifier(left_type.qualifier, right_type.qualifier),
            ));
        }
        let arithmetic = matches!(
            op,
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod
        );
        let comparison = matches!(
            op,
            BinaryOp::Eq
                | BinaryOp::NotEq
                | BinaryOp::Gt
                | BinaryOp::Gte
                | BinaryOp::Lt
                | BinaryOp::Lte
        );
        let legacy_numeric_kind = |kind| {
            matches!(
                kind,
                ValueKind::Int | ValueKind::Float | ValueKind::Bool | ValueKind::Na
            )
        };
        if matches!(dialect, PineDialect::V1 | PineDialect::V2)
            && arithmetic
            && (left_type.kind == ValueKind::Bool || right_type.kind == ValueKind::Bool)
            && legacy_numeric_kind(left_type.kind)
            && legacy_numeric_kind(right_type.kind)
        {
            if left_type.kind == ValueKind::Bool {
                self.record_bool_to_float_coercion(left_span);
            }
            if right_type.kind == ValueKind::Bool {
                self.record_bool_to_float_coercion(right_span);
            }
            let result = PineType::new(
                strongest_qualifier(left_type.qualifier, right_type.qualifier),
                ValueKind::Float,
            );
            self.expr_types.insert(self.expr_key(span), result);
            return Some(result);
        }
        if matches!(dialect, PineDialect::V1 | PineDialect::V2)
            && comparison
            && ((left_type.kind == ValueKind::Bool && is_numeric(right_type.kind))
                || (right_type.kind == ValueKind::Bool && is_numeric(left_type.kind)))
        {
            if left_type.kind == ValueKind::Bool {
                self.record_bool_to_float_coercion(left_span);
            }
            if right_type.kind == ValueKind::Bool {
                self.record_bool_to_float_coercion(right_span);
            }
            let result = PineType::new(
                strongest_qualifier(left_type.qualifier, right_type.qualifier),
                ValueKind::Bool,
            );
            self.expr_types.insert(self.expr_key(span), result);
            return Some(result);
        }
        if dialect != PineDialect::V6
            && matches!(op, BinaryOp::And | BinaryOp::Or)
            && matches!(
                left_type.kind,
                ValueKind::Bool | ValueKind::Int | ValueKind::Float | ValueKind::Na
            )
            && matches!(
                right_type.kind,
                ValueKind::Bool | ValueKind::Int | ValueKind::Float | ValueKind::Na
            )
        {
            if left_type.kind != ValueKind::Bool {
                self.record_numeric_to_bool_coercion(left_span);
            }
            if right_type.kind != ValueKind::Bool {
                self.record_numeric_to_bool_coercion(right_span);
            }
            return Some(PineType::new(
                strongest_qualifier(left_type.qualifier, right_type.qualifier),
                ValueKind::Bool,
            ));
        }
        self.infer_binary(op, left_type, right_type, span)
    }

    pub(super) fn record_bool_to_float_coercion(&mut self, span: Span) {
        self.legacy_bool_to_float_exprs.insert(self.expr_key(span));
        let version = self.legacy.dialect().version();
        self.compatibility
            .legacy_emulations
            .push(crate::compatibility::LegacyEmulation {
                feature: format!("v{version}.bool_arithmetic"),
                behavior: format!(
                    "Pine v{version} boolean arithmetic converts true to 1.0, false to 0.0, and unavailable boolean values to canonical na before applying the numeric operator"
                ),
                span,
            });
    }

    pub(crate) fn record_numeric_to_bool_coercion(&mut self, span: Span) {
        self.legacy_numeric_to_bool_exprs
            .insert(self.expr_key(span));
        let version = self.legacy.dialect().version();
        self.compatibility
            .legacy_emulations
            .push(crate::compatibility::LegacyEmulation {
                feature: format!("v{version}.numeric_to_bool"),
                behavior: format!(
                    "Pine v{version} condition conversion treats zero and na as false and every other finite numeric value as true"
                ),
                span,
            });
    }
}
