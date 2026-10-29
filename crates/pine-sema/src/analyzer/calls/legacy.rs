use std::collections::HashSet;

use crate::prelude::*;
use crate::source_graph::SourceContextId;

/// A part of the program that a marker search still has to inspect, read in
/// the source context that owns it.
#[derive(Clone, Copy)]
enum LegacyInputSearchNode<'a> {
    Expr(&'a Expr),
    Statement(&'a Stmt),
    ReturnStatements(&'a [Stmt]),
}

struct LegacyInputSearch<'a> {
    // An explicit stack keeps alias and call-result chains off the thread
    // stack; a long straight-line script would otherwise overflow it.
    pending: Vec<(SourceContextId, LegacyInputSearchNode<'a>)>,
    // Marker containment is a reachability query: an initializer or function
    // result already searched by this query cannot reveal a marker on another
    // path, so a shared dependency graph is searched once rather than per path.
    expanded_symbols: HashSet<SymbolId>,
    expanded_functions: HashSet<&'a str>,
}

impl<'a> LegacyInputSearch<'a> {
    fn push(&mut self, context: SourceContextId, node: LegacyInputSearchNode<'a>) {
        self.pending.push((context, node));
    }

    fn push_expr(&mut self, context: SourceContextId, expr: &'a Expr) {
        self.push(context, LegacyInputSearchNode::Expr(expr));
    }

    fn push_statements(&mut self, context: SourceContextId, statements: &'a [Stmt]) {
        for statement in statements {
            self.push(context, LegacyInputSearchNode::Statement(statement));
        }
    }

    fn push_return_statements(&mut self, context: SourceContextId, statements: &'a [Stmt]) {
        self.push(context, LegacyInputSearchNode::ReturnStatements(statements));
    }

    /// A nested declaration contributes every statement of its body, unlike a
    /// called function, which contributes only its result.
    fn push_function_body(&mut self, context: SourceContextId, body: &'a FunctionBody) {
        match body {
            FunctionBody::Expr(expr) => self.push_expr(context, expr),
            FunctionBody::Block(statements) => self.push_statements(context, statements),
        }
    }
}

#[cfg(test)]
thread_local! {
    static INITIALIZER_SEARCHES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(super) enum FocusedLegacyCallAnalysis {
    NotApplicable,
    Analyzed(Option<PineType>),
}

impl Analyzer {
    pub(super) fn reject_legacy_builtin_method_syntax(
        &mut self,
        receiver: &str,
        method: &str,
        span: Span,
    ) -> bool {
        if self.legacy.dialect().version() >= 5 {
            return false;
        }
        self.reject_unavailable_legacy_builtin(&format!("{receiver}.{method}"), 5, span);
        true
    }

    pub(super) fn reject_legacy_named_call_args(
        &mut self,
        name: &str,
        args: &[CallArg],
        callee_span: Span,
    ) -> bool {
        let dialect = self.legacy.dialect();
        let is_pre_v3 = matches!(dialect, crate::PineDialect::V1 | crate::PineDialect::V2);
        let is_udf = self.functions.contains_key(name);
        let rejects_legacy_udf_keywords = dialect.is_legacy() && is_udf;
        if !is_pre_v3 && !rejects_legacy_udf_keywords {
            return false;
        }
        let Some(named_arg) = args.iter().find(|arg| arg.name.is_some()) else {
            return false;
        };
        if rejects_legacy_udf_keywords {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_NAME",
                format!(
                    "named arguments are not supported by Pine {} user-defined function `{name}`; legacy user-defined function calls require positional arguments",
                    dialect.name(),
                ),
                named_arg.span.merge(callee_span),
            ));
            return true;
        }

        let resolution = self.legacy.resolve_call(name);
        let is_symbol_shadowed = self.lexical_symbol_shadows_legacy_call(name, callee_span);
        let is_annotation = !is_symbol_shadowed
            && (name == "alertcondition"
                || name == "strategy"
                || name.starts_with("strategy.")
                || matches!(
                    resolution,
                    Some(
                        crate::legacy::LegacyResolution::Focused(crate::legacy::LegacyRule {
                            kind: crate::legacy::LegacyRuleKind::FocusedDeclaration
                                | crate::legacy::LegacyRuleKind::FocusedInput
                                | crate::legacy::LegacyRuleKind::FocusedOutput,
                            ..
                        }) | crate::legacy::LegacyResolution::UnsupportedKnown(
                            crate::legacy::LegacyRule {
                                kind: crate::legacy::LegacyRuleKind::FocusedOutput,
                                ..
                            }
                        )
                    )
                ));
        let is_builtin = !is_symbol_shadowed
            && (resolution.is_some() || pine_builtins::is_phase_1_builtin(name));
        if !is_builtin || is_annotation {
            return false;
        }

        self.diagnostics.push(Diagnostic::error(
            "E_CALL_ARG_NAME",
            format!(
                "named arguments are not supported by Pine {} ordinary built-in `{name}`; before Pine v3 they are limited to annotation calls",
                dialect.name(),
            ),
            named_arg.span.merge(callee_span),
        ));
        true
    }

    pub(super) fn reject_legacy_input_constant_leaks(
        &mut self,
        name: &str,
        callee: &Expr,
        args: &[CallArg],
    ) -> bool {
        let allowed_type_span = self
            .is_unshadowed_focused_legacy_input_call(name, callee)
            .then(|| self.legacy.explicit_legacy_input_type_expr(args))
            .flatten()
            .map(|expr| expr.span);

        for arg in args {
            let is_exact_selector = self.legacy_input_type_marker(&arg.value).is_some();
            let is_allowed_selector =
                is_exact_selector && allowed_type_span == Some(arg.value.span);
            if self.legacy_input_constant_in_expr(&arg.value) && !is_allowed_selector {
                self.report_legacy_input_constant_context(
                    "legacy input type constants are only valid as the `type` selector of the versioned `input` annotation",
                    arg.value.span,
                );
                return true;
            }
        }
        false
    }

    pub(crate) fn reject_legacy_input_constant_reassignment(&mut self, value: &Expr) -> bool {
        self.reject_legacy_input_constant_semantic_use(
            value,
            "legacy input type constants cannot be reassigned; keep an exact const alias for the `type` selector of the versioned `input` annotation",
        )
    }

    pub(crate) fn reject_legacy_input_constant_symbol_reassignment(
        &mut self,
        name: &str,
        value: &Expr,
        span: Span,
    ) -> bool {
        let target_contains_marker = self.scope.resolve(name).is_some_and(|symbol| {
            self.symbol_init_exprs
                .get(&symbol.id)
                .is_some_and(|initializer| {
                    self.with_source_context_ref(initializer.source_context_id, |analyzer| {
                        analyzer.legacy_input_constant_in_expr(&initializer.expr)
                    })
                })
        });
        if !target_contains_marker && !self.legacy_input_constant_in_expr(value) {
            return false;
        }
        self.report_legacy_input_constant_context(
            "legacy input type constant aliases cannot be reassigned; keep an exact const alias for the `type` selector of the versioned `input` annotation",
            span,
        );
        true
    }

    pub(crate) fn reject_legacy_input_constant_declaration(&mut self, value: &Expr) -> bool {
        if self.legacy_input_type_marker(value).is_some() {
            return false;
        }
        self.reject_legacy_input_constant_semantic_use(
            value,
            "legacy input type constants may only be preserved as an exact const alias for the `type` selector of the versioned `input` annotation",
        )
    }

    pub(crate) fn reject_legacy_input_constant_expression(&mut self, expr: &Expr) -> bool {
        self.reject_legacy_input_constant_semantic_use(
            expr,
            "legacy input type constants cannot be used as ordinary expression values; they are only valid as the `type` selector of the versioned `input` annotation",
        )
    }

    fn reject_legacy_input_constant_semantic_use(
        &mut self,
        expr: &Expr,
        message: &'static str,
    ) -> bool {
        if !self.legacy_input_constant_in_expr(expr) {
            return false;
        }
        self.report_legacy_input_constant_context(message, expr.span);
        true
    }

    fn report_legacy_input_constant_context(&mut self, message: &'static str, span: Span) {
        if self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_LEGACY_INPUT_CONSTANT_CONTEXT")
        {
            return;
        }
        self.diagnostics.push(Diagnostic::error(
            "E_LEGACY_INPUT_CONSTANT_CONTEXT",
            message,
            span,
        ));
    }

    fn legacy_input_constant_in_expr<'a>(&'a self, expr: &'a Expr) -> bool {
        // Only a recorded legacy string value can be a marker. Scripts that
        // record none, which includes ordinary v5 and v6 scripts, skip the
        // search instead of walking every initializer chain per declaration.
        if !self.legacy.has_canonical_string_values() {
            return false;
        }
        let mut search = LegacyInputSearch {
            pending: Vec::new(),
            expanded_symbols: HashSet::new(),
            expanded_functions: HashSet::new(),
        };
        search.push_expr(self.current_source_context_id(), expr);
        while let Some((context, node)) = search.pending.pop() {
            let first_child = search.pending.len();
            let is_marker = self.with_source_context_ref(context, |_| {
                self.expand_legacy_input_search_node(context, node, &mut search)
            });
            if is_marker {
                return true;
            }
            // Children were pushed in source order; visit them in that order.
            search.pending[first_child..].reverse();
        }
        false
    }

    /// Expands one node in the current source context and reports whether the
    /// node itself is a marker. Nested parts are queued instead of searched.
    fn expand_legacy_input_search_node<'a>(
        &'a self,
        context: SourceContextId,
        node: LegacyInputSearchNode<'a>,
        search: &mut LegacyInputSearch<'a>,
    ) -> bool {
        match node {
            LegacyInputSearchNode::Expr(expr) => {
                self.expand_legacy_input_search_expr(context, expr, search)
            }
            LegacyInputSearchNode::Statement(statement) => {
                self.expand_legacy_input_search_statement(context, statement, search);
                false
            }
            LegacyInputSearchNode::ReturnStatements(statements) => {
                Self::expand_legacy_input_search_return(context, statements, search);
                false
            }
        }
    }

    fn expand_legacy_input_search_expr<'a>(
        &'a self,
        context: SourceContextId,
        expr: &'a Expr,
        search: &mut LegacyInputSearch<'a>,
    ) -> bool {
        // An exact alias of a marker is reached through the identifier
        // expansion below, so checking the node itself is enough here.
        if self
            .legacy
            .canonical_string_value(context, expr.span)
            .is_some()
        {
            return true;
        }
        match &expr.kind {
            ExprKind::Unary { expr, .. }
            | ExprKind::Group(expr)
            | ExprKind::Member { receiver: expr, .. } => search.push_expr(context, expr),
            ExprKind::Binary { left, right, .. } => {
                search.push_expr(context, left);
                search.push_expr(context, right);
            }
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                search.push_expr(context, condition);
                search.push_expr(context, then_expr);
                search.push_expr(context, else_expr);
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                search.push_expr(context, condition);
                search.push_statements(context, then_branch);
                search.push_statements(context, else_branch);
            }
            ExprKind::For {
                from,
                to,
                step,
                body,
                ..
            } => {
                search.push_expr(context, from);
                search.push_expr(context, to);
                if let Some(step) = step.as_deref() {
                    search.push_expr(context, step);
                }
                search.push_statements(context, body);
            }
            ExprKind::ForIn { iterable, body, .. } => {
                search.push_expr(context, iterable);
                search.push_statements(context, body);
            }
            ExprKind::While { condition, body } => {
                search.push_expr(context, condition);
                search.push_statements(context, body);
            }
            ExprKind::Switch { selector, arms } => {
                if let Some(selector) = selector.as_deref() {
                    search.push_expr(context, selector);
                }
                for arm in arms {
                    if let Some(condition) = &arm.condition {
                        search.push_expr(context, condition);
                    }
                    match &arm.result {
                        SwitchArmResult::Expr(result) => search.push_expr(context, result),
                        SwitchArmResult::Block(statements) => {
                            search.push_statements(context, statements);
                        }
                    }
                }
            }
            ExprKind::Tuple(items) => {
                for item in items {
                    search.push_expr(context, item);
                }
            }
            ExprKind::Call { callee, .. } => self.expand_legacy_input_call_result(callee, search),
            ExprKind::History { expr, offset } => {
                search.push_expr(context, expr);
                search.push_expr(context, offset);
            }
            ExprKind::Identifier(name) => {
                let Some(symbol) = self
                    .bound_symbol(name, expr.span)
                    .or_else(|| self.scope.resolve(name))
                else {
                    return false;
                };
                if !search.expanded_symbols.insert(symbol.id) {
                    return false;
                }
                #[cfg(test)]
                INITIALIZER_SEARCHES.with(|searches| searches.set(searches.get() + 1));
                if let Some(initializer) = self.symbol_init_exprs.get(&symbol.id) {
                    search.push_expr(initializer.source_context_id, &initializer.expr);
                }
            }
            ExprKind::Literal(_) | ExprKind::QualifiedName(_) => {}
        }
        false
    }

    fn expand_legacy_input_call_result<'a>(
        &'a self,
        callee: &Expr,
        search: &mut LegacyInputSearch<'a>,
    ) {
        let Some(name) = expr_name(callee) else {
            return;
        };

        // The focused legacy input call consumes its marker selector and
        // returns the configured value. No other call boundary is allowed to
        // erase marker provenance, including a UDF that returns one directly.
        if self.is_unshadowed_focused_legacy_input_call(&name, callee) {
            return;
        }

        let Some((name, function)) = self.functions.get_key_value(&name) else {
            return;
        };
        if !search.expanded_functions.insert(name.as_str()) {
            return;
        }
        match &function.body {
            FunctionBody::Expr(expr) => search.push_expr(function.source_context_id, expr),
            FunctionBody::Block(statements) => {
                search.push_return_statements(function.source_context_id, statements);
            }
        }
    }

    fn is_unshadowed_focused_legacy_input_call(&self, name: &str, callee: &Expr) -> bool {
        let is_focused_input = matches!(
            self.legacy.resolve_call(name),
            Some(crate::legacy::LegacyResolution::Focused(
                crate::legacy::LegacyRule {
                    kind: crate::legacy::LegacyRuleKind::FocusedInput,
                    ..
                }
            ))
        );
        if !is_focused_input || self.functions.contains_key(name) {
            return false;
        }

        let is_symbol_shadowed = match &callee.kind {
            ExprKind::Identifier(identifier) => {
                self.lexical_symbol_shadows_legacy_call(identifier, callee.span)
            }
            _ => false,
        };
        !is_symbol_shadowed
    }

    /// Queues the parts of a function body that can produce its result.
    fn expand_legacy_input_search_return<'a>(
        context: SourceContextId,
        statements: &'a [Stmt],
        search: &mut LegacyInputSearch<'a>,
    ) {
        let Some((last, prior)) = statements.split_last() else {
            return;
        };
        search.push_statements(context, prior);
        match &last.kind {
            StmtKind::Expr(expr) => search.push_expr(context, expr),
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                search.push_expr(context, condition);
                search.push_return_statements(context, then_branch);
                search.push_return_statements(context, else_branch);
            }
            StmtKind::For {
                from,
                to,
                step,
                body,
                ..
            } => {
                search.push_expr(context, from);
                search.push_expr(context, to);
                if let Some(step) = step {
                    search.push_expr(context, step);
                }
                search.push_return_statements(context, body);
            }
            StmtKind::ForIn { iterable, body, .. } => {
                search.push_expr(context, iterable);
                search.push_return_statements(context, body);
            }
            StmtKind::While { condition, body } => {
                search.push_expr(context, condition);
                search.push_return_statements(context, body);
            }
            _ => {}
        }
    }

    fn expand_legacy_input_search_statement<'a>(
        &'a self,
        context: SourceContextId,
        statement: &'a Stmt,
        search: &mut LegacyInputSearch<'a>,
    ) {
        match &statement.kind {
            StmtKind::Expr(expr)
            | StmtKind::Reassign { value: expr, .. }
            | StmtKind::FieldReassign { value: expr, .. } => search.push_expr(context, expr),
            StmtKind::Decl { value, .. } => {
                if self.legacy_input_type_marker(value).is_none() {
                    search.push_expr(context, value);
                }
            }
            StmtKind::ArrayFieldReassign {
                array,
                index,
                value,
                ..
            } => {
                search.push_expr(context, array);
                search.push_expr(context, index);
                search.push_expr(context, value);
            }
            StmtKind::TupleDecl { value, .. } => search.push_expr(context, value),
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                search.push_expr(context, condition);
                search.push_statements(context, then_branch);
                search.push_statements(context, else_branch);
            }
            StmtKind::For {
                from,
                to,
                step,
                body,
                ..
            } => {
                search.push_expr(context, from);
                search.push_expr(context, to);
                if let Some(step) = step {
                    search.push_expr(context, step);
                }
                search.push_statements(context, body);
            }
            StmtKind::ForIn { iterable, body, .. } => {
                search.push_expr(context, iterable);
                search.push_statements(context, body);
            }
            StmtKind::While { condition, body } => {
                search.push_expr(context, condition);
                search.push_statements(context, body);
            }
            StmtKind::Function { body, .. } => search.push_function_body(context, body),
            StmtKind::Export(export) => match &export.item {
                pine_syntax::ExportItem::Function { body, .. } => {
                    search.push_function_body(context, body);
                }
                pine_syntax::ExportItem::Const { value, .. } => search.push_expr(context, value),
                pine_syntax::ExportItem::UserType { .. }
                | pine_syntax::ExportItem::Unknown { .. } => {}
            },
            StmtKind::Method(method) => search.push_function_body(context, &method.body),
            StmtKind::Import(_)
            | StmtKind::Library(_)
            | StmtKind::UserType(_)
            | StmtKind::Break
            | StmtKind::Continue
            | StmtKind::Unsupported { .. } => {}
        }
    }

    /// Returns the marker that `expr` names through a chain of exact aliases.
    fn legacy_input_type_marker<'a>(&'a self, expr: &'a Expr) -> Option<&'static str> {
        if !self.legacy.has_canonical_string_values() {
            return None;
        }
        let mut context = self.current_source_context_id();
        let mut expr = expr;
        // Symbols on the alias path; rejects cyclic aliases.
        let mut aliases = HashSet::new();
        loop {
            if let Some(marker) = self.legacy.canonical_string_value(context, expr.span) {
                return Some(marker);
            }
            let ExprKind::Identifier(name) = &expr.kind else {
                return None;
            };
            let symbol = self.with_source_context_ref(context, |analyzer| {
                analyzer
                    .bound_symbol(name, expr.span)
                    .or_else(|| analyzer.scope.resolve(name))
            })?;
            let initializer = self.symbol_init_exprs.get(&symbol.id)?;
            if !aliases.insert(symbol.id) {
                return None;
            }
            context = initializer.source_context_id;
            expr = &initializer.expr;
        }
    }

    pub(super) fn analyze_focused_legacy_call(
        &mut self,
        name: &str,
        callee_span: Span,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> FocusedLegacyCallAnalysis {
        if self.legacy.dialect() == crate::PineDialect::V5
            && matches!(
                name,
                "plot" | "bgcolor" | "plotshape" | "plotchar" | "plotarrow"
            )
            && !args.iter().any(|arg| arg.name.as_deref() == Some("transp"))
        {
            return FocusedLegacyCallAnalysis::NotApplicable;
        }
        let Some(resolution) = self.legacy.resolve_call(name) else {
            return FocusedLegacyCallAnalysis::NotApplicable;
        };
        match resolution {
            crate::legacy::LegacyResolution::Focused(rule) => match rule.kind {
                crate::legacy::LegacyRuleKind::FocusedInput if name == "input" => {
                    self.analyze_focused_legacy_input(rule, callee_span, call_span, args, arg_types)
                }
                crate::legacy::LegacyRuleKind::FocusedOutput => self.analyze_focused_legacy_output(
                    rule,
                    name,
                    callee_span,
                    call_span,
                    args,
                    arg_types,
                ),
                crate::legacy::LegacyRuleKind::FocusedSecurity if name == "security" => self
                    .analyze_focused_legacy_security(rule, callee_span, call_span, args, arg_types),
                crate::legacy::LegacyRuleKind::FocusedExpression
                | crate::legacy::LegacyRuleKind::FocusedCall => self
                    .analyze_focused_legacy_expression(
                        rule,
                        name,
                        callee_span,
                        call_span,
                        args,
                        arg_types,
                    ),
                _ => FocusedLegacyCallAnalysis::NotApplicable,
            },
            crate::legacy::LegacyResolution::UnsupportedKnown(rule)
                if matches!(
                    rule.kind,
                    crate::legacy::LegacyRuleKind::FocusedInput
                        | crate::legacy::LegacyRuleKind::FocusedOutput
                        | crate::legacy::LegacyRuleKind::FocusedExpression
                        | crate::legacy::LegacyRuleKind::FocusedCall
                        | crate::legacy::LegacyRuleKind::FocusedSecurity
                ) =>
            {
                let crate::legacy::LegacyRuleSupport::UnsupportedKnown { reason } = rule.support
                else {
                    unreachable!("legacy resolver preserves rule support state")
                };
                self.unsupported(rule.source_name, reason, callee_span);
                FocusedLegacyCallAnalysis::Analyzed(None)
            }
            _ => FocusedLegacyCallAnalysis::NotApplicable,
        }
    }

    fn analyze_focused_legacy_expression(
        &mut self,
        rule: crate::legacy::LegacyRule,
        name: &str,
        callee_span: Span,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> FocusedLegacyCallAnalysis {
        let bound = match self
            .legacy
            .bind_legacy_expression(name, args, arg_types, call_span)
        {
            crate::legacy::LegacyExpressionBinding::Bound(bound) => bound,
            crate::legacy::LegacyExpressionBinding::Invalid(diagnostics) => {
                self.diagnostics.extend(diagnostics);
                return FocusedLegacyCallAnalysis::Analyzed(None);
            }
        };
        let pine_type = match bound.kind {
            crate::legacy::LegacyExpressionKind::Iff => self.analyze_legacy_iff(&bound, call_span),
            crate::legacy::LegacyExpressionKind::Offset => self.analyze_legacy_offset(&bound),
            crate::legacy::LegacyExpressionKind::RsiLength => {
                let mut canonical_args = bound.ordered_args.clone();
                canonical_args[0].name = Some("source".to_owned());
                canonical_args[1].name = Some("length".to_owned());
                let signature = pine_builtins::get_phase_1_builtin("ta.rsi")
                    .expect("canonical RSI signature is registered");
                self.analyze_registered_builtin(
                    "ta.rsi",
                    signature,
                    callee_span,
                    call_span,
                    &canonical_args,
                    &bound
                        .ordered_arg_types
                        .iter()
                        .copied()
                        .map(Some)
                        .collect::<Vec<_>>(),
                )
            }
            crate::legacy::LegacyExpressionKind::RsiSeries => {
                self.analyze_legacy_rsi_series(&bound, call_span)
            }
            crate::legacy::LegacyExpressionKind::Tostring => {
                let mut canonical_args = bound.ordered_args.clone();
                for (arg, name) in canonical_args.iter_mut().zip(["value", "format"]) {
                    arg.name = Some(name.to_owned());
                }
                let signature = pine_builtins::get_phase_1_builtin("str.tostring")
                    .expect("canonical str.tostring signature is registered");
                self.analyze_registered_builtin(
                    "str.tostring",
                    signature,
                    callee_span,
                    call_span,
                    &canonical_args,
                    &bound
                        .ordered_arg_types
                        .iter()
                        .copied()
                        .map(Some)
                        .collect::<Vec<_>>(),
                )
            }
            crate::legacy::LegacyExpressionKind::Vwap => {
                let mut canonical_args = bound.ordered_args.clone();
                canonical_args[0].name = Some("source".to_owned());
                let signature = pine_builtins::get_phase_1_builtin("ta.vwap")
                    .expect("canonical ta.vwap signature is registered");
                self.analyze_registered_builtin(
                    "ta.vwap",
                    signature,
                    callee_span,
                    call_span,
                    &canonical_args,
                    &bound
                        .ordered_arg_types
                        .iter()
                        .copied()
                        .map(Some)
                        .collect::<Vec<_>>(),
                )
            }
        };
        let Some(pine_type) = pine_type else {
            return FocusedLegacyCallAnalysis::Analyzed(None);
        };
        let source_context_id = self.current_source_context_id();
        self.legacy.record_expression_translation(
            &mut self.compatibility,
            source_context_id,
            callee_span,
            rule,
            &bound,
        );
        FocusedLegacyCallAnalysis::Analyzed(Some(pine_type))
    }

    fn analyze_focused_legacy_security(
        &mut self,
        rule: crate::legacy::LegacyRule,
        callee_span: Span,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> FocusedLegacyCallAnalysis {
        let const_strings = args
            .iter()
            .map(|arg| self.known_const_string_value(&arg.value))
            .collect::<Vec<_>>();
        let const_bools = args
            .iter()
            .map(|arg| self.known_const_bool_value(&arg.value))
            .collect::<Vec<_>>();
        let bound = match self.legacy.bind_legacy_security_args(
            args,
            arg_types,
            &const_strings,
            &const_bools,
            call_span,
        ) {
            crate::legacy::LegacySecurityBinding::Bound(bound) => bound,
            crate::legacy::LegacySecurityBinding::Invalid(diagnostics) => {
                self.diagnostics.extend(diagnostics);
                return FocusedLegacyCallAnalysis::Analyzed(None);
            }
        };
        let unsupported_before = self.compatibility.unsupported.len();
        let pine_type = self.analyze_bound_legacy_security(callee_span, &bound);
        if self.compatibility.unsupported.len() != unsupported_before || pine_type.is_none() {
            return FocusedLegacyCallAnalysis::Analyzed(pine_type);
        }
        let source_context_id = self.current_source_context_id();
        self.legacy.record_security_translation(
            &mut self.compatibility,
            source_context_id,
            callee_span,
            call_span,
            rule,
            &bound,
        );
        FocusedLegacyCallAnalysis::Analyzed(pine_type)
    }

    fn analyze_legacy_iff(
        &mut self,
        bound: &crate::legacy::BoundLegacyExpression,
        call_span: Span,
    ) -> Option<PineType> {
        let [condition_type, then_type, else_type] = bound.ordered_arg_types.as_slice() else {
            unreachable!("validated iff arity")
        };
        self.expect_bool(*condition_type, bound.ordered_args[0].span);
        if !matches!(
            then_type.kind,
            ValueKind::Int
                | ValueKind::Float
                | ValueKind::Bool
                | ValueKind::String
                | ValueKind::Color
                | ValueKind::Na
        ) || !matches!(
            else_type.kind,
            ValueKind::Int
                | ValueKind::Float
                | ValueKind::Bool
                | ValueKind::String
                | ValueKind::Color
                | ValueKind::Na
        ) {
            self.unsupported(
                "iff.result",
                "the current legacy iff slice supports scalar numeric, bool, string, color, and na results",
                call_span,
            );
            return None;
        }
        self.merge_branch_types(
            *condition_type,
            *then_type,
            *else_type,
            self.known_const_bool_value(&bound.ordered_args[0].value),
            call_span,
        )
    }

    fn analyze_legacy_offset(
        &mut self,
        bound: &crate::legacy::BoundLegacyExpression,
    ) -> Option<PineType> {
        let source_type = bound.ordered_arg_types[0];
        if matches!(source_type.kind, ValueKind::Tuple | ValueKind::Void) {
            self.unsupported(
                "offset.source",
                "legacy offset does not support tuple or void source values",
                bound.ordered_args[0].span,
            );
            return None;
        }
        self.validate_history_offset(
            &bound.ordered_args[1].value,
            Some(bound.ordered_arg_types[1]),
        );
        Some(PineType::new(Qualifier::Series, source_type.kind))
    }

    fn analyze_legacy_rsi_series(
        &mut self,
        bound: &crate::legacy::BoundLegacyExpression,
        call_span: Span,
    ) -> Option<PineType> {
        let first = bound.ordered_arg_types[0];
        let second = bound.ordered_arg_types[1];
        if first.qualifier != Qualifier::Series
            || !matches!(first.kind, ValueKind::Int | ValueKind::Float)
            || !matches!(second.kind, ValueKind::Int | ValueKind::Float)
        {
            self.diagnostics.push(Diagnostic::error(
                "E_LEGACY_RSI_OVERLOAD",
                "Pine v4 two-series `rsi(x, y)` requires a series numeric first argument and a numeric second argument",
                call_span,
            ));
            return None;
        }
        Some(PineType::new(Qualifier::Series, ValueKind::Float))
    }

    fn analyze_focused_legacy_input(
        &mut self,
        rule: crate::legacy::LegacyRule,
        callee_span: Span,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> FocusedLegacyCallAnalysis {
        let explicit_type_marker = self
            .legacy
            .explicit_legacy_input_type_expr(args)
            .and_then(|expr| self.legacy_input_type_marker(expr).map(str::to_owned));
        let bound = match self.legacy.bind_legacy_input_args(
            args,
            arg_types,
            explicit_type_marker.as_deref(),
        ) {
            crate::legacy::LegacyInputBinding::Bound(bound) => bound,
            crate::legacy::LegacyInputBinding::Invalid(diagnostics) => {
                self.diagnostics.extend(diagnostics);
                return FocusedLegacyCallAnalysis::Analyzed(None);
            }
        };
        let signature = pine_builtins::get_phase_1_builtin(bound.canonical_name)
            .expect("validated focused input target is registered");
        let source_context_id = self.current_source_context_id();
        self.legacy.record_input_translation(
            &mut self.compatibility,
            source_context_id,
            callee_span,
            rule,
            bound.canonical_name,
            bound.arg_rewrites,
        );
        FocusedLegacyCallAnalysis::Analyzed(self.analyze_registered_builtin(
            bound.canonical_name,
            signature,
            callee_span,
            call_span,
            &bound.canonical_args,
            &bound.canonical_arg_types,
        ))
    }

    fn analyze_focused_legacy_output(
        &mut self,
        rule: crate::legacy::LegacyRule,
        name: &str,
        callee_span: Span,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> FocusedLegacyCallAnalysis {
        let const_strings = args
            .iter()
            .map(|arg| self.known_const_string_value(&arg.value))
            .collect::<Vec<_>>();
        let const_ints = args
            .iter()
            .map(|arg| self.known_const_int_value(&arg.value))
            .collect::<Vec<_>>();
        let string_domains = args
            .iter()
            .map(|arg| self.known_string_value_domain(&arg.value))
            .collect::<Vec<_>>();
        let bound = match self.legacy.bind_legacy_output_args(
            name,
            args,
            arg_types,
            &const_strings,
            &const_ints,
            &string_domains,
        ) {
            crate::legacy::LegacyOutputBinding::Bound(bound) => bound,
            crate::legacy::LegacyOutputBinding::Invalid(diagnostics) => {
                self.diagnostics.extend(diagnostics);
                return FocusedLegacyCallAnalysis::Analyzed(None);
            }
        };
        let signature = pine_builtins::get_phase_1_builtin(bound.canonical_name)
            .expect("validated focused output target is registered");
        let source_context_id = self.current_source_context_id();
        self.legacy.record_output_translation(
            &mut self.compatibility,
            source_context_id,
            callee_span,
            rule,
            crate::legacy::LegacyOutputTranslationPlan {
                canonical_name: bound.canonical_name,
                arg_rewrites: bound.arg_rewrites,
                style_value_rewrites: bound.style_value_rewrites,
                requires_adaptation: bound.requires_adaptation,
                emulates_transparency: bound.emulates_transparency,
                emulates_numeric_style: bound.emulates_numeric_style,
            },
        );
        FocusedLegacyCallAnalysis::Analyzed(self.analyze_registered_builtin(
            bound.canonical_name,
            signature,
            callee_span,
            call_span,
            &bound.canonical_args,
            &bound.canonical_arg_types,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::INITIALIZER_SEARCHES;
    use pine_syntax::SourceFile;

    /// `x{i} = x{i-1} + x{i-1}` doubles the paths to `x0` at every level.
    fn shared_alias_graph(header: &str, root: &str, depth: usize, output: &str) -> String {
        let mut source = format!("{header}\nx0 = {root}\n");
        for level in 1..=depth {
            let previous = level - 1;
            source.push_str(&format!("x{level} = x{previous} + x{previous}\n"));
        }
        source.push_str(&output.replace("{last}", &format!("x{depth}")));
        source
    }

    /// `x{i} = x{i-1}{link}` makes each declaration depend on every earlier one.
    fn straight_line_chain(
        header: &str,
        root: &str,
        link: &str,
        depth: usize,
        output: &str,
    ) -> String {
        let mut source = format!("{header}\nx0 = {root}\n");
        for level in 1..=depth {
            let previous = level - 1;
            source.push_str(&format!("x{level} = x{previous}{link}\n"));
        }
        source.push_str(&output.replace("{last}", &format!("x{depth}")));
        source
    }

    /// The legacy header records a marker, so every query runs the search.
    const LEGACY_MARKER_HEADER: &str =
        "//@version=4\nstudy(\"shared aliases\")\nkind = input.integer";

    fn analyze_counting_searches(source: &str) -> (crate::Analysis, usize) {
        INITIALIZER_SEARCHES.with(|searches| searches.set(0));
        let analysis = crate::analyze_source(&SourceFile::new("shared-aliases.pine", source));
        (analysis, INITIALIZER_SEARCHES.with(std::cell::Cell::get))
    }

    /// Analyzes on a thread with the 1 MiB stack of a Windows main thread, so
    /// a search that recursed once per chain link would overflow it.
    fn analyze_counting_searches_on_main_thread_stack(source: String) -> (crate::Analysis, usize) {
        std::thread::Builder::new()
            .stack_size(1 << 20)
            .spawn(move || analyze_counting_searches(&source))
            .expect("analysis thread starts")
            .join()
            .expect("analysis thread completes")
    }

    fn diagnostic_codes(analysis: &crate::Analysis) -> Vec<&str> {
        analysis
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect()
    }

    #[test]
    fn marker_search_expands_each_shared_initializer_once_per_query() {
        const DEPTH: usize = 16;
        let source = shared_alias_graph(
            LEGACY_MARKER_HEADER,
            "tostring(close)",
            DEPTH,
            "alert({last})\n",
        );
        let (analysis, searches) = analyze_counting_searches(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
        // Each declaration searches its ancestors once, so the total is
        // quadratic in depth. The per-path search made 589,786 at this depth.
        assert!(
            searches <= DEPTH * DEPTH,
            "{searches} initializer searches for depth {DEPTH}"
        );
    }

    #[test]
    fn scripts_without_recorded_markers_skip_the_search() {
        let source = shared_alias_graph(
            "//@version=6\nindicator(\"shared aliases\")",
            "close",
            16,
            "plot({last})\n",
        );
        let (analysis, searches) = analyze_counting_searches(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
        assert_eq!(searches, 0);
    }

    #[test]
    fn long_modern_chain_analyzes_on_a_main_thread_stack() {
        // Before the search left the thread stack, about 700 links overflowed
        // a 1 MiB release-build stack and aborted the host process.
        const DEPTH: usize = 3_000;
        let source = straight_line_chain(
            "//@version=6\nindicator(\"chain\")",
            "close",
            " + 1",
            DEPTH,
            "plot({last})\n",
        );
        let (analysis, searches) = analyze_counting_searches_on_main_thread_stack(source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
        assert_eq!(searches, 0);
    }

    #[test]
    fn long_legacy_chain_search_stays_off_the_thread_stack() {
        const DEPTH: usize = 1_000;
        let source = straight_line_chain(
            LEGACY_MARKER_HEADER,
            "tostring(close)",
            " + \"a\"",
            DEPTH,
            "alert({last})\n",
        );
        let (analysis, searches) = analyze_counting_searches_on_main_thread_stack(source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_some());
        // Every declaration searched the whole chain above it.
        assert!(
            searches >= DEPTH * (DEPTH - 1) / 2,
            "{searches} initializer searches for depth {DEPTH}"
        );
    }

    #[test]
    fn marker_behind_a_long_alias_chain_is_still_rejected() {
        let source =
            straight_line_chain(LEGACY_MARKER_HEADER, "kind", "", 1_000, "alert({last})\n");
        let (analysis, _) = analyze_counting_searches_on_main_thread_stack(source);
        assert_eq!(
            diagnostic_codes(&analysis),
            vec!["E_LEGACY_INPUT_CONSTANT_CONTEXT"],
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    #[test]
    fn marker_after_a_searched_shared_graph_is_still_rejected() {
        let header = LEGACY_MARKER_HEADER;
        let clean = shared_alias_graph(header, "tostring(close)", 12, "alert({last})\n");
        let (analysis, _) = analyze_counting_searches(&clean);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );

        // The left operand searches the whole shared graph first; the marker
        // alias on the right must still be found by the same query.
        let leak = shared_alias_graph(header, "tostring(close)", 12, "alert({last} + kind)\n");
        let (analysis, _) = analyze_counting_searches(&leak);
        assert_eq!(
            diagnostic_codes(&analysis),
            vec!["E_LEGACY_INPUT_CONSTANT_CONTEXT"],
            "{:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());

        // A marker reached only through the shared graph is rejected as well.
        let shared_leak = shared_alias_graph(header, "kind", 12, "alert({last})\n");
        let (analysis, _) = analyze_counting_searches(&shared_leak);
        assert_eq!(
            diagnostic_codes(&analysis),
            vec!["E_LEGACY_INPUT_CONSTANT_CONTEXT"],
            "{:?}",
            analysis.diagnostics
        );
    }
}
