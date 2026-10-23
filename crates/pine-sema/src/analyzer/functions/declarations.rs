//! Declaration checks use an isolated analyzer. Call-specific metadata must never
//! escape this pass into lowering or a subsequent specialization of a UDF.
mod deferred;

use std::collections::HashSet;

use crate::prelude::*;
use crate::source_graph::{SourceContextId, SourceId};
use deferred::DeferredExpressions;

struct BodyDeclaration {
    name: String,
    namespace: Option<String>,
    names: Vec<String>,
    types: Vec<Option<FunctionParamInfo>>,
    body: FunctionBody,
    span: Span,
    context: SourceContextId,
    method: bool,
}

impl From<(String, FunctionInfo)> for BodyDeclaration {
    fn from((name, function): (String, FunctionInfo)) -> Self {
        Self {
            namespace: name
                .strip_prefix("@import:")
                .unwrap_or(&name)
                .rsplit_once('.')
                .map(|(prefix, _)| format!("{prefix}.")),
            name,
            names: function.params,
            types: function.param_types,
            body: function.body,
            span: function.span,
            context: function.source_context_id,
            method: false,
        }
    }
}

impl From<(String, MethodInfo)> for BodyDeclaration {
    fn from((method_name, method): (String, MethodInfo)) -> Self {
        let mut receiver_binding = method.receiver_binding.or_else(|| {
            Some(FunctionParamInfo {
                pine_type: PineType::new(Qualifier::Series, ValueKind::UserType),
                explicit_series: false,
                explicit_simple: false,
                enforce_qualifier: false,
                user_type_name: Some(method.receiver_type.clone()),
                map_type: None,
                span: method.span,
            })
        });
        if let Some(binding) = &mut receiver_binding
            && binding.pine_type.kind == ValueKind::UserType
        {
            // Imported methods are re-contextualized per alias. The receiver
            // identity comes from that method instance, as at ordinary calls.
            binding.user_type_name = Some(method.receiver_type.clone());
        }
        let receiver_namespace = receiver_binding
            .as_ref()
            .and_then(|binding| match binding.pine_type.kind {
                ValueKind::UserType => Some(method.receiver_type.as_str()),
                ValueKind::UserTypeArray => method
                    .receiver_type
                    .strip_prefix("array<")
                    .and_then(|n| n.strip_suffix('>')),
                _ => None,
            })
            .and_then(|name| name.rsplit_once('.'))
            .map(|(prefix, _)| format!("{prefix}."));
        Self {
            namespace: receiver_namespace.or_else(|| {
                method_name
                    .rsplit_once('.')
                    .map(|(prefix, _)| format!("{prefix}."))
            }),
            name: format!("method:{}.{}", method.receiver_type, method_name),
            names: std::iter::once(method.receiver_name)
                .chain(method.params.iter().map(|p| p.name.clone()))
                .collect(),
            types: std::iter::once(receiver_binding)
                .chain(method.params.into_iter().map(|p| {
                    p.binding.or_else(|| {
                        (p.pine_type != UNKNOWN).then_some(FunctionParamInfo {
                            pine_type: p.pine_type,
                            explicit_series: false,
                            explicit_simple: false,
                            enforce_qualifier: false,
                            user_type_name: p.user_type_name,
                            map_type: None,
                            span: method.span,
                        })
                    })
                }))
                .collect(),
            body: method.body,
            span: method.span,
            context: method.source_context_id,
            method: true,
        }
    }
}

impl Analyzer {
    pub(crate) fn validate_function_declaration(&mut self, name: &str) {
        if self.legacy.dialect().version() >= 5
            && let Some(function) = self.functions.get(name).cloned()
        {
            self.validate_body_declaration((name.to_owned(), function).into());
        }
    }

    pub(crate) fn validate_method_declaration(&mut self, span: Span) {
        if let Some(((.., name), method)) = self.methods.iter().find(|(_, m)| {
            m.source_id == SourceId::root()
                && m.source_context_id == self.current_source_context_id()
                && m.span == span
        }) {
            self.validate_body_declaration((name.clone(), method.clone()).into());
        }
    }

    pub(crate) fn validate_imported_body_declarations(&mut self) {
        let mut functions: Vec<_> = self
            .functions
            .iter()
            .filter(|(_, f)| f.source_id != SourceId::root())
            .flat_map(|(name, f)| {
                std::iter::once(f)
                    .chain(f.overloads.iter())
                    .map(|f| (name.clone(), f.clone()))
            })
            .collect();
        functions.sort_by(|(left_name, left), (right_name, right)| {
            (left.source_context_id, left.span.start, left_name).cmp(&(
                right.source_context_id,
                right.span.start,
                right_name,
            ))
        });
        for function in functions {
            self.validate_body_declaration(function.into());
        }
        let mut methods: Vec<_> = self
            .methods
            .iter()
            .filter(|(_, m)| m.source_id != SourceId::root())
            .map(|((_, name), method)| (name.clone(), method.clone()))
            .collect();
        methods.sort_by(|(left_name, left), (right_name, right)| {
            (
                left.source_context_id,
                left.span.start,
                &left.receiver_type,
                left_name,
            )
                .cmp(&(
                    right.source_context_id,
                    right.span.start,
                    &right.receiver_type,
                    right_name,
                ))
        });
        for method in methods {
            self.validate_body_declaration(method.into());
        }
    }

    fn validate_body_declaration(&mut self, declaration: BodyDeclaration) {
        let mut check = self.clone();
        check.diagnostics.clear();
        check.declaration_validation = true;
        check.prepare_private_declaration_types(
            declaration.context,
            declaration.namespace.as_deref(),
        );
        check.expr_types.clear();
        check.expr_maps.clear();
        check.expr_user_types.clear();
        check.expr_user_type_arrays.clear();
        check.expr_user_type_matrices.clear();
        check.expr_user_type_identities.clear();
        check.scope.push_scope();
        let mut symbols = HashSet::new();
        let mut unknown = HashSet::new();
        for (name, binding) in declaration.names.iter().zip(&declaration.types) {
            let ty = binding.as_ref().map_or(UNKNOWN, |p| p.pine_type);
            let symbol = check.define_local_symbol(name, ty, None, false);
            symbols.insert(symbol.id);
            if let Some(binding) = binding {
                if let Some(info) = binding.map_type {
                    check.mark_symbol_map(symbol, info);
                }
                if let Some(name) = &binding.user_type_name {
                    if ty.kind == ValueKind::UserTypeArray {
                        check.mark_symbol_user_type_array(symbol, name.clone());
                    } else {
                        check.mark_symbol_user_type(symbol, name.clone());
                    }
                }
            } else {
                unknown.insert(name.clone());
            }
        }
        let source_id = self
            .source_context_origins
            .get(&declaration.context)
            .map_or(0, |(id, _)| id.get());
        let deferred = DeferredExpressions::for_body(
            &declaration.body,
            unknown,
            &check.functions,
            &check.methods,
            source_id,
        );
        check.function_stack.push(declaration.name);
        check.function_param_symbols.push(symbols);
        check.function_param_const_switch_keys.push(HashMap::new());
        check.function_context_is_method.push(declaration.method);
        check.function_tuple_identity_slots.push(HashSet::new());
        check.function_depth += 1;
        check.loop_depth = 0;
        check.with_source_context(declaration.context, |analyzer| {
            analyzer.analyze_function_body(&declaration.body, declaration.span);
        });
        for diagnostic in check.diagnostics {
            // A placeholder is not proof of a type error. Name/arity/location
            // errors on closed expressions remain independently checkable.
            if !deferred.defers(&diagnostic) && !self.declaration_diagnostics.contains(&diagnostic)
            {
                self.declaration_diagnostics.push(diagnostic);
            }
        }
    }

    fn prepare_private_declaration_types(
        &mut self,
        context: SourceContextId,
        namespace: Option<&str>,
    ) {
        let Some((source_id, _)) = self.source_context_origins.get(&context).cloned() else {
            return;
        };
        if source_id == SourceId::root() {
            return;
        }
        let Some(source) = self.source_texts.get(&source_id) else {
            return;
        };
        let mut program = pine_syntax::parse_source(source).program;
        if let Some(dialect) = program
            .version
            .and_then(|v| crate::PineDialect::from_version(v.version))
            && dialect != self.legacy.dialect()
        {
            // The isolated pass emits no lowering state. A v5 library retains
            // its own language rules even when imported by a v6 root script.
            self.legacy = crate::legacy::LegacyFrontEnd::new(dialect);
        }
        let private_names: HashSet<_> = program
            .statements
            .iter()
            .filter_map(|s| match &s.kind {
                StmtKind::UserType(ty) => Some(ty.name.clone()),
                _ => None,
            })
            .collect();
        // Retain qualified imported definitions, but do not let root-local
        // declarations supply names missing from this library's own scope.
        self.user_types.clear();
        let prefix = namespace.map(str::to_owned).or_else(|| {
            self.functions.iter().find_map(|(name, f)| {
                (f.source_context_id == context)
                    .then(|| name.strip_suffix(&f.display_name))
                    .flatten()
                    .filter(|p| !p.is_empty())
                    .map(str::to_owned)
            })
        });
        let globals = self.scope.scopes.first().cloned().unwrap_or_default();
        self.scope = crate::resolver::ScopeResolver::new(
            crate::symbols::initial_symbols(),
            crate::symbols::initial_symbol_order(),
        );
        if let Some(prefix) = prefix.as_ref() {
            for (name, symbol) in globals {
                if name.starts_with(prefix) || name.starts_with("@source") {
                    self.scope.define_global(&name, symbol);
                }
            }
        }
        self.functions.retain(|name, f| {
            f.source_id != SourceId::root()
                && prefix.as_ref().is_none_or(|p| {
                    let name = name.strip_prefix("@import:").unwrap_or(name);
                    // Hidden dependency namespaces are generated only by
                    // the source's resolved imports and cannot be written
                    // as Pine identifiers in the caller's namespace.
                    name.starts_with(p) || name.starts_with("@source")
                })
        });
        self.methods.retain(|(_, name), m| {
            m.source_id != SourceId::root()
                && (m.source_context_id == context
                    || prefix.as_ref().is_none_or(|p| {
                        name.starts_with(p)
                            || m.receiver_type.starts_with(p)
                            || name.starts_with("@source")
                            || m.receiver_type.starts_with("@source")
                    }))
        });
        if let Some(prefix) = prefix.as_ref() {
            self.imported_user_types
                .retain(|name, _| name.starts_with(prefix) || name.starts_with("@source"));
        }
        let aliases: Vec<_> = self
            .functions
            .iter()
            .filter_map(|(name, function)| {
                let short = name
                    .strip_prefix("@import:")
                    .unwrap_or(name)
                    .strip_prefix(prefix.as_deref()?)?;
                (!short.contains('.')).then(|| (short.to_owned(), function.clone()))
            })
            .collect();
        self.functions.extend(aliases);
        if private_names.is_empty() {
            return;
        }
        for statement in &mut program.statements {
            if let StmtKind::Export(export) = &statement.kind
                && let pine_syntax::ExportItem::UserType { decl, .. } = &export.item
            {
                statement.kind = StmtKind::UserType(decl.clone());
            }
        }
        self.with_source_context(context, |check| check.register_user_types(&program));
        for ty in self.user_types.values_mut() {
            ty.identity.source_id = source_id;
        }
        let mut methods = program.clone();
        methods.statements.retain(|s| match &s.kind {
            StmtKind::Method(m) => m.params.first().is_some_and(|p| {
                private_names.iter().any(|name| {
                    p.type_name == *name
                        || p.type_name == format!("array<{name}>")
                        || p.type_name == format!("{name}[]")
                })
            }),
            _ => false,
        });
        self.with_source_context(context, |check| check.register_methods(&methods));
        for method in self
            .methods
            .values_mut()
            .filter(|m| m.source_id == SourceId::root())
        {
            method.source_id = source_id;
            method.source_context_id = context;
        }
    }
}
