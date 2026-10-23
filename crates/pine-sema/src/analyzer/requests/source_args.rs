use crate::prelude::*;

pub(crate) fn source_input_default(args: &[CallArg]) -> Option<&Expr> {
    args.iter()
        .find(|arg| arg.name.as_deref() == Some("defval"))
        .or_else(|| args.first().filter(|arg| arg.name.is_none()))
        .map(|arg| &arg.value)
}

pub(crate) fn security_source_expression(args: &[CallArg]) -> Option<&Expr> {
    args.iter()
        .find(|arg| arg.name.as_deref() == Some("expression"))
        .or_else(|| args.get(2).filter(|arg| arg.name.is_none()))
        .map(|arg| &arg.value)
}

/// Bind the three selector/expression slots without changing optional policies.
pub(crate) fn normalize_security_source_args(
    args: &[CallArg],
    span: Span,
) -> Result<Vec<CallArg>, Diagnostic> {
    const REQUIRED: [&str; 3] = ["symbol", "timeframe", "expression"];
    let mut slots: [Option<CallArg>; 3] = [None, None, None];
    let mut optional = Vec::new();
    let mut seen_named = false;
    for (index, arg) in args.iter().enumerate() {
        if arg.name.is_some() {
            seen_named = true;
        } else if seen_named {
            return Err(Diagnostic::error(
                "E_CALL_ARG_ORDER",
                "positional arguments cannot follow named arguments in built-in calls",
                arg.span,
            ));
        }
        let slot = match arg.name.as_deref() {
            Some(name) => REQUIRED.iter().position(|candidate| *candidate == name),
            None if index < 3 => Some(index),
            None => None,
        };
        if let Some(slot) = slot {
            if slots[slot].is_some() {
                return Err(Diagnostic::error(
                    "E_CALL_ARG_DUPLICATE",
                    format!(
                        "request argument `{}` is supplied more than once",
                        REQUIRED[slot]
                    ),
                    arg.span,
                ));
            }
            let mut bound = arg.clone();
            bound.name = None;
            slots[slot] = Some(bound);
        } else {
            optional.push(arg.clone());
        }
    }
    let mut result = Vec::with_capacity(args.len());
    for (name, slot) in REQUIRED.into_iter().zip(slots) {
        result.push(slot.ok_or_else(|| {
            Diagnostic::error(
                "E_CALL_ARITY",
                format!("request is missing argument `{name}`"),
                span,
            )
        })?);
    }
    result.extend(optional);
    Ok(result)
}
