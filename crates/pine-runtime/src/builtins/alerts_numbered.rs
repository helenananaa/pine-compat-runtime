use pine_ir::HirExprKind;

use crate::{HistoricalRuntime, PineValue};

impl HistoricalRuntime<'_> {
    pub(crate) fn render_numbered_alert_plots(&self, message: &str) -> Result<String, String> {
        if !message.contains("{{plot_") {
            return Ok(message.to_owned());
        }
        // Bind to declaration order, rather than the order in which runtime
        // outputs first happen to be observed. Mixed plot forms remain an
        // explicit capability boundary until their slot layouts are qualified.
        let mut plots = Vec::new();
        let mut mixed = false;
        crate::runtime::hir_walk::statements(&self.program.statements, &mut |expr| {
            if let HirExprKind::Call {
                callee,
                call_site_id,
                ..
            } = &expr.kind
            {
                match callee.as_str() {
                    "plot" => plots.push((call_site_id.0, mixed)),
                    "plotchar" | "plotshape" | "plotarrow" | "plotbar" | "plotcandle" => {
                        mixed = true
                    }
                    _ => {}
                }
            }
        });
        let mut rendered = String::with_capacity(message.len());
        let mut remaining = message;
        while let Some(start) = remaining.find("{{plot_") {
            rendered.push_str(&remaining[..start]);
            let tail = &remaining[start + "{{plot_".len()..];
            let end = tail
                .find("}}")
                .ok_or("unclosed numbered plot alert placeholder")?;
            let number = &tail[..end];
            let index = number
                .parse::<usize>()
                .ok()
                .filter(|index| *index < 20)
                .ok_or("numbered plot alert placeholder must use plot_0 through plot_19")?;
            let (id, mixed) = plots.get(index).ok_or_else(|| {
                format!("numbered plot alert placeholder has no plot_{index} declaration")
            })?;
            if *mixed {
                return Err(
                    "numbered plot alert placeholders crossing mixed plot forms are not qualified"
                        .to_owned(),
                );
            }
            let value = self
                .plots
                .iter()
                .find(|plot| plot.id == *id)
                .and_then(|plot| plot.values.get(self.bars - self.stored_origin));
            let text = match value {
                Some(PineValue::Int(value)) => value.to_string(),
                Some(PineValue::Float(value)) => {
                    crate::builtins::strings::format_number(*value, "")
                }
                Some(PineValue::Na) => "NaN".to_owned(),
                _ => {
                    return Err(format!(
                        "numbered plot alert placeholder has no current numeric value for plot_{index}"
                    ));
                }
            };
            rendered.push_str(&text);
            remaining = &tail[end + 2..];
        }
        rendered.push_str(remaining);
        Ok(rendered)
    }
}
