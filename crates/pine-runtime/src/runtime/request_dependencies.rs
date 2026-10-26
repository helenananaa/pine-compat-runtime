//! Declarations that requested contexts reconstruct dependencies from.
//!
//! The index is built once per program and shared by every runtime and fork
//! that executes it. HIR nodes have no ids and a prepared program is shared
//! without a borrow, so each declaration is stored as its child positions
//! from `program.statements`.
use std::collections::HashMap;

use pine_ir::{
    HirExpr, HirExprKind, HirHistoryOffset, HirProgram, HirStmt, HirStmtKind, HirSymbol, SymbolId,
};

use super::metadata::RuntimeMetadata;

#[derive(Debug, Default)]
pub(crate) struct RequestDependencyIndex {
    /// Declared value per symbol, including literal tuple items.
    initializers: HashMap<SymbolId, Box<[usize]>>,
    /// Tuple declaration per declared symbol.
    tuples: HashMap<SymbolId, Box<[usize]>>,
}

impl RequestDependencyIndex {
    /// Matches `request_dependency_initializers` and
    /// `request_tuple_dependency_statements`, including later declarations
    /// replacing earlier ones.
    pub(crate) fn from_program(program: &HirProgram) -> Self {
        let mut index = Self::default();
        let mut path = Vec::new();
        for (position, statement) in program.statements.iter().enumerate() {
            path.push(position);
            index.record(Node::Stmt(statement), &mut path);
            path.pop();
        }
        index
    }

    fn record(&mut self, node: Node<'_>, path: &mut Vec<usize>) {
        if let Node::Stmt(statement) = node {
            match &statement.kind {
                HirStmtKind::Decl { symbol, .. } => {
                    self.initializers.insert(*symbol, child_path(path, &[0]));
                }
                HirStmtKind::TupleDecl { symbols, value } => {
                    if let HirExprKind::Tuple(items) = &value.kind {
                        for (item, symbol) in symbols.iter().take(items.len()).enumerate() {
                            self.initializers
                                .insert(*symbol, child_path(path, &[0, item]));
                        }
                    }
                    for symbol in symbols {
                        self.tuples.insert(*symbol, path.as_slice().into());
                    }
                }
                _ => {}
            }
        }
        let mut position = 0;
        for_each_child(node, &mut |child| {
            path.push(position);
            self.record(child, path);
            path.pop();
            position += 1;
        });
    }
}

/// Request dependencies resolved against the program they were indexed from.
#[derive(Clone, Copy)]
pub(crate) struct RequestDependencies<'p> {
    program: &'p HirProgram,
    metadata: &'p RuntimeMetadata,
    index: &'p RequestDependencyIndex,
}

impl<'p> RequestDependencies<'p> {
    pub(super) fn new(
        program: &'p HirProgram,
        metadata: &'p RuntimeMetadata,
        index: &'p RequestDependencyIndex,
    ) -> Self {
        Self {
            program,
            metadata,
            index,
        }
    }

    pub(crate) fn initializer(self, symbol: SymbolId) -> Option<&'p HirExpr> {
        match self.resolve(self.index.initializers.get(&symbol)?)? {
            Node::Expr(expr) => Some(expr),
            Node::Stmt(_) => None,
        }
    }

    pub(crate) fn tuple_statement(self, symbol: SymbolId) -> Option<&'p HirStmt> {
        match self.resolve(self.index.tuples.get(&symbol)?)? {
            Node::Stmt(statement) => Some(statement),
            Node::Expr(_) => None,
        }
    }

    /// The first program symbol with this id.
    pub(crate) fn symbol(self, symbol: SymbolId) -> Option<&'p HirSymbol> {
        self.program
            .symbols
            .get(self.metadata.symbol_index(symbol)?)
    }

    fn resolve(self, path: &[usize]) -> Option<Node<'p>> {
        let (first, rest) = path.split_first()?;
        let mut node = Node::Stmt(self.program.statements.get(*first)?);
        for &position in rest {
            node = nth_child(node, position)?;
        }
        Some(node)
    }
}

fn child_path(path: &[usize], suffix: &[usize]) -> Box<[usize]> {
    path.iter().chain(suffix).copied().collect()
}

#[derive(Clone, Copy)]
enum Node<'p> {
    Stmt(&'p HirStmt),
    Expr(&'p HirExpr),
}

fn nth_child(node: Node<'_>, position: usize) -> Option<Node<'_>> {
    let mut seen = 0;
    let mut found = None;
    for_each_child(node, &mut |child| {
        if seen == position {
            found = Some(child);
        }
        seen += 1;
    });
    found
}

fn visit_statements<'p>(statements: &'p [HirStmt], visit: &mut impl FnMut(Node<'p>)) {
    for statement in statements {
        visit(Node::Stmt(statement));
    }
}

/// Every child statement and expression, in the request collectors' order.
fn for_each_child<'p>(node: Node<'p>, visit: &mut impl FnMut(Node<'p>)) {
    match node {
        Node::Stmt(statement) => match &statement.kind {
            HirStmtKind::Expr(expr)
            | HirStmtKind::Decl { value: expr, .. }
            | HirStmtKind::Reassign { value: expr, .. }
            | HirStmtKind::FieldReassign { value: expr, .. }
            | HirStmtKind::TupleDecl { value: expr, .. } => visit(Node::Expr(expr)),
            HirStmtKind::ArrayFieldReassign {
                array,
                index,
                value,
                ..
            } => {
                for expr in [array, index, value] {
                    visit(Node::Expr(expr));
                }
            }
            HirStmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                visit(Node::Expr(condition));
                visit_statements(then_branch, visit);
                visit_statements(else_branch, visit);
            }
            HirStmtKind::Switch { selector, arms } => {
                if let Some(selector) = selector {
                    visit(Node::Expr(selector));
                }
                for arm in arms {
                    if let Some(condition) = &arm.condition {
                        visit(Node::Expr(condition));
                    }
                    visit_statements(&arm.body, visit);
                }
            }
            HirStmtKind::For {
                from,
                to,
                step,
                body,
                ..
            } => {
                visit(Node::Expr(from));
                visit(Node::Expr(to));
                if let Some(step) = step {
                    visit(Node::Expr(step));
                }
                visit_statements(body, visit);
            }
            HirStmtKind::ForIn { iterable, body, .. } => {
                visit(Node::Expr(iterable));
                visit_statements(body, visit);
            }
            HirStmtKind::While { condition, body } => {
                visit(Node::Expr(condition));
                visit_statements(body, visit);
            }
            HirStmtKind::Break | HirStmtKind::Continue => {}
        },
        Node::Expr(expr) => match &expr.kind {
            HirExprKind::Literal(_) | HirExprKind::Builtin(_) | HirExprKind::Symbol(_) => {}
            HirExprKind::Unary { expr, .. } | HirExprKind::FieldAccess { value: expr, .. } => {
                visit(Node::Expr(expr));
            }
            HirExprKind::Binary { left, right, .. } => {
                visit(Node::Expr(left));
                visit(Node::Expr(right));
            }
            HirExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                for expr in [condition, then_expr, else_expr] {
                    visit(Node::Expr(expr));
                }
            }
            HirExprKind::Switch { selector, arms } => {
                if let Some(selector) = selector {
                    visit(Node::Expr(selector));
                }
                for arm in arms {
                    if let Some(condition) = &arm.condition {
                        visit(Node::Expr(condition));
                    }
                    visit(Node::Expr(&arm.result));
                }
            }
            HirExprKind::For {
                from,
                to,
                step,
                statements: body,
                result,
                ..
            } => {
                visit(Node::Expr(from));
                visit(Node::Expr(to));
                if let Some(step) = step {
                    visit(Node::Expr(step));
                }
                visit_statements(body, visit);
                visit(Node::Expr(result));
            }
            HirExprKind::ForIn {
                iterable,
                statements: body,
                result,
                ..
            } => {
                visit(Node::Expr(iterable));
                visit_statements(body, visit);
                visit(Node::Expr(result));
            }
            HirExprKind::While {
                condition,
                statements: body,
                result,
            } => {
                visit(Node::Expr(condition));
                visit_statements(body, visit);
                visit(Node::Expr(result));
            }
            HirExprKind::Tuple(values)
            | HirExprKind::UserTypeConstruct { fields: values, .. }
            | HirExprKind::UserTypeArrayConstruct {
                elements: values, ..
            } => {
                for value in values {
                    visit(Node::Expr(value));
                }
            }
            HirExprKind::Block {
                statements: body,
                result,
            } => {
                visit_statements(body, visit);
                visit(Node::Expr(result));
            }
            HirExprKind::Call { args, .. } => {
                for arg in args {
                    visit(Node::Expr(&arg.value));
                }
            }
            HirExprKind::History { expr, offset } => {
                visit(Node::Expr(expr));
                if let HirHistoryOffset::Dynamic(offset) = offset {
                    visit(Node::Expr(offset));
                }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::requests::{
        request_dependency_initializers, request_tuple_dependency_statements,
    };
    use crate::retention::SeriesRetention;

    #[test]
    fn index_resolves_the_declarations_the_request_collectors_find() {
        let nested = r#"//@version=6
indicator("dependencies")
pair(float value) =>
    float doubled = value * 2
    [doubled, value]
scaled(float value) =>
    float result = value
    if value > 1
        float bump = value + 1
        result := bump
    result
[a, b] = pair(close)
[up, down] = [close > open ? volume : 0.0, close < open ? -volume : 0.0]
float total = 0.0
for i = 0 to 2
    float step = i * close
    total += step
float picked = switch
    close > open => scaled(close)
    => open
if close > 0
    float inner = close * 3
    total += inner
float countdown = 3
while countdown > 0
    float half = countdown / 2
    countdown -= 1
plot(a + b + up + down + total + picked + countdown)
"#;
        let shared = "//@version=6\nindicator(\"shared\")\nx0 = close\nx1 = x0 + x0\nx2 = x1 + x1\nplot(request.security(\"B\", \"1\", x2))\n";
        for source in [nested, shared] {
            let analysis =
                pine_sema::analyze_source(&pine_syntax::SourceFile::new("index.pine", source));
            assert!(
                analysis.diagnostics.is_empty(),
                "{:?}",
                analysis.diagnostics
            );
            let program = analysis.hir.unwrap();
            let metadata =
                RuntimeMetadata::from_program(&program, &SeriesRetention::from_program(&program));
            let dependencies = metadata.request_dependencies(&program);
            let initializers = request_dependency_initializers(&program);
            let tuples = request_tuple_dependency_statements(&program);
            assert_eq!(dependencies.index.initializers.len(), initializers.len());
            assert_eq!(dependencies.index.tuples.len(), tuples.len());
            for symbol in &program.symbols {
                assert_eq!(
                    dependencies.initializer(symbol.id).map(std::ptr::from_ref),
                    initializers
                        .get(&symbol.id)
                        .map(|value| std::ptr::from_ref(*value)),
                    "{}",
                    symbol.name
                );
                assert_eq!(
                    dependencies
                        .tuple_statement(symbol.id)
                        .map(std::ptr::from_ref),
                    tuples
                        .get(&symbol.id)
                        .map(|statement| std::ptr::from_ref(*statement)),
                    "{}",
                    symbol.name
                );
                assert!(std::ptr::eq(
                    dependencies.symbol(symbol.id).unwrap(),
                    program
                        .symbols
                        .iter()
                        .find(|candidate| candidate.id == symbol.id)
                        .unwrap()
                ));
            }
        }
    }
}
