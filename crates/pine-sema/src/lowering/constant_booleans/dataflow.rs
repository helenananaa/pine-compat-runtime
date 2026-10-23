use super::*;
use std::collections::HashSet;

#[cfg(test)]
mod tests;

type Bindings<'a> = HashMap<SymbolId, Vec<&'a HirExpr>>;

enum Node<'a> {
    Expr(&'a HirExpr),
    Stmt(&'a HirStmt),
}

/// Trace only the returned scalar's value dependencies. Effects that do not
/// contribute to that value do not change its constant-materialization phase.
/// Both walks are iterative so long HIR chains cannot exhaust the compiler stack.
pub(super) fn result_depends_on_parameters(
    value: &HirExpr,
    params: &HashMap<String, HirExpr>,
) -> bool {
    let parameters = params
        .values()
        .filter_map(|value| match value.kind {
            HirExprKind::Symbol(symbol) => Some(symbol),
            _ => None,
        })
        .collect::<HashSet<_>>();
    let bindings = collect_bindings(value);
    let mut symbols = HashSet::new();
    let mut stack = vec![value];
    while let Some(expr) = stack.pop() {
        match &expr.kind {
            HirExprKind::Symbol(symbol) => {
                if parameters.contains(symbol) {
                    return true;
                }
                if symbols.insert(*symbol)
                    && let Some(values) = bindings.get(symbol)
                {
                    stack.extend(values.iter().copied());
                }
            }
            HirExprKind::Unary {
                op: HirUnaryOp::FunctionBoolReturn { .. },
                ..
            } => return true,
            _ => stack.extend(expression_children(expr)),
        }
    }
    false
}

fn bind<'a>(
    bindings: &mut Bindings<'a>,
    symbol: SymbolId,
    values: impl IntoIterator<Item = &'a HirExpr>,
    controls: &[&'a HirExpr],
) {
    let entry = bindings.entry(symbol).or_default();
    entry.extend(values);
    entry.extend(controls.iter().copied());
}

fn collect_bindings(root: &HirExpr) -> Bindings<'_> {
    let mut bindings = HashMap::new();
    let mut stack = vec![(Node::Expr(root), Vec::<&HirExpr>::new())];
    while let Some((node, controls)) = stack.pop() {
        match node {
            Node::Expr(expr) => match &expr.kind {
                HirExprKind::Unary {
                    op: HirUnaryOp::FunctionBoolReturn { .. },
                    ..
                } => {}
                HirExprKind::Block { statements, result } => {
                    stack.push((Node::Expr(result), controls.clone()));
                    stack.extend(
                        statements
                            .iter()
                            .map(|statement| (Node::Stmt(statement), controls.clone())),
                    );
                }
                HirExprKind::Ternary {
                    condition,
                    then_expr,
                    else_expr,
                } => {
                    stack.push((Node::Expr(condition), controls.clone()));
                    let mut branch = controls;
                    branch.push(condition);
                    if let HirExprKind::Literal(HirLiteral::Bool(value)) = condition.kind {
                        stack.push((
                            Node::Expr(if value { then_expr } else { else_expr }),
                            branch,
                        ));
                    } else {
                        stack.push((Node::Expr(then_expr), branch.clone()));
                        stack.push((Node::Expr(else_expr), branch));
                    }
                }
                HirExprKind::For {
                    counter,
                    from,
                    to,
                    step,
                    statements,
                    result,
                } => {
                    let mut body = controls.clone();
                    body.extend([from.as_ref(), to.as_ref()]);
                    body.extend(step.as_deref());
                    bind(
                        &mut bindings,
                        *counter,
                        [from.as_ref(), to.as_ref()],
                        &controls,
                    );
                    stack.extend(
                        statements
                            .iter()
                            .map(|statement| (Node::Stmt(statement), body.clone())),
                    );
                    stack.push((Node::Expr(result), body));
                    stack.extend(
                        expression_children(expr)
                            .into_iter()
                            .filter(|child| !std::ptr::eq(*child, result.as_ref()))
                            .map(|child| (Node::Expr(child), controls.clone())),
                    );
                }
                HirExprKind::ForIn {
                    index,
                    value,
                    iterable,
                    statements,
                    result,
                } => {
                    let mut body = controls.clone();
                    body.push(iterable);
                    for symbol in index.iter().chain(std::iter::once(value)) {
                        bind(&mut bindings, *symbol, [iterable.as_ref()], &controls);
                    }
                    stack.extend(
                        statements
                            .iter()
                            .map(|statement| (Node::Stmt(statement), body.clone())),
                    );
                    stack.push((Node::Expr(result), body));
                    stack.push((Node::Expr(iterable), controls));
                }
                HirExprKind::While {
                    condition,
                    statements,
                    result,
                } => {
                    let mut body = controls.clone();
                    body.push(condition);
                    stack.extend(
                        statements
                            .iter()
                            .map(|statement| (Node::Stmt(statement), body.clone())),
                    );
                    stack.push((Node::Expr(result), body));
                    stack.push((Node::Expr(condition), controls));
                }
                _ => stack.extend(
                    expression_children(expr)
                        .into_iter()
                        .map(|child| (Node::Expr(child), controls.clone())),
                ),
            },
            Node::Stmt(statement) => match &statement.kind {
                HirStmtKind::Decl { symbol, value } | HirStmtKind::Reassign { symbol, value } => {
                    bind(&mut bindings, *symbol, [value], &controls);
                    stack.push((Node::Expr(value), controls));
                }
                HirStmtKind::TupleDecl { symbols, value } => {
                    for symbol in symbols {
                        bind(&mut bindings, *symbol, [value], &controls);
                    }
                    stack.push((Node::Expr(value), controls));
                }
                HirStmtKind::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    stack.push((Node::Expr(condition), controls.clone()));
                    let mut branch = controls;
                    branch.push(condition);
                    if let HirExprKind::Literal(HirLiteral::Bool(value)) = condition.kind {
                        stack.extend(
                            (if value { then_branch } else { else_branch })
                                .iter()
                                .map(|statement| (Node::Stmt(statement), branch.clone())),
                        );
                    } else {
                        stack.extend(
                            then_branch
                                .iter()
                                .chain(else_branch)
                                .map(|statement| (Node::Stmt(statement), branch.clone())),
                        );
                    }
                }
                HirStmtKind::Switch { selector, arms } => {
                    let mut branch = controls.clone();
                    branch.extend(selector.as_ref());
                    for arm in arms {
                        let mut arm_controls = branch.clone();
                        arm_controls.extend(arm.condition.as_ref());
                        stack.extend(
                            arm.body
                                .iter()
                                .map(|statement| (Node::Stmt(statement), arm_controls.clone())),
                        );
                        if let Some(condition) = &arm.condition {
                            stack.push((Node::Expr(condition), controls.clone()));
                        }
                    }
                    if let Some(selector) = selector {
                        stack.push((Node::Expr(selector), controls));
                    }
                }
                HirStmtKind::For {
                    counter,
                    from,
                    to,
                    step,
                    body,
                } => {
                    let mut nested = controls.clone();
                    nested.extend([from, to]);
                    nested.extend(step.as_ref());
                    bind(&mut bindings, *counter, [from, to], &controls);
                    stack.extend(
                        body.iter()
                            .map(|statement| (Node::Stmt(statement), nested.clone())),
                    );
                    stack.extend(
                        [from, to]
                            .into_iter()
                            .chain(step.as_ref())
                            .map(|expr| (Node::Expr(expr), controls.clone())),
                    );
                }
                HirStmtKind::ForIn {
                    index,
                    value,
                    iterable,
                    body,
                } => {
                    for symbol in index.iter().chain(std::iter::once(value)) {
                        bind(&mut bindings, *symbol, [iterable], &controls);
                    }
                    let mut nested = controls.clone();
                    nested.push(iterable);
                    stack.extend(
                        body.iter()
                            .map(|statement| (Node::Stmt(statement), nested.clone())),
                    );
                    stack.push((Node::Expr(iterable), controls));
                }
                HirStmtKind::While { condition, body } => {
                    let mut nested = controls.clone();
                    nested.push(condition);
                    stack.extend(
                        body.iter()
                            .map(|statement| (Node::Stmt(statement), nested.clone())),
                    );
                    stack.push((Node::Expr(condition), controls));
                }
                HirStmtKind::Expr(value) | HirStmtKind::FieldReassign { value, .. } => {
                    stack.push((Node::Expr(value), controls))
                }
                HirStmtKind::ArrayFieldReassign {
                    array,
                    index,
                    value,
                    ..
                } => stack.extend(
                    [array, index, value]
                        .into_iter()
                        .map(|expr| (Node::Expr(expr), controls.clone())),
                ),
                HirStmtKind::Break | HirStmtKind::Continue => {}
            },
        }
    }
    bindings
}

fn expression_children(expr: &HirExpr) -> Vec<&HirExpr> {
    match &expr.kind {
        HirExprKind::Literal(_) | HirExprKind::Symbol(_) | HirExprKind::Builtin(_) => vec![],
        HirExprKind::Unary { expr, .. } | HirExprKind::FieldAccess { value: expr, .. } => {
            vec![expr]
        }
        HirExprKind::Binary { op, left, right } => {
            if let HirExprKind::Literal(HirLiteral::Bool(value)) = left.kind
                && ((*op == HirBinaryOp::And && !value) || (*op == HirBinaryOp::Or && value))
            {
                return vec![left];
            }
            vec![left, right]
        }
        HirExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            if let HirExprKind::Literal(HirLiteral::Bool(value)) = condition.kind {
                vec![condition, if value { then_expr } else { else_expr }]
            } else {
                vec![condition, then_expr, else_expr]
            }
        }
        HirExprKind::Block { result, .. } => vec![result],
        HirExprKind::Call { args, .. } => args.iter().map(|arg| &arg.value).collect(),
        HirExprKind::Tuple(items)
        | HirExprKind::UserTypeConstruct { fields: items, .. }
        | HirExprKind::UserTypeArrayConstruct {
            elements: items, ..
        } => items.iter().collect(),
        HirExprKind::History { expr, offset } => {
            let mut values = vec![expr.as_ref()];
            if let HirHistoryOffset::Dynamic(offset) = offset {
                values.push(offset);
            }
            values
        }
        HirExprKind::Switch { selector, arms } => selector
            .as_deref()
            .into_iter()
            .chain(arms.iter().flat_map(|arm| {
                arm.condition
                    .as_ref()
                    .into_iter()
                    .chain(std::iter::once(&arm.result))
            }))
            .collect(),
        HirExprKind::For {
            from,
            to,
            step,
            result,
            ..
        } => [from.as_ref(), to.as_ref(), result.as_ref()]
            .into_iter()
            .chain(step.as_deref())
            .collect(),
        HirExprKind::ForIn {
            iterable, result, ..
        } => vec![iterable, result],
        HirExprKind::While {
            condition, result, ..
        } => vec![condition, result],
    }
}

enum MutNode<'a> {
    Expr(&'a mut HirExpr),
    Stmt(&'a mut HirStmt),
}

fn strip_root_boundary(expr: &mut HirExpr) -> bool {
    let mut removed = false;
    while matches!(
        expr.kind,
        HirExprKind::Unary {
            op: HirUnaryOp::FunctionBoolReturn { .. },
            ..
        }
    ) {
        let outer_series = expr.series_id;
        let kind = std::mem::replace(
            &mut expr.kind,
            HirExprKind::Literal(HirLiteral::Bool(false)),
        );
        let HirExprKind::Unary { expr: child, .. } = kind else {
            unreachable!()
        };
        *expr = *child;
        if outer_series.is_some() {
            expr.series_id = outer_series;
        }
        removed = true;
    }
    removed
}

/// A program without constant identities must retain its previous runtime IR:
/// provenance-only UDF return nodes are removed before history inference.
pub(crate) fn strip_boolean_return_boundaries(statements: &mut [HirStmt]) {
    let mut stack = statements.iter_mut().map(MutNode::Stmt).collect::<Vec<_>>();
    while let Some(node) = stack.pop() {
        match node {
            MutNode::Expr(expr) => {
                strip_root_boundary(expr);
                if let HirExprKind::Block { statements, result } = &mut expr.kind {
                    while strip_root_boundary(result)
                        && matches!(result.kind, HirExprKind::Block { .. })
                    {
                        let kind = std::mem::replace(
                            &mut result.kind,
                            HirExprKind::Literal(HirLiteral::Bool(false)),
                        );
                        let HirExprKind::Block {
                            statements: nested,
                            result: nested_result,
                        } = kind
                        else {
                            unreachable!()
                        };
                        statements.extend(nested);
                        *result = nested_result;
                    }
                }
                match &mut expr.kind {
                    HirExprKind::Unary { expr, .. }
                    | HirExprKind::FieldAccess { value: expr, .. } => {
                        stack.push(MutNode::Expr(expr))
                    }
                    HirExprKind::Binary { left, right, .. } => {
                        stack.push(MutNode::Expr(left));
                        stack.push(MutNode::Expr(right));
                    }
                    HirExprKind::Ternary {
                        condition,
                        then_expr,
                        else_expr,
                    } => {
                        stack.push(MutNode::Expr(condition));
                        stack.push(MutNode::Expr(then_expr));
                        stack.push(MutNode::Expr(else_expr));
                    }
                    HirExprKind::Block { statements, result } => {
                        stack.extend(statements.iter_mut().map(MutNode::Stmt));
                        stack.push(MutNode::Expr(result));
                    }
                    HirExprKind::Call { args, .. } => {
                        stack.extend(args.iter_mut().map(|arg| MutNode::Expr(&mut arg.value)))
                    }
                    HirExprKind::Tuple(items)
                    | HirExprKind::UserTypeConstruct { fields: items, .. }
                    | HirExprKind::UserTypeArrayConstruct {
                        elements: items, ..
                    } => stack.extend(items.iter_mut().map(MutNode::Expr)),
                    HirExprKind::History { expr, offset } => {
                        stack.push(MutNode::Expr(expr));
                        if let HirHistoryOffset::Dynamic(offset) = offset {
                            stack.push(MutNode::Expr(offset));
                        }
                    }
                    HirExprKind::Switch { selector, arms } => {
                        if let Some(selector) = selector {
                            stack.push(MutNode::Expr(selector));
                        }
                        for arm in arms {
                            if let Some(condition) = &mut arm.condition {
                                stack.push(MutNode::Expr(condition));
                            }
                            stack.push(MutNode::Expr(&mut arm.result));
                        }
                    }
                    HirExprKind::For {
                        from,
                        to,
                        step,
                        statements,
                        result,
                        ..
                    } => {
                        stack.push(MutNode::Expr(from));
                        stack.push(MutNode::Expr(to));
                        if let Some(step) = step {
                            stack.push(MutNode::Expr(step));
                        }
                        stack.extend(statements.iter_mut().map(MutNode::Stmt));
                        stack.push(MutNode::Expr(result));
                    }
                    HirExprKind::ForIn {
                        iterable,
                        statements,
                        result,
                        ..
                    } => {
                        stack.push(MutNode::Expr(iterable));
                        stack.extend(statements.iter_mut().map(MutNode::Stmt));
                        stack.push(MutNode::Expr(result));
                    }
                    HirExprKind::While {
                        condition,
                        statements,
                        result,
                    } => {
                        stack.push(MutNode::Expr(condition));
                        stack.extend(statements.iter_mut().map(MutNode::Stmt));
                        stack.push(MutNode::Expr(result));
                    }
                    HirExprKind::Literal(_) | HirExprKind::Symbol(_) | HirExprKind::Builtin(_) => {}
                }
            }
            MutNode::Stmt(statement) => match &mut statement.kind {
                HirStmtKind::Expr(value)
                | HirStmtKind::Decl { value, .. }
                | HirStmtKind::Reassign { value, .. }
                | HirStmtKind::FieldReassign { value, .. }
                | HirStmtKind::TupleDecl { value, .. } => stack.push(MutNode::Expr(value)),
                HirStmtKind::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    stack.push(MutNode::Expr(condition));
                    stack.extend(then_branch.iter_mut().chain(else_branch).map(MutNode::Stmt));
                }
                HirStmtKind::Switch { selector, arms } => {
                    if let Some(selector) = selector {
                        stack.push(MutNode::Expr(selector));
                    }
                    for arm in arms {
                        if let Some(condition) = &mut arm.condition {
                            stack.push(MutNode::Expr(condition));
                        }
                        stack.extend(arm.body.iter_mut().map(MutNode::Stmt));
                    }
                }
                HirStmtKind::For {
                    from,
                    to,
                    step,
                    body,
                    ..
                } => {
                    stack.push(MutNode::Expr(from));
                    stack.push(MutNode::Expr(to));
                    if let Some(step) = step {
                        stack.push(MutNode::Expr(step));
                    }
                    stack.extend(body.iter_mut().map(MutNode::Stmt));
                }
                HirStmtKind::ForIn { iterable, body, .. } => {
                    stack.push(MutNode::Expr(iterable));
                    stack.extend(body.iter_mut().map(MutNode::Stmt));
                }
                HirStmtKind::While { condition, body } => {
                    stack.push(MutNode::Expr(condition));
                    stack.extend(body.iter_mut().map(MutNode::Stmt));
                }
                HirStmtKind::ArrayFieldReassign {
                    array,
                    index,
                    value,
                    ..
                } => {
                    stack.push(MutNode::Expr(array));
                    stack.push(MutNode::Expr(index));
                    stack.push(MutNode::Expr(value));
                }
                HirStmtKind::Break | HirStmtKind::Continue => {}
            },
        }
    }
}
