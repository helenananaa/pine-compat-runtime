//! Bind static call namespaces once, retaining manual-HIR compatibility.
use std::collections::HashMap;

use pine_ir::{CallSiteId, HirExprKind, HirProgram};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CallFamily {
    Legacy,
    Variable,
    RuntimeError,
    Alert,
    Output,
    Drawing,
    ChartPoint,
    Request,
    Strategy,
    Color,
    String,
    Syminfo,
    Ticker,
    Time,
    Cast,
    Math,
    Ta,
    Array,
    Map,
    Matrix,
    Unsupported,
}

impl CallFamily {
    pub(crate) fn for_name(callee: &str) -> Self {
        match callee {
            "$legacy.iff" | "$legacy.rsi_series" => Self::Legacy,
            "$legacy.security.gaps_off.lookahead_off"
            | "$legacy.security.gaps_on.lookahead_off"
            | "$legacy.security.gaps_off.lookahead_on"
            | "$legacy.security.gaps_on.lookahead_on" => Self::Request,
            "indicator" | "strategy" | "max_bars_back" | "input" | "na" | "nz" | "fixnan" => {
                Self::Variable
            }
            "runtime.error" => Self::RuntimeError,
            "alert" | "alertcondition" => Self::Alert,
            "plot" | "plotchar" | "plotshape" | "plotarrow" | "plotbar" | "plotcandle"
            | "bgcolor" | "barcolor" | "hline" | "fill" => Self::Output,
            "year" | "month" | "weekofyear" | "dayofmonth" | "dayofweek" | "hour" | "minute"
            | "second" | "timestamp" | "time" | "time_close" => Self::Time,
            "int" | "float" | "bool" | "string" | "box" | "color" | "label" | "line"
            | "linefill" | "polyline" | "table" => Self::Cast,
            _ if callee.starts_with("chart.point.") => Self::ChartPoint,
            _ => match callee.split_once('.').map(|(namespace, _)| namespace) {
                Some("input") => Self::Variable,
                Some("label" | "line" | "linefill" | "polyline" | "box" | "table") => Self::Drawing,
                Some("request") => Self::Request,
                Some("strategy") => Self::Strategy,
                Some("color") => Self::Color,
                Some("str") => Self::String,
                Some("syminfo") => Self::Syminfo,
                Some("ticker") => Self::Ticker,
                Some("timeframe") => Self::Time,
                Some("math") => Self::Math,
                Some("ta") => Self::Ta,
                Some("array") => Self::Array,
                Some("map") => Self::Map,
                Some("matrix") => Self::Matrix,
                _ => Self::Unsupported,
            },
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct CallPlan {
    bindings: HashMap<CallSiteId, Option<(String, CallFamily)>>,
}

impl CallPlan {
    pub(crate) fn from_program(program: &HirProgram) -> Self {
        let mut plan = Self::default();
        super::hir_walk::statements(&program.statements, &mut |expr| {
            if let HirExprKind::Call {
                callee,
                call_site_id,
                ..
            } = &expr.kind
            {
                let family = CallFamily::for_name(callee);
                plan.bindings
                    .entry(*call_site_id)
                    .and_modify(|binding| {
                        if binding.as_ref().is_some_and(|(name, _)| name != callee) {
                            *binding = None;
                        }
                    })
                    .or_insert_with(|| Some((callee.clone(), family)));
            }
        });
        plan
    }

    pub(crate) fn family(&self, site: CallSiteId, callee: &str) -> CallFamily {
        self.bindings
            .get(&site)
            .and_then(Option::as_ref)
            .filter(|(name, _)| name == callee)
            .map_or_else(|| CallFamily::for_name(callee), |(_, family)| *family)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conflicting_manual_call_site_ids_do_not_dispatch_to_the_wrong_namespace() {
        let source = pine_syntax::SourceFile::new(
            "calls.pine",
            "//@version=6\nindicator(\"calls\")\nplot(math.abs(close))\n",
        );
        let mut program = pine_sema::analyze_source(&source).hir.unwrap();
        for statement in &mut program.statements {
            if let pine_ir::HirStmtKind::Expr(expr) = &mut statement.kind
                && let HirExprKind::Call { call_site_id, .. } = &mut expr.kind
            {
                *call_site_id = CallSiteId(0);
            }
        }
        let plan = CallPlan::from_program(&program);
        assert!(matches!(plan.bindings.get(&CallSiteId(0)), Some(None)));
        assert_eq!(plan.family(CallSiteId(u32::MAX), "ta.sma"), CallFamily::Ta);
        assert_eq!(plan.family(CallSiteId(0), "str.length"), CallFamily::String);
        assert_eq!(CallFamily::for_name("strategy"), CallFamily::Variable);
        for gaps in ["off", "on"] {
            for lookahead in ["off", "on"] {
                assert_eq!(
                    CallFamily::for_name(&format!(
                        "$legacy.security.gaps_{gaps}.lookahead_{lookahead}"
                    )),
                    CallFamily::Request
                );
            }
        }
        assert_eq!(CallFamily::for_name("strategy.entry"), CallFamily::Strategy);
        assert_eq!(
            CallFamily::for_name("chart.point.new"),
            CallFamily::ChartPoint
        );
    }
}
