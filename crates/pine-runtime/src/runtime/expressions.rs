use pine_ir::{HirBinaryOp, HirExpr, HirExprKind, HirLiteral, HirUnaryOp};

use crate::builtins::colors::parse_color_hex;
use crate::*;

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_expr(&mut self, expr: &HirExpr) -> Result<PineValue, RuntimeError> {
        if self.eval_expr_depth >= MAX_RUNTIME_EVAL_DEPTH {
            return Err(RuntimeError {
                message: "runtime expression evaluation exceeded maximum depth".to_owned(),
            });
        }

        self.eval_expr_depth += 1;
        let result = self.eval_expr_inner(expr);
        self.eval_expr_depth -= 1;

        let value = result?;
        if let Some(series_id) = expr.series_id {
            self.activate_bar_aligned_series(series_id);
            self.current_series.insert(series_id, value.clone());
        }

        Ok(value)
    }

    fn eval_expr_inner(&mut self, expr: &HirExpr) -> Result<PineValue, RuntimeError> {
        let value = match &expr.kind {
            HirExprKind::Literal(literal) => eval_literal(literal),
            HirExprKind::Symbol(symbol) if self.program.timenow_symbol == Some(*symbol) => self
                .current_execution_time
                .map(PineValue::Int)
                .ok_or_else(|| RuntimeError {
                    message:
                        "timenow requires an explicit execution timestamp for this script execution"
                            .to_owned(),
                })?,
            HirExprKind::Symbol(symbol) => self
                .current_symbols
                .get(symbol)
                .cloned()
                .unwrap_or(PineValue::Na),
            HirExprKind::Builtin(name) => self.eval_builtin_value(name),
            HirExprKind::Unary { op, expr } => {
                let value = self.eval_expr(expr)?;
                eval_unary(*op, value)
            }
            HirExprKind::Binary { op, left, right } => {
                if self.uses_v6_semantics() {
                    match op {
                        HirBinaryOp::And => {
                            let left = self.eval_expr(left)?;
                            if matches!(left, PineValue::Bool(false)) {
                                return Ok(PineValue::Bool(false));
                            }
                            let right = self.eval_expr(right)?;
                            return eval_binary_with_semantics(*op, left, right, true);
                        }
                        HirBinaryOp::Or => {
                            let left = self.eval_expr(left)?;
                            if matches!(left, PineValue::Bool(true)) {
                                return Ok(PineValue::Bool(true));
                            }
                            let right = self.eval_expr(right)?;
                            return eval_binary_with_semantics(*op, left, right, true);
                        }
                        _ => {}
                    }
                }
                let left = self.eval_expr(left)?;
                let right = self.eval_expr(right)?;
                eval_binary_with_semantics(*op, left, right, self.uses_v6_semantics())?
            }
            HirExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => match self.eval_expr(condition)? {
                PineValue::Bool(true) => self.eval_expr(then_expr)?,
                PineValue::Bool(false) | PineValue::Na => self.eval_expr(else_expr)?,
                _ => PineValue::Na,
            },
            HirExprKind::Switch { selector, arms } => {
                self.eval_switch(selector.as_deref(), arms)?
            }
            HirExprKind::For {
                counter,
                from,
                to,
                step,
                statements,
                result,
            } => self.eval_for_loop(
                *counter,
                from,
                to,
                step.as_deref(),
                statements,
                Some(result),
            )?,
            HirExprKind::ForIn {
                index,
                value,
                iterable,
                statements,
                result,
            } => self.eval_for_in_expr(*index, *value, iterable, statements, result)?,
            HirExprKind::While {
                condition,
                statements,
                result,
            } => self.eval_while_loop(condition, statements, Some(result))?,
            HirExprKind::Tuple(items) => PineValue::Tuple(
                items
                    .iter()
                    .map(|item| self.eval_expr(item))
                    .collect::<Result<_, _>>()?,
            ),
            HirExprKind::UserTypeConstruct { fields, .. } => PineValue::UserType(
                fields
                    .iter()
                    .map(|field| self.eval_expr(field))
                    .collect::<Result<_, _>>()?,
            ),
            HirExprKind::UserTypeArrayConstruct {
                type_name,
                elements,
            } => self.eval_user_type_array_construct(type_name, elements)?,
            HirExprKind::FieldAccess { value, index } => match self.eval_expr(value)? {
                PineValue::UserType(fields) => fields.get(*index).cloned().unwrap_or(PineValue::Na),
                PineValue::ChartPoint(point) => point.field(*index),
                PineValue::Na => PineValue::Na,
                _ => {
                    return Err(RuntimeError {
                        message: "field access receiver is not an object value".to_owned(),
                    });
                }
            },
            HirExprKind::Block { statements, result } => {
                for statement in statements {
                    match self.eval_stmt(statement)? {
                        StmtControl::None => {}
                        StmtControl::Break => return Err(RuntimeError::loop_break()),
                        StmtControl::Continue => return Err(RuntimeError::loop_continue()),
                    }
                }
                self.eval_expr(result)?
            }
            HirExprKind::Call {
                callee,
                call_site_id,
                args,
            } => self.eval_call(callee, *call_site_id, args)?,
            HirExprKind::History { expr, offset } => self.eval_history(expr, offset)?,
        };

        Ok(value)
    }

    fn eval_user_type_array_construct(
        &mut self,
        type_name: &str,
        elements: &[HirExpr],
    ) -> Result<PineValue, RuntimeError> {
        if elements.len() > MAX_ARRAY_ELEMENTS {
            return Err(RuntimeError {
                message: format!(
                    "user-defined type array cannot exceed {MAX_ARRAY_ELEMENTS} elements"
                ),
            });
        }
        let values = elements
            .iter()
            .map(|element| self.eval_expr(element))
            .collect::<Result<_, _>>()?;
        Ok(self.new_user_type_array_from_values(type_name.to_owned(), values))
    }

    pub(crate) fn eval_switch(
        &mut self,
        selector: Option<&HirExpr>,
        arms: &[pine_ir::HirSwitchArm],
    ) -> Result<PineValue, RuntimeError> {
        let selector_value = match selector {
            Some(selector) => Some(self.eval_expr(selector)?),
            None => None,
        };

        for arm in arms {
            let matches = match (&selector_value, &arm.condition) {
                (Some(selector_value), Some(case_expr)) => {
                    let case_value = self.eval_expr(case_expr)?;
                    matches!(
                        eval_binary_with_semantics(
                            HirBinaryOp::Eq,
                            selector_value.clone(),
                            case_value,
                            self.uses_v6_semantics(),
                        )?,
                        PineValue::Bool(true)
                    )
                }
                (None, Some(condition)) => {
                    matches!(self.eval_expr(condition)?, PineValue::Bool(true))
                }
                (_, None) => true,
            };

            if matches {
                return self.eval_expr(&arm.result);
            }
        }

        Ok(PineValue::Na)
    }
}

pub(crate) fn eval_literal(literal: &HirLiteral) -> PineValue {
    match literal {
        HirLiteral::Int(value) => PineValue::Int(*value),
        HirLiteral::Float(value) => PineValue::Float(*value),
        HirLiteral::Bool(value) => PineValue::Bool(*value),
        HirLiteral::String(value) => PineValue::String(value.clone()),
        HirLiteral::ColorHex(value) => PineValue::Color(parse_color_hex(value)),
    }
}

pub(crate) fn eval_unary(op: HirUnaryOp, value: PineValue) -> PineValue {
    if value.is_na() {
        return PineValue::Na;
    }

    match op {
        HirUnaryOp::Plus => value,
        HirUnaryOp::Minus => match value {
            PineValue::Int(value) => value
                .checked_neg()
                .map_or_else(|| finite_float_or_na(-(value as f64)), PineValue::Int),
            PineValue::Float(value) => PineValue::Float(-value),
            _ => PineValue::Na,
        },
        HirUnaryOp::Not => match value {
            PineValue::Bool(value) => PineValue::Bool(!value),
            _ => PineValue::Na,
        },
    }
}

pub(crate) fn eval_binary(
    op: HirBinaryOp,
    left: PineValue,
    right: PineValue,
) -> Result<PineValue, RuntimeError> {
    eval_binary_with_semantics(op, left, right, false)
}

pub(crate) fn eval_binary_with_semantics(
    op: HirBinaryOp,
    left: PineValue,
    right: PineValue,
    uses_v6_semantics: bool,
) -> Result<PineValue, RuntimeError> {
    Ok(match op {
        HirBinaryOp::And => eval_logical_and(left, right, uses_v6_semantics),
        HirBinaryOp::Or => eval_logical_or(left, right, uses_v6_semantics),
        HirBinaryOp::Eq
        | HirBinaryOp::NotEq
        | HirBinaryOp::Gt
        | HirBinaryOp::Gte
        | HirBinaryOp::Lt
        | HirBinaryOp::Lte
            if left.is_na() || right.is_na() =>
        {
            if uses_v6_semantics {
                PineValue::Bool(false)
            } else {
                PineValue::Na
            }
        }
        _ if left.is_na() || right.is_na() => PineValue::Na,
        HirBinaryOp::Add => add(left, right)?,
        HirBinaryOp::Sub => numeric_sub(left, right),
        HirBinaryOp::Mul => numeric_mul(left, right),
        HirBinaryOp::Div => numeric_float_binary(left, right, |left, right| left / right),
        HirBinaryOp::Mod => numeric_mod(left, right),
        HirBinaryOp::Eq => PineValue::Bool(values_equal(&left, &right)),
        HirBinaryOp::NotEq => PineValue::Bool(!values_equal(&left, &right)),
        HirBinaryOp::Gt => compare_binary(left, right, |left, right| left > right),
        HirBinaryOp::Gte => compare_binary(left, right, |left, right| left >= right),
        HirBinaryOp::Lt => compare_binary(left, right, |left, right| left < right),
        HirBinaryOp::Lte => compare_binary(left, right, |left, right| left <= right),
    })
}

fn eval_logical_and(left: PineValue, right: PineValue, uses_v6_semantics: bool) -> PineValue {
    match (left, right) {
        (PineValue::Bool(false), _) | (_, PineValue::Bool(false)) => PineValue::Bool(false),
        (PineValue::Bool(true), PineValue::Bool(true)) => PineValue::Bool(true),
        (PineValue::Na, PineValue::Na)
        | (PineValue::Na, PineValue::Bool(true))
        | (PineValue::Bool(true), PineValue::Na) => {
            if uses_v6_semantics {
                PineValue::Bool(false)
            } else {
                PineValue::Na
            }
        }
        _ => PineValue::Na,
    }
}

fn eval_logical_or(left: PineValue, right: PineValue, uses_v6_semantics: bool) -> PineValue {
    match (left, right) {
        (PineValue::Bool(true), _) | (_, PineValue::Bool(true)) => PineValue::Bool(true),
        (PineValue::Bool(false), PineValue::Bool(false)) => PineValue::Bool(false),
        (PineValue::Na, PineValue::Na)
        | (PineValue::Na, PineValue::Bool(false))
        | (PineValue::Bool(false), PineValue::Na) => {
            if uses_v6_semantics {
                PineValue::Bool(false)
            } else {
                PineValue::Na
            }
        }
        _ => PineValue::Na,
    }
}

fn add(left: PineValue, right: PineValue) -> Result<PineValue, RuntimeError> {
    match (left, right) {
        (PineValue::String(mut left), PineValue::String(right)) => {
            let result_chars = left.chars().count().saturating_add(right.chars().count());
            if result_chars > MAX_STRING_CHARS {
                return Err(RuntimeError {
                    message: format!(
                        "string concatenation result cannot exceed {MAX_STRING_CHARS} characters"
                    ),
                });
            }
            left.push_str(&right);
            Ok(PineValue::String(left))
        }
        (left, right) => Ok(numeric_add(left, right)),
    }
}

fn numeric_add(left: PineValue, right: PineValue) -> PineValue {
    match (left, right) {
        (PineValue::Int(left), PineValue::Int(right)) => left.checked_add(right).map_or_else(
            || finite_float_or_na(left as f64 + right as f64),
            PineValue::Int,
        ),
        (left, right) => numeric_float_binary(left, right, |left, right| left + right),
    }
}

fn numeric_sub(left: PineValue, right: PineValue) -> PineValue {
    match (left, right) {
        (PineValue::Int(left), PineValue::Int(right)) => left.checked_sub(right).map_or_else(
            || finite_float_or_na(left as f64 - right as f64),
            PineValue::Int,
        ),
        (left, right) => numeric_float_binary(left, right, |left, right| left - right),
    }
}

fn numeric_mul(left: PineValue, right: PineValue) -> PineValue {
    match (left, right) {
        (PineValue::Int(left), PineValue::Int(right)) => left.checked_mul(right).map_or_else(
            || finite_float_or_na(left as f64 * right as f64),
            PineValue::Int,
        ),
        (left, right) => numeric_float_binary(left, right, |left, right| left * right),
    }
}

fn numeric_mod(left: PineValue, right: PineValue) -> PineValue {
    match (left, right) {
        (PineValue::Int(_), PineValue::Int(0)) => PineValue::Na,
        (PineValue::Int(left), PineValue::Int(right)) => left.checked_rem(right).map_or_else(
            || finite_float_or_na(left as f64 % right as f64),
            PineValue::Int,
        ),
        (left, right) => numeric_float_binary(left, right, |left, right| left % right),
    }
}

fn numeric_float_binary(
    left: PineValue,
    right: PineValue,
    op: impl FnOnce(f64, f64) -> f64,
) -> PineValue {
    match (left.as_f64(), right.as_f64()) {
        (Some(left), Some(right)) => finite_float_or_na(op(left, right)),
        _ => PineValue::Na,
    }
}

fn compare_binary(
    left: PineValue,
    right: PineValue,
    op: impl FnOnce(f64, f64) -> bool,
) -> PineValue {
    match (left.as_f64(), right.as_f64()) {
        (Some(left), Some(right)) => PineValue::Bool(op(left, right)),
        _ => PineValue::Na,
    }
}

pub(crate) fn values_equal(left: &PineValue, right: &PineValue) -> bool {
    match (left.as_f64(), right.as_f64()) {
        // Pine Script `==` performs exact numeric equality (no tolerance);
        // the `as_f64` branch keeps cross-type comparisons such as
        // `int == float` working without introducing an arbitrary epsilon.
        #[allow(clippy::float_cmp)]
        (Some(left), Some(right)) => left == right,
        _ => left == right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concatenates_strings_and_propagates_na() {
        assert_eq!(
            eval_binary(
                HirBinaryOp::Add,
                PineValue::String("Pine ".to_owned()),
                PineValue::String("脚本".to_owned()),
            )
            .expect("string concatenation"),
            PineValue::String("Pine 脚本".to_owned())
        );
        assert_eq!(
            eval_binary(
                HirBinaryOp::Add,
                PineValue::Na,
                PineValue::String("suffix".to_owned()),
            )
            .expect("na propagation"),
            PineValue::Na
        );
    }

    #[test]
    fn enforces_string_concatenation_character_limit() {
        let at_limit = "界".repeat(MAX_STRING_CHARS);
        assert_eq!(
            eval_binary(
                HirBinaryOp::Add,
                PineValue::String(at_limit.clone()),
                PineValue::String(String::new()),
            )
            .expect("limit-sized string"),
            PineValue::String(at_limit.clone())
        );

        let error = eval_binary(
            HirBinaryOp::Add,
            PineValue::String(at_limit),
            PineValue::String("x".to_owned()),
        )
        .expect_err("over-limit concatenation");
        assert_eq!(
            error.message,
            "string concatenation result cannot exceed 40960 characters"
        );
    }
}
