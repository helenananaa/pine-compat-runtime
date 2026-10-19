mod flow;

use super::super::context::FunctionParamInfo;
use crate::prelude::*;
use flow::{Flow, Walker};
use std::collections::{BTreeSet, HashMap};

#[derive(Clone)]
struct Unit {
    key: String,
    names: Vec<String>,
    params: Vec<Option<FunctionParamInfo>>,
    defaults: Vec<Option<Expr>>,
    body: FunctionBody,
    context: crate::source_graph::SourceContextId,
}

#[derive(Clone, Default, PartialEq, Eq)]
struct Summary {
    simple: BTreeSet<usize>,
    returned: Flow,
}

fn function_key(name: &str, function: &FunctionInfo) -> String {
    format!(
        "f:{name}:{:?}:{}:{}",
        function.source_context_id, function.span.start, function.span.end
    )
}

fn auto_scalar(param: &FunctionParamInfo) -> bool {
    !param.explicit_simple
        && !param.explicit_series
        && param.pine_type.qualifier != Qualifier::Const
        && matches!(
            param.pine_type.kind,
            ValueKind::Int
                | ValueKind::Float
                | ValueKind::Bool
                | ValueKind::String
                | ValueKind::Color
        )
}

fn resolved(param: &FunctionParamInfo, simple: bool) -> FunctionParamInfo {
    let mut result = param.clone();
    result.enforce_qualifier = true;
    if simple && auto_scalar(param) {
        result.pine_type.qualifier = Qualifier::Simple;
    }
    result
}

fn imported_binding(param: &MethodParamInfo, span: Span) -> Option<FunctionParamInfo> {
    param.binding.clone().or_else(|| {
        (param.pine_type != UNKNOWN).then(|| FunctionParamInfo {
            pine_type: param.pine_type,
            explicit_series: false,
            explicit_simple: false,
            enforce_qualifier: false,
            user_type_name: param.user_type_name.clone(),
            map_type: None,
            span,
        })
    })
}

impl Analyzer {
    pub(crate) fn infer_parameter_qualifiers(&mut self, program: &Program) {
        if self.legacy.dialect() < crate::PineDialect::V5 {
            return;
        }
        let mut units = HashMap::new();
        let mut functions = HashMap::<String, Vec<String>>::new();
        for (name, group) in &self.functions {
            for function in std::iter::once(group).chain(group.overloads.iter()) {
                let key = function_key(name, function);
                functions.entry(name.clone()).or_default().push(key.clone());
                units.insert(
                    key.clone(),
                    Unit {
                        key,
                        names: function.params.clone(),
                        params: function.param_types.clone(),
                        defaults: function.default_values.clone(),
                        body: function.body.clone(),
                        context: function.source_context_id,
                    },
                );
            }
        }
        let mut methods = HashMap::new();
        for ((receiver, name), method) in &self.methods {
            let key = format!("m:{receiver}:{name}");
            let receiver_binding =
                method
                    .receiver_binding
                    .clone()
                    .unwrap_or_else(|| FunctionParamInfo {
                        pine_type: PineType::new(Qualifier::Series, ValueKind::UserType),
                        explicit_series: false,
                        explicit_simple: false,
                        enforce_qualifier: false,
                        user_type_name: Some(receiver.clone()),
                        map_type: None,
                        span: method.span,
                    });
            let names = std::iter::once(method.receiver_name.clone())
                .chain(method.params.iter().map(|p| p.name.clone()))
                .collect::<Vec<_>>();
            let params = std::iter::once(Some(receiver_binding))
                .chain(
                    method
                        .params
                        .iter()
                        .map(|p| imported_binding(p, method.span)),
                )
                .collect::<Vec<_>>();
            units.insert(
                key.clone(),
                Unit {
                    key: key.clone(),
                    defaults: vec![None; names.len()],
                    names,
                    params,
                    body: method.body.clone(),
                    context: method.source_context_id,
                },
            );
            methods.insert((receiver.clone(), name.clone()), key);
        }
        if units.is_empty() {
            return;
        }
        let mut summaries = units
            .keys()
            .map(|k| (k.clone(), Summary::default()))
            .collect::<HashMap<_, _>>();
        let limit = units.len()
            + units.values().map(|u| u.params.len()).sum::<usize>()
            + program.statements.len()
            + 2;
        for _ in 0..limit {
            let mut global =
                Walker::new(self, &units, &functions, &methods, &summaries, Vec::new());
            global.statements(&program.statements);
            let globals = global.env;
            let mut next = summaries.clone();
            for unit in units.values() {
                let old = &summaries[&unit.key];
                let params = unit
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, p)| p.as_ref().map(|p| resolved(p, old.simple.contains(&i))))
                    .collect::<Vec<_>>();
                let mut walker = Walker::new(
                    self,
                    &units,
                    &functions,
                    &methods,
                    &summaries,
                    params.clone(),
                );
                walker.env = globals.clone();
                for (i, name) in unit.names.iter().enumerate() {
                    walker
                        .env
                        .insert(name.clone(), Flow::parameter(i, params[i].as_ref()));
                }
                let returned =
                    self.with_source_context_ref(unit.context, |_| walker.body(&unit.body));
                let simple = old.simple.union(&walker.simple).copied().collect();
                next.insert(unit.key.clone(), Summary { simple, returned });
            }
            if next == summaries {
                break;
            }
            summaries = next;
        }
        for (name, group) in &mut self.functions {
            let apply = |function: &mut FunctionInfo| {
                let summary = &summaries[&function_key(name, function)];
                for (i, param) in function.param_types.iter_mut().enumerate() {
                    if let Some(p) = param {
                        *p = resolved(p, summary.simple.contains(&i));
                    }
                }
            };
            for function in &mut group.overloads {
                apply(function);
            }
            apply(group);
        }
        for (key, method) in &mut self.methods {
            let summary = &summaries[&methods[key]];
            if let Some(p) = &mut method.receiver_binding {
                *p = resolved(p, summary.simple.contains(&0));
            }
            for (i, param) in method.params.iter_mut().enumerate() {
                if let Some(p) = imported_binding(param, method.span) {
                    let p = resolved(&p, summary.simple.contains(&(i + 1)));
                    param.pine_type = p.pine_type;
                    param.binding = Some(p);
                }
            }
        }
    }
}
