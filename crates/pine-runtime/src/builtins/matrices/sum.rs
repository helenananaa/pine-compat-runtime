use super::*;

impl HistoricalRuntime<'_> {
    pub(crate) fn eval_matrix_sum(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let left = self.eval_expr(&args[0].value)?;
        let right = self.eval_expr(&args[1].value)?;
        let PineValue::Matrix(id) = left else {
            return Ok(PineValue::Na);
        };
        let Some(source) = self.matrix_store.get(&id) else {
            return Ok(PineValue::Na);
        };
        let (rows, columns, mut kind, len) = (
            source.rows,
            source.columns,
            source.kind,
            source.values.len(),
        );
        if let PineValue::Matrix(other) = right {
            let Some(other) = self.matrix_store.get(&other) else {
                return Ok(PineValue::Na);
            };
            if (rows, columns) != (other.rows, other.columns) {
                return Err(RuntimeError {
                    message: "matrix sum requires matching row and column counts".to_owned(),
                });
            }
            if other.kind == MatrixElementKind::Float {
                kind = MatrixElementKind::Float;
            }
        } else if matches!(right, PineValue::Float(_))
            || args[1].value.pine_type.kind == pine_ir::ValueKind::Float
        {
            // A typed float can evaluate to Na without losing the result's
            // declared element type. Later writes must still accept floats.
            kind = MatrixElementKind::Float;
        }
        // Charge before constructing the output; neither input is modified.
        if !self.record_collection_allocation(len) {
            return Err(self.resource_budget.collection_error());
        }
        let source = self.matrix_store.get(&id).expect("validated matrix");
        let other = if let PineValue::Matrix(other) = right {
            self.matrix_store.get(&other)
        } else {
            None
        };
        let values = source
            .values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let rhs = other.map_or(&right, |matrix| &matrix.values[index]);
                if kind == MatrixElementKind::Int {
                    match (value, rhs) {
                        (PineValue::Int(left), PineValue::Int(right)) => left
                            .checked_add(*right)
                            .map_or(PineValue::Na, PineValue::Int),
                        _ => PineValue::Na,
                    }
                } else {
                    match (value.as_f64(), rhs.as_f64()) {
                        (Some(left), Some(right)) => finite_float_or_na(left + right),
                        _ => PineValue::Na,
                    }
                }
            })
            .collect::<Vec<_>>();
        Ok(self.insert_matrix_payload(kind, rows, columns, values.into()))
    }
}
