use std::collections::{HashMap, HashSet};

use crate::prelude::*;

/// Only expressions whose result depends on an unbound generic parameter are
/// deferred. Structural name/arity errors remain checkable independently;
/// type constraints on sibling arguments defer with the whole generic call.
pub(super) struct DeferredExpressions {
    spans: HashSet<(usize, usize, usize)>,
    type_spans: HashSet<(usize, usize, usize)>,
    targets: HashSet<(usize, usize, usize)>,
}

struct DependencyWalker<'a> {
    spans: HashSet<(usize, usize, usize)>,
    type_spans: HashSet<(usize, usize, usize)>,
    targets: HashSet<(usize, usize, usize)>,
    visited: HashSet<(String, usize, Vec<bool>)>,
    functions: &'a HashMap<String, FunctionInfo>,
    methods: &'a HashMap<(String, String), MethodInfo>,
    source_id: usize,
    depth: usize,
    scopes: Vec<HashMap<String, bool>>,
}

impl DeferredExpressions {
    pub(super) fn for_body(
        body: &FunctionBody,
        names: HashSet<String>,
        functions: &HashMap<String, FunctionInfo>,
        methods: &HashMap<(String, String), MethodInfo>,
        source_id: usize,
    ) -> Self {
        let mut result = DependencyWalker {
            scopes: vec![names.into_iter().map(|n| (n, true)).collect()],
            spans: HashSet::new(),
            type_spans: HashSet::new(),
            targets: HashSet::new(),
            visited: HashSet::new(),
            functions,
            methods,
            source_id,
            depth: 0,
        };
        match body {
            FunctionBody::Expr(expr) => {
                result.expr(expr);
            }
            FunctionBody::Block(statements) => {
                result.statements(statements);
            }
        }
        Self {
            spans: result.spans,
            type_spans: result.type_spans,
            targets: result.targets,
        }
    }

    pub(super) fn defers(&self, diagnostic: &Diagnostic) -> bool {
        let source = diagnostic.source.as_ref().map_or(0, |s| s.source_id);
        let span = diagnostic.span;
        let key = (source, span.start, span.end);
        match diagnostic.code.as_str() {
            "E_FUNCTION_PARAM"
            | "E_CONST_REASSIGN"
            | "E_LOOP_CONTROL"
            | "E_METHOD_DECL_LOCATION"
            | "E_RECURSIVE_FUNCTION"
            | "E_RECURSIVE_METHOD"
            | "E_FUNCTION_CALL_DEPTH"
            | "E_CALL_ARG_ORDER"
            | "E_CALL_ARG_NAME" => false,
            "E_UNKNOWN_FUNCTION" | "E_UNKNOWN_METHOD" | "E_CALL_TARGET" | "E_CALL_ARITY" => {
                self.targets.contains(&key)
            }
            _ => self.spans.contains(&key) || self.type_spans.contains(&key),
        }
    }
}

impl DependencyWalker<'_> {
    fn record(&mut self, span: Span, deferred: bool) -> bool {
        if deferred {
            self.spans.insert((self.source_id, span.start, span.end));
        }
        deferred
    }

    fn lookup(&self, name: &str) -> bool {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.get(name))
            .copied()
            .unwrap_or(false)
    }

    fn define(&mut self, name: &str, deferred: bool) {
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_owned(), deferred);
    }

    fn assign(&mut self, name: &str, deferred: bool) {
        if let Some(scope) = self.scopes.iter_mut().rev().find(|s| s.contains_key(name)) {
            // A branch/loop may execute zero times. Retain both possibilities.
            *scope.get_mut(name).unwrap() |= deferred;
        }
    }

    fn branch(&mut self, statements: &[Stmt]) -> bool {
        self.scopes.push(HashMap::new());
        let result = self.statements(statements);
        self.scopes.pop();
        result
    }

    fn statements(&mut self, statements: &[Stmt]) -> bool {
        let mut result = false;
        for stmt in statements {
            result = self.statement(stmt);
        }
        result
    }

    fn statement(&mut self, statement: &Stmt) -> bool {
        let deferred = match &statement.kind {
            StmtKind::Expr(expr) => self.expr(expr),
            StmtKind::Decl { name, value, .. } => {
                let value = self.expr(value);
                self.define(name, value);
                value
            }
            StmtKind::TupleDecl { names, value } => {
                let value = self.expr(value);
                for name in names {
                    self.define(name, value);
                }
                value
            }
            StmtKind::Reassign { name, value } => {
                let previous = self.lookup(name);
                let value = self.expr(value);
                self.assign(name, value);
                previous | value
            }
            StmtKind::FieldReassign {
                receiver, value, ..
            } => {
                let value = self.expr(value);
                self.lookup(receiver) | value
            }
            StmtKind::ArrayFieldReassign {
                array,
                index,
                value,
                ..
            } => self.expr(array) | self.expr(index) | self.expr(value),
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.expr(condition) | self.branch(then_branch) | self.branch(else_branch),
            StmtKind::While { condition, body } => self.expr(condition) | self.branch(body),
            StmtKind::For {
                counter,
                from,
                to,
                step,
                body,
            } => self.for_loop(counter, from, to, step.as_ref(), body),
            StmtKind::ForIn {
                index,
                value,
                iterable,
                body,
            } => self.for_in(index.as_deref(), value, iterable, body),
            _ => false,
        };
        self.record(statement.span, deferred)
    }

    fn for_loop(
        &mut self,
        counter: &str,
        from: &Expr,
        to: &Expr,
        step: Option<&Expr>,
        body: &[Stmt],
    ) -> bool {
        let bounds = self.expr(from) | self.expr(to) | step.is_some_and(|s| self.expr(s));
        self.scopes.push(HashMap::new());
        self.define(counter, bounds);
        let result = self.statements(body);
        self.scopes.pop();
        bounds | result
    }

    fn for_in(&mut self, index: Option<&str>, value: &str, iterable: &Expr, body: &[Stmt]) -> bool {
        let iterable = self.expr(iterable);
        self.scopes.push(HashMap::new());
        if let Some(index) = index {
            self.define(index, iterable);
        }
        self.define(value, iterable);
        let result = self.statements(body);
        self.scopes.pop();
        iterable | result
    }

    fn expr(&mut self, expression: &Expr) -> bool {
        let deferred = match &expression.kind {
            ExprKind::Identifier(name) => self.lookup(name),
            ExprKind::QualifiedName(path) => path.first().is_some_and(|n| self.lookup(n)),
            ExprKind::Literal(_) => false,
            ExprKind::Unary { expr, .. }
            | ExprKind::Group(expr)
            | ExprKind::Member { receiver: expr, .. } => self.expr(expr),
            ExprKind::Binary { left, right, .. } => self.expr(left) | self.expr(right),
            ExprKind::History { expr, offset } => self.expr(expr) | self.expr(offset),
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => self.expr(condition) | self.expr(then_expr) | self.expr(else_expr),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.expr(condition) | self.branch(then_branch) | self.branch(else_branch),
            ExprKind::While { condition, body } => self.expr(condition) | self.branch(body),
            ExprKind::For {
                counter,
                from,
                to,
                step,
                body,
            } => self.for_loop(counter, from, to, step.as_deref(), body),
            ExprKind::ForIn {
                index,
                value,
                iterable,
                body,
            } => self.for_in(index.as_deref(), value, iterable, body),
            ExprKind::Tuple(items) => {
                let mut result = false;
                for item in items {
                    result |= self.expr(item);
                }
                result
            }
            ExprKind::Switch { selector, arms } => {
                let mut result = selector.as_deref().is_some_and(|s| self.expr(s));
                for arm in arms {
                    result |= arm.condition.as_ref().is_some_and(|e| self.expr(e));
                    result |= match &arm.result {
                        SwitchArmResult::Expr(e) => self.expr(e),
                        SwitchArmResult::Block(b) => self.branch(b),
                    };
                }
                result
            }
            ExprKind::Call { callee, args } => {
                let target = self.expr(callee);
                let mut result = target;
                if target {
                    self.targets
                        .insert((self.source_id, callee.span.start, callee.span.end));
                    self.targets.insert((
                        self.source_id,
                        expression.span.start,
                        expression.span.end,
                    ));
                }
                let mut arguments = Vec::new();
                for arg in args {
                    let value = self.expr(&arg.value);
                    self.record(arg.span, value);
                    result |= value;
                    arguments.push(value);
                }
                if result {
                    self.type_spans
                        .insert((self.source_id, callee.span.start, callee.span.end));
                    // Overload/type resolution of a call can depend on any
                    // generic argument, including constraints on its siblings.
                    for arg in args {
                        self.type_spans
                            .insert((self.source_id, arg.span.start, arg.span.end));
                        self.type_spans.insert((
                            self.source_id,
                            arg.value.span.start,
                            arg.value.span.end,
                        ));
                    }
                    self.called_body(callee, args, &arguments, target);
                }
                result
            }
        };
        self.record(expression.span, deferred)
    }

    fn called_body(&mut self, callee: &Expr, args: &[CallArg], arguments: &[bool], target: bool) {
        if self.depth >= MAX_FUNCTION_CALL_DEPTH {
            return;
        }
        let name = expr_name(callee);
        let function = name
            .as_ref()
            .and_then(|name| self.functions.get(name))
            .cloned();
        let Some(function) = function else {
            self.called_methods(callee, args, arguments, target);
            return;
        };
        let name = name.unwrap();
        for function in std::iter::once(&function).chain(function.overloads.iter()) {
            let Ok(indices) = crate::analyzer::functions::resolve_udf_arg_indices_with_defaults(
                &function.params,
                args,
                &function.default_values,
            ) else {
                continue;
            };
            let mut unknown = HashMap::new();
            for (argument, index) in arguments.iter().zip(indices) {
                unknown.insert(function.params[index].clone(), *argument);
                if *argument && let Some(Some(binding)) = function.param_types.get(index) {
                    self.type_spans.insert((
                        function.source_id.get(),
                        binding.span.start,
                        binding.span.end,
                    ));
                }
            }
            self.visit_body(
                &name,
                function.span,
                function.source_id.get(),
                &function.params,
                &function.body,
                unknown,
            );
        }
    }

    fn called_methods(
        &mut self,
        callee: &Expr,
        args: &[CallArg],
        arguments: &[bool],
        target: bool,
    ) {
        let (name, member_call) = match &callee.kind {
            ExprKind::Identifier(name) => (name.as_str(), false),
            ExprKind::QualifiedName(path) => match path.last() {
                Some(name) => (name.as_str(), path.len() > 1),
                None => return,
            },
            ExprKind::Member { name, .. } => (name.as_str(), true),
            _ => return,
        };
        let candidates: Vec<_> = self
            .methods
            .iter()
            .filter(|((_, method_name), _)| method_name == name)
            .map(|(_, method)| method.clone())
            .collect();
        for method in candidates {
            let (args, arguments, receiver) = if member_call {
                (args, arguments, target)
            } else {
                let Some((receiver, arguments)) = arguments.split_first() else {
                    continue;
                };
                (&args[1..], arguments, *receiver)
            };
            let params: Vec<_> = method
                .params
                .iter()
                .map(|param| param.name.clone())
                .collect();
            let Ok(indices) = crate::analyzer::functions::resolve_udf_arg_indices(&params, args)
            else {
                continue;
            };
            let mut unknown = HashMap::from([(method.receiver_name.clone(), receiver)]);
            for (argument, index) in arguments.iter().zip(indices) {
                unknown.insert(params[index].clone(), *argument);
            }
            let params: Vec<_> = std::iter::once(method.receiver_name.clone())
                .chain(params)
                .collect();
            // A generic receiver can have multiple applicable specializations.
            // Visit each matching body; unrelated closed errors are still checked
            // in that body's own declaration pass. No overload is selected here.
            self.visit_body(
                &format!("method:{}.{}", method.receiver_type, name),
                method.span,
                method.source_id.get(),
                &params,
                &method.body,
                unknown,
            );
        }
    }

    fn visit_body(
        &mut self,
        name: &str,
        span: Span,
        source_id: usize,
        params: &[String],
        body: &FunctionBody,
        unknown: HashMap<String, bool>,
    ) {
        let mask = params
            .iter()
            .map(|p| unknown.get(p).copied().unwrap_or(false))
            .collect();
        if !self.visited.insert((name.to_owned(), span.start, mask)) {
            return;
        }
        let previous_source = std::mem::replace(&mut self.source_id, source_id);
        let previous_scopes = std::mem::replace(&mut self.scopes, vec![unknown]);
        self.depth += 1;
        match body {
            FunctionBody::Expr(expr) => {
                self.expr(expr);
            }
            FunctionBody::Block(body) => {
                self.statements(body);
            }
        }
        self.depth -= 1;
        self.scopes = previous_scopes;
        self.source_id = previous_source;
    }
}
