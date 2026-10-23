use super::*;

impl Analyzer {
    pub(super) fn type_of_branch_return_with_params(
        &self,
        branch: &[Stmt],
        param_types: &HashMap<String, PineType>,
    ) -> Option<PineType> {
        let last = branch.last()?;
        match &last.kind {
            StmtKind::Decl { name, .. } | StmtKind::Reassign { name, .. } => self
                .bound_symbol(name, last.span)
                .map(|symbol| symbol.pine_type),
            StmtKind::Expr(expr) | StmtKind::TupleDecl { value: expr, .. } => {
                self.type_of_expr_with_params(expr, param_types)
            }
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.type_of_if_expr_with_params(condition, then_branch, else_branch, param_types),
            StmtKind::For {
                from,
                to,
                step,
                body,
                ..
            } => self.type_of_for_expr_with_params(from, to, step.as_ref(), body, param_types),
            StmtKind::ForIn { iterable, body, .. } => {
                self.type_of_for_in_expr_with_params(iterable, body, param_types)
            }
            StmtKind::While { condition, body } => {
                self.type_of_while_expr_with_params(condition, body, param_types)
            }
            _ => None,
        }
    }

    pub(super) fn type_of_loop_body_return_with_params(
        &self,
        statement: &Stmt,
        param_types: &HashMap<String, PineType>,
    ) -> Option<PineType> {
        match &statement.kind {
            StmtKind::Decl { name, .. } | StmtKind::Reassign { name, .. } => self
                .bound_symbol(name, statement.span)
                .map(|symbol| symbol.pine_type),
            StmtKind::FieldReassign {
                receiver,
                path,
                field,
                ..
            } => self.type_of_expr_with_params(
                &crate::analyzer::functions::field_reassign_result_expr(
                    receiver,
                    path,
                    field,
                    statement.span,
                ),
                param_types,
            ),
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.type_of_function_if_return_with_params(
                condition,
                then_branch,
                else_branch,
                param_types,
            ),
            StmtKind::Expr(expr) | StmtKind::TupleDecl { value: expr, .. } => {
                self.type_of_expr_with_params(expr, param_types)
            }
            StmtKind::For {
                from,
                to,
                step,
                body,
                ..
            } => self.type_of_for_expr_with_params(from, to, step.as_ref(), body, param_types),
            StmtKind::ForIn { iterable, body, .. } => {
                self.type_of_for_in_expr_with_params(iterable, body, param_types)
            }
            StmtKind::While { condition, body } => {
                self.type_of_while_expr_with_params(condition, body, param_types)
            }
            _ => None,
        }
    }
}
