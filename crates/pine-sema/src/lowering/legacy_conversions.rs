use super::*;
use crate::PineDialect;

impl Analyzer {
    pub(super) fn lower_legacy_direction_order_conditions(
        &mut self,
        name: &str,
        args: &mut [HirCallArg],
        span: Span,
    ) -> Option<()> {
        if self.legacy.dialect().version() > 3 || !name.starts_with("strategy.") {
            return Some(());
        }
        let Some(index) = pine_builtins::get_phase_1_builtin(name)
            .and_then(|sig| sig.params.iter().position(|p| p.name == "when"))
        else {
            return Some(());
        };
        for (i, arg) in args.iter_mut().enumerate() {
            if (arg.name.as_deref() == Some("when") || (arg.name.is_none() && i == index))
                && arg.value.pine_type.kind == ValueKind::StrategyDirection
            {
                if !self.record_lowering_node(span) {
                    return None;
                }
                let pine_type = PineType::new(arg.value.pine_type.qualifier, ValueKind::Bool);
                arg.value = HirExpr {
                    pine_type,
                    series_id: self.series_id_for_type(pine_type),
                    kind: HirExprKind::Call {
                        callee: "__legacy_direction_order_condition".to_owned(),
                        call_site_id: self.alloc_call_site(),
                        args: vec![HirCallArg {
                            name: None,
                            value: arg.value.clone(),
                        }],
                    },
                };
            }
        }
        Some(())
    }
    pub(super) fn lower_ternary_bool_branch(
        &mut self,
        branch: HirExpr,
        result_kind: ValueKind,
        span: Span,
    ) -> Option<HirExpr> {
        if !matches!(self.legacy.dialect(), PineDialect::V4 | PineDialect::V5)
            || result_kind != ValueKind::Bool
            || !matches!(branch.pine_type.kind, ValueKind::Int | ValueKind::Float)
        {
            return Some(branch);
        }
        if !self.record_lowering_node(span) {
            return None;
        }
        let pine_type = PineType::new(branch.pine_type.qualifier, ValueKind::Bool);
        Some(HirExpr {
            pine_type,
            series_id: self.series_id_for_type(pine_type),
            kind: HirExprKind::Call {
                callee: "bool".to_owned(),
                call_site_id: self.alloc_call_site(),
                args: vec![HirCallArg {
                    name: None,
                    value: branch,
                }],
            },
        })
    }

    pub(super) fn finish_legacy_expr_coercion(
        &mut self,
        expr: &Expr,
        mut lowered: HirExpr,
    ) -> Option<HirExpr> {
        let key = self.expr_key(expr.span);
        if self.legacy_bool_to_float_exprs.contains(&key) {
            if !self.record_lowering_node(expr.span) {
                return None;
            }
            let pine_type = PineType::new(lowered.pine_type.qualifier, ValueKind::Float);
            lowered = HirExpr {
                series_id: self.series_id_for_type(pine_type),
                pine_type,
                kind: HirExprKind::Call {
                    callee: "float".to_owned(),
                    call_site_id: self.alloc_call_site(),
                    args: vec![HirCallArg {
                        name: None,
                        value: lowered,
                    }],
                },
            };
        }
        if self.legacy_numeric_to_bool_exprs.contains(&key) {
            if !self.record_lowering_node(expr.span) {
                return None;
            }
            let pine_type = PineType::new(lowered.pine_type.qualifier, ValueKind::Bool);
            lowered = HirExpr {
                series_id: self.series_id_for_type(pine_type),
                pine_type,
                kind: HirExprKind::Call {
                    callee: "bool".to_owned(),
                    call_site_id: self.alloc_call_site(),
                    args: vec![HirCallArg {
                        name: None,
                        value: lowered,
                    }],
                },
            };
        }
        if self.legacy_integer_division_exprs.contains(&key) {
            if !self.record_lowering_node(expr.span) {
                return None;
            }
            let pine_type = PineType::new(lowered.pine_type.qualifier, ValueKind::Int);
            lowered = HirExpr {
                series_id: self.series_id_for_type(pine_type),
                pine_type,
                kind: HirExprKind::Call {
                    callee: "int".to_owned(),
                    call_site_id: self.alloc_call_site(),
                    args: vec![HirCallArg {
                        name: None,
                        value: lowered,
                    }],
                },
            };
        }
        if self.lowering_inline_depth > 0
            && let Some(series_id) = lowered.series_id
        {
            self.execution_scoped_series_ids.insert(series_id);
        }
        Some(lowered)
    }
}
