use super::*;
mod dataflow;
use dataflow::result_depends_on_parameters;
pub(super) use dataflow::strip_boolean_return_boundaries;

impl Analyzer {
    pub(super) fn materialize_lowered_boolean(&self, value: HirExpr) -> HirExpr {
        if self.constant_bool_origins.is_empty() || value.pine_type.kind != ValueKind::Bool {
            return value;
        }
        boolean_unary(HirUnaryOp::MaterializeBool, value)
    }

    pub(super) fn materialize_function_boolean(
        &mut self,
        value: HirExpr,
        params: &HashMap<String, HirExpr>,
    ) -> HirExpr {
        if self.legacy.dialect().version() < 5 || value.pine_type.kind != ValueKind::Bool {
            return value;
        }
        self.has_boolean_return_boundaries = true;
        let materialize = !result_depends_on_parameters(&value, params);
        boolean_unary(HirUnaryOp::FunctionBoolReturn { materialize }, value)
    }

    pub(super) fn preserve_if_boolean_identity(
        &mut self,
        source_condition: &Expr,
        source_else: &[Stmt],
        result: &mut HirExpr,
    ) {
        if result.pine_type.kind != ValueKind::Bool {
            return;
        }
        let HirExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } = &mut result.kind
        else {
            return;
        };
        if condition.pine_type.qualifier != Qualifier::Const {
            return;
        }
        let version = self.legacy.dialect().version();
        if version == 5 {
            for branch in [then_expr, else_expr] {
                if branch.pine_type.kind == ValueKind::Bool
                    && branch.pine_type.qualifier == Qualifier::Const
                {
                    self.register_constant_bool_origin(source_condition);
                    **branch =
                        boolean_unary(HirUnaryOp::ConstantBoolIdentity(None), (**branch).clone());
                }
            }
        } else if version >= 6
            && source_else.is_empty()
            && !matches!(
                source_condition.without_groups().kind,
                ExprKind::Literal(Literal::Bool(true))
            )
        {
            let identity = self.register_constant_bool_origin(source_condition);
            **else_expr = boolean_unary(
                HirUnaryOp::ConstantBoolIdentity(Some(identity)),
                (**else_expr).clone(),
            );
        }
    }

    fn register_constant_bool_origin(&mut self, condition: &Expr) -> u64 {
        let next = self.constant_bool_origins.len() as u64;
        *self
            .constant_bool_origins
            .entry((self.source_context_id.get(), condition.span.start))
            .or_insert(next)
    }
}

fn boolean_unary(op: HirUnaryOp, expr: HirExpr) -> HirExpr {
    HirExpr {
        pine_type: expr.pine_type,
        series_id: expr.series_id,
        kind: HirExprKind::Unary {
            op,
            expr: Box::new(expr),
        },
    }
}
