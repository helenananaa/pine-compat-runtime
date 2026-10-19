//! Read-only, bounded evaluation of scalar input metadata. No original user
//! blocks, loops, requests or chart state are executed by this evaluator.
use std::collections::{HashMap, HashSet};

use pine_ir::{
    DrawingSettings, HirBinaryOp, HirCallArg, HirExpr, HirExprKind, HirHistoryRequirements,
    HirLiteral, HirProgram, Qualifier, ScriptMode, StrategySettings, SymbolId,
};

use crate::runtime::expressions::{eval_binary_with_semantics, eval_literal, eval_unary};
use crate::{ExecutionLimits, HistoricalRuntime, PineValue};

const MAX_DEPTH: usize = 64;
const MAX_STEPS: u64 = 16_384;

pub(super) struct ConstEvaluator<'a> {
    initializers: HashMap<SymbolId, &'a HirExpr>,
    cached: HashMap<SymbolId, PineValue>,
    visiting: HashSet<SymbolId>,
    minimal: &'a HirProgram,
    runtime: Option<HistoricalRuntime<'a>>,
    remaining: u64,
    risk_context: Option<(&'a crate::InputOverrides, &'a crate::ChartContext)>,
}

impl<'a> ConstEvaluator<'a> {
    pub(super) fn new(program: &'a HirProgram, minimal: &'a HirProgram) -> Self {
        let constants: HashSet<_> = program
            .symbols
            .iter()
            .filter(|symbol| symbol.pine_type.qualifier == Qualifier::Const)
            .map(|symbol| symbol.id)
            .collect();
        let mut initializers = super::bindings::initializers(program);
        initializers.retain(|symbol, _| constants.contains(symbol));
        Self {
            initializers,
            cached: HashMap::new(),
            visiting: HashSet::new(),
            minimal,
            runtime: None,
            remaining: MAX_STEPS,
            risk_context: None,
        }
    }

    pub(super) fn for_risk(
        program: &'a HirProgram,
        minimal: &'a HirProgram,
        overrides: &'a crate::InputOverrides,
        chart: &'a crate::ChartContext,
    ) -> Self {
        let mut value = Self::new(program, minimal);
        let simple: HashSet<_> = program
            .symbols
            .iter()
            .filter(|symbol| symbol.pine_type.qualifier != Qualifier::Series)
            .map(|symbol| symbol.id)
            .collect();
        value.initializers = super::bindings::initializers(program);
        value.initializers.retain(|id, _| simple.contains(id));
        value.risk_context = Some((overrides, chart));
        value
    }

    pub(super) fn value(&mut self, expr: &HirExpr) -> Option<PineValue> {
        self.remaining = MAX_STEPS;
        self.eval(expr, 0)
    }

    pub(super) fn metadata_value(&mut self, expr: &HirExpr) -> Option<PineValue> {
        let value = self.value(expr)?.materialize_boolean();
        // Keep the public color input convention (a string accepted by color
        // overrides), while evaluating colors internally with runtime semantics.
        if let PineValue::Color(color) = value {
            let text = match &expr.kind {
                HirExprKind::Literal(HirLiteral::ColorHex(text)) | HirExprKind::Builtin(text) => {
                    text.clone()
                }
                _ => {
                    let (r, g, b, a) = crate::builtins::colors::color_rgba(color);
                    if a == 255 {
                        format!("#{r:02x}{g:02x}{b:02x}")
                    } else {
                        format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
                    }
                }
            };
            Some(PineValue::String(text))
        } else {
            Some(value)
        }
    }

    fn eval(&mut self, expr: &HirExpr, depth: usize) -> Option<PineValue> {
        if depth >= MAX_DEPTH || self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        let next = depth + 1;
        match &expr.kind {
            HirExprKind::Literal(literal) => Some(eval_literal(literal)),
            HirExprKind::Builtin(name) => {
                if name == "na" {
                    return Some(PineValue::Na);
                }
                if let Some((_, chart)) = self.risk_context {
                    let contextual = match name.as_str() {
                        "syminfo.mintick" => Some(PineValue::Float(chart.min_tick())),
                        "syminfo.minmove" => Some(PineValue::Int(i64::from(chart.min_move()))),
                        "syminfo.pricescale" => {
                            Some(PineValue::Int(i64::from(chart.price_scale())))
                        }
                        "syminfo.mincontract" => Some(PineValue::Float(chart.min_contract())),
                        "syminfo.currency" => Some(PineValue::String(chart.currency().to_owned())),
                        "syminfo.tickerid" => Some(PineValue::String(chart.symbol().to_owned())),
                        _ => None,
                    };
                    if contextual.is_some() {
                        return contextual;
                    }
                }
                let value = crate::builtins::variables::eval_static_builtin_value(name);
                (!matches!(value, PineValue::Void)).then_some(value)
            }
            HirExprKind::Symbol(symbol) => {
                if let Some(value) = self.cached.get(symbol) {
                    return Some(value.clone());
                }
                let initializer = *self.initializers.get(symbol)?;
                if !self.visiting.insert(*symbol) {
                    return None;
                }
                let value = self.eval(initializer, next);
                self.visiting.remove(symbol);
                // Unknown may mean this expression exhausted its own depth or
                // work budget. It must not poison a later, shorter reference.
                if let Some(value) = &value {
                    self.cached.insert(*symbol, value.clone());
                }
                value
            }
            HirExprKind::Unary { op, expr } => Some(eval_unary(*op, self.eval(expr, next)?)),
            HirExprKind::Binary { op, left, right } => {
                let left = self.eval(left, next)?;
                let v6 = self.minimal.language_version.is_some_and(|v| v >= 6);
                if v6
                    && ((*op == HirBinaryOp::And && left == PineValue::Bool(false))
                        || (*op == HirBinaryOp::Or && left.is_condition_true()))
                {
                    return Some(left);
                }
                let right = self.eval(right, next)?;
                eval_binary_with_semantics(*op, left, right, v6).ok()
            }
            HirExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => match self.eval(condition, next)? {
                PineValue::Bool(true) | PineValue::ConstantBool { .. } => {
                    self.eval(then_expr, next)
                }
                PineValue::Bool(false) | PineValue::Na => self.eval(else_expr, next),
                _ => None,
            },
            HirExprKind::Call {
                callee,
                call_site_id,
                args,
            } => {
                if let Some((overrides, _)) = self.risk_context
                    && (callee == "input" || callee.starts_with("input."))
                {
                    return overrides.get(*call_site_id).cloned().or_else(|| {
                        super::input_arg(args, 0, "defval").and_then(|expr| self.eval(expr, next))
                    });
                }
                let mut literals = Vec::with_capacity(args.len());
                for arg in args {
                    let value = self.eval(&arg.value, next)?;
                    literals.push(HirCallArg {
                        name: arg.name.clone(),
                        value: literal_expr(&arg.value, value)?,
                    });
                }
                if !pure_call(callee, &literals) {
                    return None;
                }
                let call = HirExpr {
                    pine_type: expr.pine_type,
                    series_id: None,
                    kind: HirExprKind::Call {
                        callee: callee.clone(),
                        call_site_id: *call_site_id,
                        args: literals,
                    },
                };
                let runtime = self.runtime.get_or_insert_with(|| {
                    HistoricalRuntime::new(self.minimal).with_execution_limits(ExecutionLimits {
                        max_steps_per_bar: MAX_STEPS,
                        max_loop_iterations_per_bar: 0,
                    })
                });
                runtime.reset_execution_budget();
                runtime.eval_expr(&call).ok()
            }
            HirExprKind::Block { statements, result }
                if self.risk_context.is_some()
                    && statements.iter().all(|statement| {
                        matches!(
                            statement.kind,
                            pine_ir::HirStmtKind::Decl { .. }
                                | pine_ir::HirStmtKind::TupleDecl { .. }
                        )
                    }) =>
            {
                self.eval(result, next)
            }
            HirExprKind::Switch { selector, arms } if self.risk_context.is_some() => {
                let selector = match selector {
                    Some(expr) => Some(self.eval(expr, next)?),
                    None => None,
                };
                for arm in arms {
                    let matches = match &arm.condition {
                        None => true,
                        Some(condition) => {
                            let value = self.eval(condition, next)?;
                            selector.as_ref().map_or_else(
                                || value.is_condition_true(),
                                |selector| selector == &value,
                            )
                        }
                    };
                    if matches {
                        return self.eval(&arm.result, next);
                    }
                }
                Some(PineValue::Na)
            }
            // HIR supplied by callers may contain arbitrary execution nodes.
            _ => None,
        }
    }
}

fn literal_expr(original: &HirExpr, value: PineValue) -> Option<HirExpr> {
    if let PineValue::ConstantBool { value, identity } = value {
        return Some(HirExpr {
            kind: HirExprKind::Unary {
                op: pine_ir::HirUnaryOp::ConstantBoolIdentity(identity),
                expr: Box::new(HirExpr {
                    kind: HirExprKind::Literal(HirLiteral::Bool(value)),
                    series_id: None,
                    pine_type: original.pine_type,
                }),
            },
            series_id: None,
            pine_type: original.pine_type,
        });
    }
    if matches!(value, PineValue::Na) {
        return Some(HirExpr {
            kind: HirExprKind::Builtin("na".to_owned()),
            series_id: None,
            pine_type: original.pine_type,
        });
    }
    let literal = match value {
        PineValue::Int(v) => HirLiteral::Int(v),
        PineValue::Float(v) => HirLiteral::Float(v),
        PineValue::Bool(v) => HirLiteral::Bool(v),
        PineValue::String(v) => HirLiteral::String(v),
        PineValue::Color(v) => {
            let (r, g, b, a) = crate::builtins::colors::color_rgba(v);
            HirLiteral::ColorHex(format!("#{r:02x}{g:02x}{b:02x}{a:02x}"))
        }
        _ => return None,
    };
    Some(HirExpr {
        kind: HirExprKind::Literal(literal),
        series_id: None,
        pine_type: original.pine_type,
    })
}

fn pure_call(callee: &str, args: &[HirCallArg]) -> bool {
    match callee {
        "int"
        | "float"
        | "bool"
        | "string"
        | "color"
        | "na"
        | "nz"
        | "math.abs"
        | "math.max"
        | "math.min"
        | "math.avg"
        | "math.floor"
        | "math.ceil"
        | "math.trunc"
        | "math.sqrt"
        | "math.cbrt"
        | "math.log"
        | "math.log10"
        | "math.exp"
        | "math.acos"
        | "math.asin"
        | "math.atan"
        | "math.sign"
        | "math.todegrees"
        | "math.toradians"
        | "math.sin"
        | "math.cos"
        | "math.tan"
        | "math.pow"
        | "math.hypot"
        | "math.round"
        | "color.new"
        | "color.rgb"
        | "color.r"
        | "color.g"
        | "color.b"
        | "color.t"
        | "color.from_gradient"
        | "str.length"
        | "str.upper"
        | "str.lower"
        | "str.contains"
        | "str.startswith"
        | "str.endswith"
        | "str.pos"
        | "str.substring"
        | "str.trim"
        | "str.repeat"
        | "str.replace"
        | "str.tonumber" => true,
        "str.replace_all" => replacement_output_is_bounded(args),
        "str.tostring" => !uses_chart_tick(args) && formatting_output_is_bounded(args, false),
        "str.format" => !uses_chart_tick(args) && formatting_output_is_bounded(args, true),
        "str.format_time" => args.get(2).is_some_and(|arg| {
            matches!(&arg.value.kind, HirExprKind::Literal(HirLiteral::String(_)))
        }),
        "timestamp" => {
            args.iter().any(|arg| {
                arg.name.as_deref() == Some("timezone") || arg.name.as_deref() == Some("dateString")
            }) || args.first().is_some_and(|arg| {
                arg.name.is_none()
                    && matches!(arg.value.kind, HirExprKind::Literal(HirLiteral::String(_)))
            })
        }
        _ => false,
    }
}

fn uses_chart_tick(args: &[HirCallArg]) -> bool {
    args.iter().any(|arg| matches!(&arg.value.kind,
        HirExprKind::Literal(HirLiteral::String(value)) if value.contains("format.mintick") || value == "mintick"))
}

fn string_arg(args: &[HirCallArg], index: usize) -> Option<&str> {
    match &args.get(index)?.value.kind {
        HirExprKind::Literal(HirLiteral::String(value)) => Some(value),
        _ => None,
    }
}

fn replacement_output_is_bounded(args: &[HirCallArg]) -> bool {
    let (Some(source), Some(target), Some(replacement)) = (
        string_arg(args, 0),
        string_arg(args, 1),
        string_arg(args, 2),
    ) else {
        return false;
    };
    let count = if target.is_empty() {
        source.chars().count().saturating_add(1)
    } else {
        source.matches(target).count()
    };
    // This deliberately retains removed characters in the upper bound. A
    // rejected bound stays Unknown instead of allocating the expanded output.
    source
        .chars()
        .count()
        .saturating_add(count.saturating_mul(replacement.chars().count()))
        <= crate::MAX_STRING_CHARS
}

fn formatting_output_is_bounded(args: &[HirCallArg], template: bool) -> bool {
    if !template {
        if let Some(source) = string_arg(args, 0) {
            return source.chars().count() <= crate::MAX_STRING_CHARS;
        }
        // Decimal digits, grouping and literal suffixes are bounded by the
        // pattern size plus the finite f64 exponent range.
        let pattern = string_arg(args, 1).unwrap_or("#.########");
        return pattern
            .chars()
            .count()
            .saturating_mul(32)
            .saturating_add(512)
            <= crate::MAX_STRING_CHARS;
    }
    let Some(pattern) = string_arg(args, 0) else {
        return false;
    };
    let count = pattern.chars().filter(|ch| *ch == '{').count();
    let scalar = args
        .iter()
        .skip(1)
        .map(|arg| match &arg.value.kind {
            HirExprKind::Literal(HirLiteral::String(value)) => value.chars().count(),
            _ => 512,
        })
        .max()
        .unwrap_or(0);
    // Padding, separators and UTC date/time pattern expansion are covered by
    // this conservative factor; each placeholder may also copy a scalar value.
    pattern
        .chars()
        .count()
        .saturating_mul(32)
        .saturating_add(count.saturating_mul(scalar))
        <= crate::MAX_STRING_CHARS
}

pub(super) fn minimal_program(language_version: Option<u16>) -> HirProgram {
    HirProgram {
        language_version,
        dynamic_requests: language_version.is_some_and(|version| version >= 6),
        script_mode: ScriptMode::Indicator,
        timenow_symbol: None,
        strategy_settings: StrategySettings::default(),
        drawing_settings: DrawingSettings::default(),
        user_types: Vec::new(),
        symbols: Vec::new(),
        statements: Vec::new(),
        next_series_id: 0,
        next_call_site_id: 0,
        call_site_sources: Vec::new(),
        lower_tf_tuple_types: Vec::new(),
        next_var_slot_id: 0,
        max_bars_back: None,
        calc_bars_count: None,
        series_max_bars_back: Vec::new(),
        history: HirHistoryRequirements::default(),
        series_history: Vec::new(),
        execution_scoped_series: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn constant_boolean_identity_keeps_runtime_condition_semantics() {
        let program = minimal_program(Some(6));
        let literal = |kind, value| HirExpr {
            kind: HirExprKind::Literal(value),
            series_id: None,
            pine_type: pine_ir::PineType::new(Qualifier::Const, kind),
        };
        let condition = literal(pine_ir::ValueKind::Bool, HirLiteral::Bool(false));
        let expr = HirExpr {
            pine_type: pine_ir::PineType::new(Qualifier::Const, pine_ir::ValueKind::Int),
            series_id: None,
            kind: HirExprKind::Ternary {
                condition: Box::new(HirExpr {
                    pine_type: condition.pine_type,
                    series_id: None,
                    kind: HirExprKind::Unary {
                        op: pine_ir::HirUnaryOp::ConstantBoolIdentity(Some(7)),
                        expr: Box::new(condition),
                    },
                }),
                then_expr: Box::new(literal(pine_ir::ValueKind::Int, HirLiteral::Int(9))),
                else_expr: Box::new(literal(pine_ir::ValueKind::Int, HirLiteral::Int(2))),
            },
        };
        assert_eq!(
            ConstEvaluator::new(&program, &program).value(&expr),
            Some(PineValue::Int(9))
        );
    }
}
