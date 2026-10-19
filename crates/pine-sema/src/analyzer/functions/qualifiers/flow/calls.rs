use super::*;

impl Walker<'_> {
    pub(super) fn call(&mut self, callee: &Expr, args: &[CallArg]) -> Flow {
        let flows = args.iter().map(|a| self.expr(&a.value)).collect::<Vec<_>>();
        let name = expr_name(callee);
        if let Some(keys) = name.as_ref().and_then(|n| self.functions.get(n))
            && let Some(key) = keys
                .iter()
                .find(|k| self.fits(&self.units[*k], args, &flows))
        {
            return self.user_call(key, args, &flows);
        }
        if let Some(name) = &name
            && let Some(signature) = pine_builtins::get_phase_1_builtin(name)
        {
            return self.builtin(signature, args, &flows);
        }
        let receiver_method = match &callee.kind {
            ExprKind::Member { receiver, name } => Some((self.expr(receiver), name.clone(), false)),
            ExprKind::QualifiedName(parts) if parts.len() > 1 => {
                let prefix = parts[..parts.len() - 1].join(".");
                if let Some(receiver) = self.env.get(&prefix) {
                    Some((receiver.clone(), parts.last().unwrap().clone(), false))
                } else if postfix_call_result_method_parts(callee, args).is_some() {
                    flows
                        .first()
                        .cloned()
                        .map(|flow| (flow, parts.last().unwrap().clone(), true))
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some((receiver, method, already_first)) = receiver_method {
            let mut method_flows = flows.clone();
            let mut method_args = args.to_vec();
            if !already_first {
                method_flows.insert(0, receiver.clone());
                method_args.insert(
                    0,
                    CallArg {
                        name: None,
                        value: callee.clone(),
                        span: callee.span,
                    },
                );
            }
            let family = if is_array_kind(receiver.kind) {
                Some("array")
            } else if is_matrix_kind(receiver.kind) {
                Some("matrix")
            } else {
                receiver.hint.as_deref()
            };
            if is_array_kind(receiver.kind)
                && let Some(signature) =
                    pine_builtins::get_phase_1_builtin(&format!("array.{method}"))
            {
                return self.builtin(signature, &method_args, &method_flows);
            }
            if let Some(key) = receiver
                .hint
                .as_ref()
                .and_then(|hint| self.methods.get(&(hint.clone(), method.clone())))
            {
                return self.user_call(key, &method_args, &method_flows);
            }
            if let Some(signature) = family.and_then(|prefix| {
                pine_builtins::get_phase_1_builtin(&format!("{prefix}.{method}"))
            }) {
                return self.builtin(signature, &method_args, &method_flows);
            }
        }
        // Methods can also be invoked as ordinary functions with the receiver
        // supplied in the first argument slot.
        if let (Some(name), Some(receiver)) = (&name, flows.first())
            && let Some(key) = receiver
                .hint
                .as_ref()
                .and_then(|hint| self.methods.get(&(hint.clone(), name.clone())))
        {
            return self.user_call(key, args, &flows);
        }
        if let Some(name) = name {
            if let Some(ty) = name.strip_suffix(".new") {
                let mut result = Flow::typed(PineType::new(Qualifier::Series, ValueKind::UserType));
                result.hint = Some(ty.to_owned());
                return result;
            }
            if let Some(element) = name
                .strip_prefix("array.new<")
                .and_then(|s| s.strip_suffix('>'))
            {
                let mut result = Flow::typed(PineType::new(
                    Qualifier::Series,
                    array_kind_from_element_type_name(element).unwrap_or(ValueKind::UserTypeArray),
                ));
                result.hint = Some(format!("array<{element}>"));
                return result;
            }
        }
        Flow::unknown()
    }
    fn argument_slots(
        &self,
        names: &[String],
        args: &[CallArg],
        flows: &[Flow],
    ) -> Vec<Option<Flow>> {
        let mut slots = vec![None; names.len()];
        let mut positional = 0;
        for (arg, flow) in args.iter().zip(flows) {
            let index = if let Some(name) = &arg.name {
                names.iter().position(|n| n == name)
            } else {
                let index = positional;
                positional += 1;
                Some(index)
            };
            if let Some(slot) = index.and_then(|i| slots.get_mut(i)) {
                *slot = Some(flow.clone());
            }
        }
        slots
    }
    fn fits(&self, unit: &Unit, args: &[CallArg], flows: &[Flow]) -> bool {
        if args.len() > unit.names.len() {
            return false;
        }
        let slots = self.argument_slots(&unit.names, args, flows);
        slots.iter().enumerate().all(|(i, slot)| match slot {
            None => unit.defaults.get(i).is_some_and(Option::is_some),
            Some(flow) => unit.params[i].as_ref().is_none_or(|param| {
                flow.kind == ValueKind::Na
                    || param.pine_type.kind == flow.kind
                    || (param.pine_type.kind == ValueKind::Float && flow.kind == ValueKind::Int)
            }),
        })
    }
    fn user_call(&mut self, key: &str, args: &[CallArg], flows: &[Flow]) -> Flow {
        let unit = &self.units[key];
        let summary = &self.summaries[key];
        let mut slots = self.argument_slots(&unit.names, args, flows);
        for (i, slot) in slots.iter_mut().enumerate() {
            if slot.is_none()
                && let Some(Some(default)) = unit.defaults.get(i)
            {
                *slot = Some(self.expr(default));
            }
        }
        for (i, param) in unit.params.iter().enumerate() {
            if (summary.simple.contains(&i) || param.as_ref().is_some_and(|p| p.explicit_simple))
                && let Some(Some(flow)) = slots.get(i)
            {
                self.require_simple(flow);
            }
        }
        let mut returned = summary.returned.clone();
        returned.deps.clear();
        for index in &summary.returned.deps {
            if let Some(Some(param)) = unit.params.get(*index) {
                let param = super::super::resolved(param, summary.simple.contains(index));
                returned.fixed = strongest_qualifier(returned.fixed, param.pine_type.qualifier);
            } else if let Some(Some(flow)) = slots.get(*index) {
                returned.fixed = strongest_qualifier(returned.fixed, flow.fixed);
                returned.deps.extend(&flow.deps);
            }
        }
        if let Some(index) = returned
            .hint
            .as_ref()
            .and_then(|h| h.strip_prefix('@'))
            .and_then(|s| s.parse::<usize>().ok())
            && let Some(Some(flow)) = slots.get(index)
        {
            returned.kind = flow.kind;
            returned.hint = flow.hint.clone();
        }
        returned
    }
    fn builtin(&mut self, signature: &BuiltinSignature, args: &[CallArg], flows: &[Flow]) -> Flow {
        let names = signature
            .params
            .iter()
            .map(|p| p.name.to_owned())
            .collect::<Vec<_>>();
        let slots = self.argument_slots(&names, args, flows);
        for (param, flow) in signature.params.iter().zip(&slots) {
            if let Some(flow) = flow {
                let kinds = if flow.kind == ValueKind::Na {
                    vec![
                        ValueKind::Int,
                        ValueKind::Float,
                        ValueKind::Bool,
                        ValueKind::String,
                        ValueKind::Color,
                    ]
                } else {
                    vec![flow.kind]
                };
                if kinds.iter().any(|kind| {
                    accepts_type(param.accepts, PineType::new(Qualifier::Simple, *kind))
                        && !accepts_type(param.accepts, PineType::new(Qualifier::Series, *kind))
                }) {
                    self.require_simple(flow);
                }
            }
        }
        let types = flows
            .iter()
            .map(|f| Some(f.pine_type(&self.params)))
            .collect::<Vec<_>>();
        let ty = self
            .analyzer
            .return_type_for_call(signature, args, &types)
            .unwrap_or(UNKNOWN);
        let selected = match signature.returns {
            ReturnSpec::SameAsArg(i)
            | ReturnSpec::BoolFromArg(i)
            | ReturnSpec::ColorFromArg(i)
            | ReturnSpec::FloatFromStringArg(i)
            | ReturnSpec::IntFromArg(i)
            | ReturnSpec::FloatFromArg(i) => slots.get(i).and_then(Option::as_ref).cloned(),
            ReturnSpec::PromotedColor
            | ReturnSpec::PromotedBool
            | ReturnSpec::PromotedInt
            | ReturnSpec::PromotedString
            | ReturnSpec::PromotedNumeric
            | ReturnSpec::PromotedFloat
            | ReturnSpec::Round => Some(flows.iter().cloned().fold(Flow::default(), Flow::merge)),
            _ => None,
        };
        if let Some(mut flow) = selected {
            flow.kind = ty.kind;
            if !matches!(signature.returns, ReturnSpec::SameAsArg(_)) {
                flow.hint = Some(value_kind_name(ty.kind).to_owned());
            }
            return flow;
        }
        Flow::typed(ty)
    }
}
