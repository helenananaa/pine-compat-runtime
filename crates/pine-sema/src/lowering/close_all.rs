use crate::prelude::*;

impl Analyzer {
    pub(super) fn lower_close_all_eager_call(
        &mut self,
        expr: &Expr,
        source_args: &[CallArg],
        mut lowered_args: Vec<HirCallArg>,
        param_types: &HashMap<String, PineType>,
        call_info: (pine_ir::CallSiteId, PineType, Option<pine_ir::SeriesId>),
    ) -> Option<HirExpr> {
        let statements =
            self.lower_close_all_argument_evaluations(source_args, &mut lowered_args, param_types)?;
        let (call_site_id, pine_type, series_id) = call_info;
        self.finish_legacy_expr_coercion(
            expr,
            super::prepend_block_statements(
                statements,
                HirExpr {
                    pine_type,
                    series_id,
                    kind: HirExprKind::Call {
                        callee: "strategy.close_all".to_owned(),
                        call_site_id,
                        args: lowered_args,
                    },
                },
            ),
        )
    }

    /// Evaluate explicit arguments before the broker's conditional gate.
    /// Runtime slots remain stable; temporary values preserve overload parameter order and
    /// ensure stateful expressions execute exactly once even when `when` is false.
    pub(super) fn lower_close_all_argument_evaluations(
        &mut self,
        source_args: &[CallArg],
        lowered_args: &mut [HirCallArg],
        param_types: &HashMap<String, PineType>,
    ) -> Option<Vec<HirStmt>> {
        let first_type = source_args
            .first()
            .and_then(|arg| self.type_of_expr_with_params(&arg.value, param_types));
        let comment_overload = Self::close_all_comment_overload(source_args, first_type);
        let signature = pine_builtins::get_phase_1_builtin("strategy.close_all")?;
        let mut statements = Vec::with_capacity(source_args.len());
        for parameter in 0..4 {
            let Some(name) = pine_builtins::strategy_close_all_source_parameter(
                parameter,
                self.legacy.dialect().version(),
                comment_overload,
            ) else {
                break;
            };
            let source = source_args.iter().enumerate().find_map(|(index, arg)| {
                let arg_name = arg.name.as_deref().or_else(|| {
                    pine_builtins::strategy_close_all_source_parameter(
                        index,
                        self.legacy.dialect().version(),
                        comment_overload,
                    )
                });
                (arg_name == Some(name)).then_some(arg)
            });
            let Some(source) = source else {
                continue;
            };
            let slot = signature
                .params
                .iter()
                .position(|param| param.name == name)?;
            let arg = lowered_args.get_mut(slot)?;
            if !self.record_lowering_temp_symbol(source.span)
                || !self.record_lowering_node(source.span)
                || !self.record_lowering_node(source.span)
            {
                return None;
            }
            let symbol = self.fresh_temp_symbol("close_all.argument", arg.value.pine_type);
            let value = std::mem::replace(
                &mut arg.value,
                HirExpr {
                    kind: HirExprKind::Symbol(symbol.id),
                    pine_type: symbol.pine_type,
                    series_id: symbol.series_id,
                },
            );
            statements.push(HirStmt {
                kind: HirStmtKind::Decl {
                    symbol: symbol.id,
                    value,
                },
            });
        }
        if statements.len() != source_args.len()
            || (!statements.is_empty() && !self.record_lowering_node(source_args[0].span))
        {
            return None;
        }
        Some(statements)
    }
}
