use super::*;

impl Analyzer {
    pub(super) fn method_parameter_binding(
        &mut self,
        name: &str,
        span: Span,
    ) -> Option<FunctionParamInfo> {
        let start = self.diagnostics.len();
        let result = self.function_param_type(name, span);
        for diagnostic in &mut self.diagnostics[start..] {
            if diagnostic.code == "E_FUNCTION_PARAM_TYPE" {
                diagnostic.code = "E_METHOD_PARAM".to_owned();
            }
        }
        result
    }

    pub(crate) fn method_type_key(name: &str) -> &str {
        name.strip_prefix("const ")
            .or_else(|| name.strip_prefix("simple "))
            .or_else(|| name.strip_prefix("series "))
            .unwrap_or(name)
    }

    pub(crate) fn builtin_method_receiver_declaration(name: &str) -> bool {
        matches!(
            name,
            "int"
                | "float"
                | "bool"
                | "string"
                | "color"
                | "label"
                | "line"
                | "linefill"
                | "polyline"
                | "box"
                | "table"
                | "chart.point"
        ) || name.starts_with("array<")
            || super::super::functions::collections::collection_parameter_info(name).is_some()
    }

    pub(crate) fn method_receiver_key(&self, receiver: &Expr, ty: PineType) -> Option<String> {
        match ty.kind {
            ValueKind::UserType => self.user_type_name_of_expr(receiver),
            ValueKind::Map => self.map_type_of_expr(receiver).map(|m| {
                format!(
                    "map<{},{}>",
                    value_kind_name(m.key_kind),
                    value_kind_name(m.value_kind)
                )
            }),
            ValueKind::UserTypeArray => self
                .user_type_array_name_of_expr(receiver)
                .map(|name| format!("array<{name}>")),
            _ => Some(value_kind_name(ty.kind).to_owned()),
        }
    }

    pub(crate) fn named_method_receiver_key(
        &self,
        name: &str,
        span: Span,
        ty: PineType,
    ) -> Option<String> {
        self.method_receiver_key(
            &Expr {
                kind: ExprKind::Identifier(name.to_owned()),
                span,
            },
            ty,
        )
    }

    pub(crate) fn has_named_user_method(
        &self,
        name: &str,
        span: Span,
        ty: PineType,
        method: &str,
    ) -> bool {
        ty.kind == ValueKind::UserType
            || self
                .named_method_receiver_key(name, span, ty)
                .is_some_and(|key| self.methods.contains_key(&(key, method.to_owned())))
    }

    pub(crate) fn analyze_expression_user_method(
        &mut self,
        receiver: &Expr,
        ty: PineType,
        name: &str,
        span: Span,
        args: &[CallArg],
        types: &[Option<PineType>],
    ) -> Option<Option<PineType>> {
        let key = self.method_receiver_key(receiver, ty)?;
        if !self.methods.contains_key(&(key.clone(), name.to_owned())) {
            return None;
        }
        self.analyze_user_method_call_with_receiver(
            MethodCallReceiver {
                type_name: key,
                pine_type: ty,
                span: receiver.span,
            },
            name,
            span,
            args,
            types,
        )
    }

    pub(crate) fn analyze_plain_user_method(
        &mut self,
        name: &str,
        span: Span,
        args: &[CallArg],
        types: &[Option<PineType>],
    ) -> Option<Option<PineType>> {
        if name.contains('.') {
            return None;
        }
        let receiver = args.first()?;
        self.analyze_expression_user_method(
            &receiver.value,
            types.first().copied().flatten()?,
            name,
            span,
            &args[1..],
            &types[1..],
        )
    }
}
