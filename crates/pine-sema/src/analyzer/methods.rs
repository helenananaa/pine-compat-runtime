mod bindings;

use std::collections::HashSet;

use crate::analyzer::functions::contains_output_or_declaration_call;
use crate::prelude::*;
use crate::source_graph::{SourceContextId, SourceId};

struct MethodCallReceiver {
    type_name: String,
    pine_type: PineType,
    span: Span,
}

impl Analyzer {
    pub(crate) fn analyze_postfix_user_type_call_result_method(
        &mut self,
        callee: &Expr,
        args: &[CallArg],
        call_span: Span,
        arg_types: &[Option<PineType>],
    ) -> Option<Option<PineType>> {
        let (_, method_name) = postfix_call_result_method_parts(callee, args)?;
        let receiver = args.first()?;
        let receiver_type = arg_types.first().copied().flatten()?;
        let receiver_type_name = self.method_receiver_key(&receiver.value, receiver_type)?;
        if receiver_type.kind != ValueKind::UserType
            && !self
                .methods
                .contains_key(&(receiver_type_name.clone(), method_name.to_owned()))
        {
            return None;
        }
        self.analyze_user_method_call_with_receiver(
            MethodCallReceiver {
                type_name: receiver_type_name,
                pine_type: receiver_type,
                span: callee.span,
            },
            method_name,
            call_span,
            &args[1..],
            &arg_types[1..],
        )
    }

    pub(crate) fn register_methods(&mut self, program: &Program) {
        for statement in &program.statements {
            let StmtKind::Method(method) = &statement.kind else {
                continue;
            };
            if self.legacy.dialect() < crate::PineDialect::V5 {
                self.diagnostics.push(Diagnostic::error(
                    "E_METHOD_VERSION",
                    "user-defined methods require Pine v5 or later",
                    statement.span,
                ));
                continue;
            }
            let Some(receiver) = method.params.first() else {
                self.diagnostics.push(Diagnostic::error(
                    "E_METHOD_PARAM",
                    format!("method `{}` must declare a receiver parameter", method.name),
                    statement.span,
                ));
                continue;
            };
            let receiver_name = Self::method_type_key(&receiver.type_name);
            if !self.user_types.contains_key(receiver_name)
                && !Self::builtin_method_receiver_declaration(receiver_name)
            {
                self.diagnostics.push(Diagnostic::error(
                    "E_METHOD_RECEIVER_TYPE",
                    format!(
                        "method `{}` receiver type `{}` is not a known user-defined type",
                        method.name, receiver.type_name
                    ),
                    receiver.span,
                ));
                continue;
            }
            let Some(receiver_binding) =
                self.function_param_type(&receiver.type_name, receiver.span)
            else {
                continue;
            };
            let key = (receiver_name.to_owned(), method.name.clone());
            if self.methods.contains_key(&key) {
                self.diagnostics.push(Diagnostic::error(
                    "E_METHOD_DUPLICATE",
                    format!(
                        "method `{}` is already defined for `{}`",
                        method.name, receiver.type_name
                    ),
                    method.name_span,
                ));
                continue;
            }
            let mut params = Vec::new();
            let mut names = vec![receiver.name.clone()];
            let mut valid = true;
            for param in method.params.iter().skip(1) {
                if names.iter().any(|name| name == &param.name) {
                    self.diagnostics.push(Diagnostic::error(
                        "E_METHOD_PARAM",
                        format!(
                            "method `{}` has duplicate parameter `{}`",
                            method.name, param.name
                        ),
                        param.span,
                    ));
                    valid = false;
                    continue;
                }
                names.push(param.name.clone());
                let binding = if param.type_name.is_empty() {
                    None
                } else {
                    let Some(resolved) =
                        self.method_parameter_binding(&param.type_name, param.span)
                    else {
                        valid = false;
                        continue;
                    };
                    Some(resolved)
                };
                params.push(MethodParamInfo {
                    name: param.name.clone(),
                    pine_type: binding.as_ref().map_or(UNKNOWN, |b| b.pine_type),
                    user_type_name: binding.as_ref().and_then(|b| b.user_type_name.clone()),
                    binding,
                });
            }
            if !valid {
                continue;
            }
            self.methods.insert(
                key,
                MethodInfo {
                    source_id: SourceId::root(),
                    source_context_id: SourceContextId::root(),
                    receiver_type: receiver_name.to_owned(),
                    receiver_name: receiver.name.clone(),
                    receiver_binding: Some(receiver_binding),
                    params,
                    body: method.body.clone(),
                    span: statement.span,
                },
            );
        }
    }

    pub(crate) fn analyze_user_method_call(
        &mut self,
        receiver_name: &str,
        method_name: &str,
        span: Span,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> Option<Option<PineType>> {
        let receiver_symbol = self.scope.resolve(receiver_name)?;
        let receiver = MethodCallReceiver {
            type_name: self.named_method_receiver_key(
                receiver_name,
                span,
                receiver_symbol.pine_type,
            )?,
            pine_type: receiver_symbol.pine_type,
            span,
        };
        self.analyze_user_method_call_with_receiver(
            receiver,
            method_name,
            call_span,
            args,
            arg_types,
        )
    }

    fn analyze_user_method_call_with_receiver(
        &mut self,
        receiver: MethodCallReceiver,
        method_name: &str,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> Option<Option<PineType>> {
        let Some(method) = self
            .methods
            .get(&(receiver.type_name.clone(), method_name.to_owned()))
            .cloned()
        else {
            if self.imported_user_types.contains_key(&receiver.type_name) {
                self.diagnostics.push(Diagnostic::error(
                    "E_IMPORT_UNSUPPORTED_METHOD",
                    format!(
                        "imported method `{method_name}` for receiver `{}` is not supported; imported method dispatch requires imported UDT identity",
                        receiver.type_name
                    ),
                    receiver.span,
                ));
                return Some(None);
            }
            self.diagnostics.push(Diagnostic::error(
                "E_UNKNOWN_METHOD",
                format!(
                    "unknown method `{method_name}` for `{}`",
                    receiver.type_name
                ),
                receiver.span,
            ));
            return Some(None);
        };
        let stack_name = format!("method:{}.{}", method.receiver_type, method_name);
        if self
            .function_stack
            .iter()
            .any(|active| active == &stack_name)
        {
            self.diagnostics.push(Diagnostic::error(
                "E_RECURSIVE_METHOD",
                format!("recursive method `{method_name}` is not supported"),
                receiver.span,
            ));
            return Some(None);
        }
        if self.function_stack.len() >= MAX_FUNCTION_CALL_DEPTH {
            self.diagnostics.push(Diagnostic::error(
                "E_FUNCTION_CALL_DEPTH",
                "user-defined method call chain is too deep",
                receiver.span,
            ));
            return Some(None);
        }
        for arg in args {
            if contains_output_or_declaration_call(&arg.value) {
                self.unsupported(
                    "function_side_effect",
                    "side-effecting calls cannot be passed as user-defined method arguments",
                    arg.span,
                );
            }
        }
        let params: Vec<_> = method
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let arg_indices = match resolve_udf_arg_indices(&params, args) {
            Ok(arg_indices) => arg_indices,
            Err(error) => {
                self.report_udf_arg_error(
                    method_name,
                    receiver.span,
                    params.len(),
                    args.len(),
                    error,
                );
                return Some(None);
            }
        };

        self.compatibility.supported.push(FeatureUse {
            feature: "user-defined methods".to_owned(),
            span: method.span,
        });
        self.scope.push_scope();
        let receiver_type = method
            .receiver_binding
            .as_ref()
            .map_or(receiver.pine_type, |binding| {
                binding.bound_type(receiver.pine_type)
            });
        if let Some(binding) = &method.receiver_binding
            && !can_assign(binding.pine_type, receiver.pine_type)
        {
            self.diagnostics.push(Diagnostic::error(
                "E_METHOD_ARG_TYPE",
                format!(
                    "cannot pass {} to method receiver `{}` of type {}",
                    pine_type_name(receiver.pine_type),
                    method.receiver_name,
                    pine_type_name(binding.pine_type)
                ),
                receiver.span,
            ));
        }
        let receiver_symbol =
            self.define_local_symbol(&method.receiver_name, receiver_type, None, false);
        if receiver.pine_type.kind == ValueKind::UserType {
            self.mark_symbol_user_type(receiver_symbol, method.receiver_type.clone());
        }
        if receiver_type.kind == ValueKind::UserTypeArray
            && let Some(name) = method
                .receiver_binding
                .as_ref()
                .and_then(|b| b.user_type_name.clone())
        {
            self.mark_symbol_user_type_array(receiver_symbol, name);
        }

        if let Some(info) = method.receiver_binding.as_ref().and_then(|b| b.map_type) {
            self.mark_symbol_map(receiver_symbol, info);
        }
        let mut param_symbols = std::collections::HashSet::from([receiver_symbol.id]);
        let mut resolved_arg_types = vec![None; method.params.len()];
        let mut resolved_arg_user_types = vec![None; method.params.len()];
        let mut resolved_arg_user_type_arrays = vec![None; method.params.len()];
        let mut resolved_arg_map_infos = vec![None; method.params.len()];
        let mut resolved_arg_const_switch_keys = vec![None; method.params.len()];
        for (arg_index, param_index) in arg_indices.iter().copied().enumerate() {
            resolved_arg_types[param_index] = arg_types.get(arg_index).copied().flatten();
            resolved_arg_map_infos[param_index] = self.map_type_of_expr(&args[arg_index].value);
            resolved_arg_user_types[param_index] = args
                .get(arg_index)
                .and_then(|arg| self.user_type_name_of_expr(&arg.value));
            resolved_arg_user_type_arrays[param_index] = args
                .get(arg_index)
                .and_then(|arg| self.user_type_array_name_of_expr(&arg.value));
            resolved_arg_const_switch_keys[param_index] = args
                .get(arg_index)
                .and_then(|arg| self.known_const_switch_key(&arg.value));
        }
        let mut param_const_switch_keys = std::collections::HashMap::new();
        for (
            param,
            (
                (((arg_type, arg_user_type), arg_user_type_array), arg_const_switch_key),
                arg_map_info,
            ),
        ) in method.params.iter().zip(
            resolved_arg_types
                .into_iter()
                .zip(resolved_arg_user_types)
                .zip(resolved_arg_user_type_arrays)
                .zip(resolved_arg_const_switch_keys)
                .zip(resolved_arg_map_infos),
        ) {
            let arg_type = arg_type.unwrap_or(UNKNOWN);
            let symbol =
                self.define_local_symbol(&param.name, param.bound_type(arg_type), None, false);
            param_symbols.insert(symbol.id);
            if let Some(key) = arg_const_switch_key.as_ref() {
                self.record_symbol_const_switch_key(symbol, key);
            }
            if param.pine_type != UNKNOWN
                && !param.binding.as_ref().map_or_else(
                    || can_assign(param.pine_type, arg_type),
                    |b| b.accepts_argument(arg_type, arg_map_info),
                )
            {
                self.diagnostics.push(Diagnostic::error(
                    "E_METHOD_ARG_TYPE",
                    format!(
                        "cannot pass {} to method parameter `{}` of type {}",
                        pine_type_name(arg_type),
                        param.name,
                        if param
                            .binding
                            .as_ref()
                            .is_some_and(|binding| binding.explicit_series
                                || binding.explicit_simple
                                || (binding.enforce_qualifier
                                    && binding.pine_type.qualifier == Qualifier::Simple)
                                || binding.pine_type.qualifier == Qualifier::Const)
                        {
                            pine_type_name(param.pine_type)
                        } else {
                            value_kind_name(param.pine_type.kind).to_owned()
                        }
                    ),
                    receiver.span,
                ));
            }
            if param.pine_type.kind == ValueKind::UserTypeArray {
                if let Some(expected_type_name) = &param.user_type_name {
                    if arg_user_type_array.as_deref() == Some(expected_type_name.as_str()) {
                        self.mark_symbol_user_type_array(symbol, expected_type_name.clone());
                    } else if arg_type.kind == ValueKind::UserTypeArray {
                        self.diagnostics.push(Diagnostic::error(
                            "E_METHOD_ARG_TYPE",
                            format!(
                                "cannot pass a different user-defined type array to method parameter `{}`",
                                param.name
                            ),
                            receiver.span,
                        ));
                    }
                }
            } else if let Some(expected_type_name) = &param.user_type_name {
                if arg_user_type.as_deref() == Some(expected_type_name.as_str()) {
                    self.mark_symbol_user_type(symbol, expected_type_name.clone());
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        "E_METHOD_ARG_TYPE",
                        format!(
                            "cannot pass argument to method parameter `{}` of user-defined type `{}`",
                            param.name, expected_type_name
                        ),
                        receiver.span,
                    ));
                }
            }
            if let Some(type_name) = arg_user_type {
                self.mark_symbol_user_type(symbol, type_name);
            }
            if let Some(type_name) = arg_user_type_array {
                self.mark_symbol_user_type_array(symbol, type_name);
            }
            if let Some(info) = param
                .binding
                .as_ref()
                .and_then(|b| b.map_type)
                .or(arg_map_info)
            {
                self.mark_symbol_map(symbol, info);
            }
            if let Some(key) = arg_const_switch_key {
                param_const_switch_keys.insert(param.name.clone(), key);
            }
        }
        self.function_stack.push(stack_name);
        self.function_param_symbols.push(param_symbols);
        self.function_param_const_switch_keys
            .push(param_const_switch_keys);
        self.function_context_is_method.push(true);
        self.function_tuple_identity_slots.push(HashSet::new());
        self.function_depth += 1;
        let (return_type, body_user_type, body_user_type_array, body_map, unresolved_tuple_slots) =
            self.with_source_context(method.source_context_id, |analyzer| {
                let diagnostic_start = analyzer.diagnostics.len();
                let return_type = analyzer.analyze_function_body(&method.body, method.span);
                let body_user_type = return_type
                    .is_some_and(|pine_type| pine_type.kind == ValueKind::UserType)
                    .then(|| analyzer.user_type_name_of_function_body(&method.body))
                    .flatten();
                let body_user_type_array = return_type
                    .is_some_and(|pine_type| pine_type.kind == ValueKind::UserTypeArray)
                    .then(|| analyzer.user_type_array_name_of_function_body(&method.body))
                    .flatten();
                let body_map = return_type
                    .is_some_and(|pine_type| pine_type.kind == ValueKind::Map)
                    .then(|| analyzer.map_type_of_function_body(&method.body))
                    .flatten();
                let has_new_errors = analyzer.diagnostics[diagnostic_start..]
                    .iter()
                    .any(|diagnostic| diagnostic.severity == Severity::Error);
                let unresolved_tuple_slots = if !has_new_errors
                    && return_type.is_some_and(|pine_type| pine_type.kind == ValueKind::Tuple)
                {
                    analyzer.unresolved_function_body_user_type_array_tuple_slots(&method.body)
                } else {
                    Vec::new()
                };
                (
                    return_type,
                    body_user_type,
                    body_user_type_array,
                    body_map,
                    unresolved_tuple_slots,
                )
            });
        self.function_depth -= 1;
        self.function_context_is_method.pop();
        self.function_param_const_switch_keys.pop();
        self.function_param_symbols.pop();
        self.function_stack.pop();
        self.scope.pop_scope();

        let mut tuple_identity_slots = self
            .function_tuple_identity_slots
            .pop()
            .expect("tuple identity call scope should exist");
        tuple_identity_slots.extend(unresolved_tuple_slots);
        if let Some(parent_slots) = self.function_tuple_identity_slots.last_mut() {
            parent_slots.extend(tuple_identity_slots);
        } else {
            let mut tuple_identity_slots: Vec<_> = tuple_identity_slots.into_iter().collect();
            tuple_identity_slots.sort_unstable();
            for index in tuple_identity_slots {
                self.diagnostics.push(Diagnostic::error(
                    "E_TUPLE_UDT_ARRAY_IDENTITY",
                    format!(
                        "tuple element {} user-defined type array must resolve to one element identity",
                        index + 1
                    ),
                    call_span,
                ));
            }
        }

        if return_type.is_some_and(|pine_type| pine_type.kind == ValueKind::UserType)
            && let Some(type_name) = body_user_type
        {
            self.mark_expr_user_type(call_span, type_name.clone());
            self.mark_expr_user_type(receiver.span, type_name);
        }
        if return_type.is_some_and(|pine_type| pine_type.kind == ValueKind::UserTypeArray)
            && let Some(type_name) = body_user_type_array
        {
            self.mark_expr_user_type_array(call_span, type_name.clone());
            self.mark_expr_user_type_array(receiver.span, type_name);
        }
        if return_type.is_some_and(|pine_type| pine_type.kind == ValueKind::Map)
            && let Some(info) = body_map
        {
            self.mark_expr_map(call_span, info);
            self.mark_expr_map(receiver.span, info);
        }
        self.user_method_call_results
            .insert(self.expr_key(call_span));
        Some(return_type)
    }

    pub(crate) fn analyze_alias_qualified_user_method_call(
        &mut self,
        name: &str,
        span: Span,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> Option<Option<PineType>> {
        let (alias, method_name) = alias_qualified_method_name(name)?;
        if !self.methods.keys().any(|(receiver_type, candidate)| {
            candidate == method_name && receiver_type.starts_with(&format!("{alias}."))
        }) {
            return None;
        }
        let Some(receiver_arg) = args.first() else {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                format!("`{name}` expects a receiver argument"),
                span,
            ));
            return Some(None);
        };
        let receiver_type = arg_types.first().copied().flatten();
        let Some(receiver_user_type) = self.user_type_name_of_expr(&receiver_arg.value) else {
            if let Some(receiver_type) = receiver_type {
                self.diagnostics.push(Diagnostic::error(
                    "E_METHOD_ARG_TYPE",
                    format!(
                        "cannot pass {} as receiver to imported method `{name}`",
                        pine_type_name(receiver_type)
                    ),
                    receiver_arg.span,
                ));
            }
            return Some(None);
        };
        if !receiver_user_type.starts_with(&format!("{alias}.")) {
            self.diagnostics.push(Diagnostic::error(
                "E_METHOD_ARG_TYPE",
                format!("cannot pass receiver `{receiver_user_type}` to imported method `{name}`"),
                receiver_arg.span,
            ));
            return Some(None);
        }
        if !self
            .methods
            .contains_key(&(receiver_user_type.clone(), method_name.to_owned()))
        {
            self.diagnostics.push(Diagnostic::error(
                "E_UNKNOWN_METHOD",
                format!("unknown imported method `{name}` for receiver `{receiver_user_type}`"),
                span,
            ));
            return Some(None);
        }
        self.analyze_user_method_call_with_receiver(
            MethodCallReceiver {
                type_name: receiver_user_type,
                pine_type: receiver_type.unwrap_or(UNKNOWN),
                span,
            },
            method_name,
            call_span,
            &args[1..],
            &arg_types[1..],
        )
    }

    pub(crate) fn analyze_local_qualified_user_method_call(
        &mut self,
        name: &str,
        span: Span,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> Option<Option<PineType>> {
        let (type_name, method_name) = alias_qualified_method_name(name)?;
        if !self.user_types.contains_key(type_name) {
            return None;
        }
        let Some(receiver_arg) = args.first() else {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                format!("`{name}` expects a receiver argument"),
                span,
            ));
            return Some(None);
        };
        let receiver_type = arg_types.first().copied().flatten();
        let Some(receiver_user_type) = self.user_type_name_of_expr(&receiver_arg.value) else {
            if let Some(receiver_type) = receiver_type {
                self.diagnostics.push(Diagnostic::error(
                    "E_METHOD_ARG_TYPE",
                    format!(
                        "cannot pass {} as receiver to method `{name}`",
                        pine_type_name(receiver_type)
                    ),
                    receiver_arg.span,
                ));
            }
            return Some(None);
        };
        if receiver_user_type != type_name {
            self.diagnostics.push(Diagnostic::error(
                "E_METHOD_ARG_TYPE",
                format!("cannot pass receiver `{receiver_user_type}` to method `{name}`"),
                receiver_arg.span,
            ));
            return Some(None);
        }
        if !self
            .methods
            .contains_key(&(receiver_user_type.clone(), method_name.to_owned()))
        {
            self.diagnostics.push(Diagnostic::error(
                "E_UNKNOWN_METHOD",
                format!("unknown method `{name}` for receiver `{receiver_user_type}`"),
                span,
            ));
            return Some(None);
        }
        self.analyze_user_method_call_with_receiver(
            MethodCallReceiver {
                type_name: receiver_user_type,
                pine_type: receiver_type.unwrap_or(UNKNOWN),
                span,
            },
            method_name,
            call_span,
            &args[1..],
            &arg_types[1..],
        )
    }
}
