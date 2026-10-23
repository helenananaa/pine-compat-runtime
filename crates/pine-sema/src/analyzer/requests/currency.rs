use crate::prelude::*;

impl Analyzer {
    pub(super) fn analyze_currency_rate(
        &mut self,
        span: Span,
        args: &[CallArg],
    ) -> Option<PineType> {
        let signature = pine_builtins::get_phase_1_builtin("request.currency_rate")
            .expect("currency rate signature");
        let arg_types: Vec<_> = args
            .iter()
            .map(|arg| self.analyze_expr(&arg.value))
            .collect();
        self.validate_call_args(signature, args, &arg_types);
        if !self.dynamic_requests {
            for (index, arg) in args.iter().enumerate() {
                let selector = match arg.name.as_deref() {
                    Some("from" | "to") => true,
                    Some(_) => false,
                    None => index < 2,
                };
                let ignore_flag = arg.name.as_deref() == Some("ignore_invalid_currency")
                    || (arg.name.is_none() && index == 2);
                if (selector || ignore_flag)
                    && arg_types[index]
                        .is_some_and(|ty| !qualifier_at_most(ty.qualifier, Qualifier::Simple))
                {
                    let message = if selector {
                        "`request.currency_rate` currency arguments require simple strings when dynamic_requests=false"
                    } else {
                        "`request.currency_rate` ignore_invalid_currency requires a simple bool when dynamic_requests=false"
                    };
                    self.diagnostics
                        .push(Diagnostic::error("E_CALL_ARG_TYPE", message, arg.span));
                }
            }
        }
        self.compatibility.supported.push(FeatureUse {
            feature: "request.currency_rate".to_owned(),
            span,
        });
        Some(PineType::new(Qualifier::Series, ValueKind::Float))
    }
}
