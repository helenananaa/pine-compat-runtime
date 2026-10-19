use crate::prelude::*;

fn unsupported_alert_placeholder(value: &str, supported: &[&str]) -> Option<String> {
    let mut remaining = value;
    while let Some(start) = remaining.find("{{") {
        let placeholder_tail = &remaining[start..];
        let Some(relative_end) = placeholder_tail.find("}}") else {
            return Some("{{".to_owned());
        };
        let end = relative_end + 2;
        let placeholder = &placeholder_tail[..end];
        let named_plot = placeholder
            .strip_prefix("{{plot(\"")
            .and_then(|title| title.strip_suffix("\")}}"))
            .is_some_and(|title| !title.is_empty() && !title.contains('"'));
        let numbered_plot = placeholder
            .strip_prefix("{{plot_")
            .and_then(|number| number.strip_suffix("}}"))
            .is_some_and(|number| (0..20).any(|index| number == index.to_string()));
        if !(supported.contains(&placeholder)
            || (named_plot || numbered_plot) && !supported.is_empty())
        {
            return Some(placeholder.to_owned());
        }
        remaining = &placeholder_tail[end..];
    }
    None
}

impl Analyzer {
    pub(crate) fn validate_alert_args(&mut self, signature: &BuiltinSignature, args: &[CallArg]) {
        if !matches!(signature.name, "alert" | "alertcondition") {
            return;
        }

        for (index, arg) in args.iter().enumerate() {
            let Some(param_name) = self
                .resolve_param(signature, index, arg)
                .map(|param| param.name)
            else {
                continue;
            };

            // Frequency is an input string. Its finite value domain is
            // validated when the alert executes, including literal strings.

            // alert() messages and alertcondition() titles are plain strings,
            // including braces that resemble placeholders. Only the message
            // of alertcondition() participates in placeholder interpolation.
            if signature.name == "alertcondition" && param_name == "message" {
                let supported_placeholders =
                    if matches!(self.script_declaration, Some((ScriptMode::Strategy, _))) {
                        &[
                            "{{open}}",
                            "{{high}}",
                            "{{low}}",
                            "{{close}}",
                            "{{volume}}",
                            "{{ticker}}",
                            "{{interval}}",
                            "{{exchange}}",
                            "{{time}}",
                            "{{timenow}}",
                            "{{strategy.position_size}}",
                        ][..]
                    } else {
                        &[
                            "{{open}}",
                            "{{high}}",
                            "{{low}}",
                            "{{close}}",
                            "{{volume}}",
                            "{{ticker}}",
                            "{{interval}}",
                            "{{exchange}}",
                            "{{time}}",
                            "{{timenow}}",
                        ][..]
                    };

                if let Some(placeholder) = self
                    .known_const_string_value(&arg.value)
                    .as_deref()
                    .and_then(|value| unsupported_alert_placeholder(value, supported_placeholders))
                {
                    self.unsupported(
                        "alert_placeholders",
                        &format!(
                            "alert placeholder `{placeholder}` is not supported in the current alert subset"
                        ),
                        arg.span,
                    );
                }
            }
        }
    }
}
