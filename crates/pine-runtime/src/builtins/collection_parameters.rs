//! Integer collections passed to float templates retain their shared ID.
//! Promotion is monotonic and participates in ordinary checkpoint COW/rollback.
use pine_ir::HirCallArg;

use super::arrays::{ArrayElementKind, array_value_for_kind};
use super::matrices::MatrixElementKind;
use crate::runtime::collection_gc::collection_values_allocation_bytes;
use crate::{HistoricalRuntime, PineValue, RuntimeError};

impl HistoricalRuntime<'_> {
    pub(crate) fn eval_collection_parameter(
        &mut self,
        callee: &str,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let value = self.eval_expr(&args[0].value)?;
        match value {
            PineValue::Map(id) => {
                let Some(storage) = self.map_store.get(&id) else {
                    return Ok(value);
                };
                let template = callee.strip_prefix("$parameter.map.").unwrap_or("");
                let (key, element) = template.split_once('.').unwrap_or(("", ""));
                let key_kind = if key == "float" && storage.key_kind == ArrayElementKind::Int {
                    ArrayElementKind::Float
                } else {
                    storage.key_kind
                };
                let value_kind =
                    if element == "float" && storage.value_kind == ArrayElementKind::Int {
                        ArrayElementKind::Float
                    } else {
                        storage.value_kind
                    };
                if key_kind == storage.key_kind && value_kind == storage.value_kind {
                    return Ok(value);
                }
                let bytes = storage.entries.clone_allocation_bytes().saturating_add(
                    collection_values_allocation_bytes(
                        storage.entries.iter().flat_map(|(k, v)| [k, v]),
                    ),
                );
                if !self.record_collection_bytes(bytes) {
                    return Err(self.resource_budget.collection_error());
                }
                let entries: Vec<_> = self
                    .map_store
                    .get(&id)
                    .expect("existing parameter map")
                    .entries
                    .iter()
                    .map(|(k, v)| {
                        (
                            array_value_for_kind(key_kind, k.clone()),
                            array_value_for_kind(value_kind, v.clone()),
                        )
                    })
                    .collect();
                let storage = self.map_store.get_mut(&id).expect("existing parameter map");
                storage.key_kind = key_kind;
                storage.value_kind = value_kind;
                storage.entries = entries.into();
            }
            PineValue::Matrix(id) if callee == "$parameter.matrix.float" => {
                let Some(storage) = self.matrix_store.get(&id) else {
                    return Ok(value);
                };
                if storage.kind != MatrixElementKind::Int {
                    return Ok(value);
                }
                let bytes = collection_values_allocation_bytes(storage.values.iter());
                if !self.record_collection_bytes(bytes) {
                    return Err(self.resource_budget.collection_error());
                }
                let values: Vec<_> = self
                    .matrix_store
                    .get(&id)
                    .expect("existing parameter matrix")
                    .values
                    .iter()
                    .cloned()
                    .map(|v| array_value_for_kind(ArrayElementKind::Float, v))
                    .collect();
                let storage = self
                    .matrix_store
                    .get_mut(&id)
                    .expect("existing parameter matrix");
                storage.kind = MatrixElementKind::Float;
                storage.values = values.into();
            }
            _ => {}
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bar, ResourceLimits};

    #[test]
    fn rejected_parameter_promotion_leaves_shared_storage_unchanged() {
        for (init, body, read) in [
            (
                "var v=map.new<int,int>()\nif barstate.isfirst\n    v.put(1,2)",
                "f(map<float,float> v)=>v.get(1)",
                "v.get(1)",
            ),
            (
                "var v=matrix.new<int>(1,1,2)",
                "f(matrix<float> v)=>v.get(0,0)",
                "v.get(0,0)",
            ),
        ] {
            let source = pine_syntax::SourceFile::new(
                "budget.pine",
                format!(
                    "//@version=6\nindicator(\"parameter budget\")\n{init}\n{body}\nif bar_index>0\n    f(v)\nplot({read})\n"
                ),
            );
            let analysis = pine_sema::analyze_source(&source);
            assert!(
                analysis.diagnostics.is_empty(),
                "{:?}",
                analysis.diagnostics
            );
            let program = analysis.hir.unwrap();
            let mut runtime = HistoricalRuntime::new(&program);
            let bar = |time| Bar {
                time,
                open: 10.,
                high: 10.,
                low: 10.,
                close: 10.,
                volume: 1.,
            };
            runtime.append_bar(bar(0)).unwrap();
            let map = runtime.map_store.get(&0).cloned();
            let matrix = runtime.matrix_store.get(&0).cloned();
            let mut runtime = runtime.with_resource_limits(ResourceLimits {
                max_collection_bytes_per_bar: Some(0),
                ..Default::default()
            });
            let error = runtime.append_bar(bar(60000)).unwrap_err();
            assert!(error.message.contains("collection"), "{error:?}");
            assert_eq!(runtime.map_store.get(&0), map.as_ref());
            assert_eq!(runtime.matrix_store.get(&0), matrix.as_ref());
        }
    }
}
