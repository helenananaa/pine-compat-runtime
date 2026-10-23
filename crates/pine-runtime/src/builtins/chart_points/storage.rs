//! chart.point identities share the runtime's checkpointed object arena.
//! Owned ChartPointValue remains the host-neutral snapshot representation.
use crate::{ChartPointValue, HistoricalRuntime, PineValue, RuntimeError};

pub(crate) fn require_defined_point(value: &PineValue) -> Result<(), RuntimeError> {
    if matches!(value, PineValue::Na) {
        Err(RuntimeError {
            message:
                "E_CHART_POINT_NA_FIELD: cannot access a field of an undefined (na) chart.point"
                    .to_owned(),
        })
    } else {
        Ok(())
    }
}

impl HistoricalRuntime<'_> {
    pub(crate) fn allocate_chart_point(
        &mut self,
        point: ChartPointValue,
    ) -> Result<PineValue, RuntimeError> {
        let id = u32::try_from(self.next_object_id).map_err(|_| RuntimeError {
            message: "chart.point identity space exhausted".to_owned(),
        })?;
        let fields = vec![*point.time, *point.index, *point.price];
        if !self.record_collection_values(&fields) {
            return Err(self.resource_budget.collection_error());
        }
        self.next_object_id += 1;
        self.object_store.insert(id, fields);
        self.object_varip_fields.insert(id, vec![false; 3]);
        Ok(PineValue::ChartPointRef(id))
    }

    pub(crate) fn materialize_chart_point(
        &self,
        value: PineValue,
    ) -> Result<PineValue, RuntimeError> {
        match value {
            PineValue::ChartPointRef(id) => Ok(PineValue::ChartPoint(ChartPointValue::new(
                self.object_field(id, 0)?,
                self.object_field(id, 1)?,
                self.object_field(id, 2)?,
            ))),
            value => Ok(value),
        }
    }

    pub(crate) fn set_chart_point_field(
        &mut self,
        id: u32,
        index: usize,
        value: PineValue,
    ) -> Result<(), RuntimeError> {
        self.set_object_field(id, index, super::normalize_chart_point_field(index, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_roots_survive_collection_and_checkpoints_keep_independent_fields() {
        let report = pine_sema::analyze_source(&pine_syntax::SourceFile::new(
            "point-gc.pine",
            "//@version=6\nindicator(\"point GC\")\np=chart.point.from_index(1,10)\nplot(p.price)\n",
        ));
        let hir = report.hir.unwrap();
        let symbol = hir.symbols.iter().find(|s| s.name == "p").unwrap().id;
        let mut runtime = HistoricalRuntime::new(&hir);
        let root = runtime
            .allocate_chart_point(ChartPointValue::new(
                PineValue::Na,
                PineValue::Int(1),
                PineValue::Float(10.),
            ))
            .unwrap();
        runtime.current_symbols.insert(symbol, root.clone());
        for index in 0..30000 {
            runtime
                .allocate_chart_point(ChartPointValue::new(
                    PineValue::Na,
                    PineValue::Int(index),
                    PineValue::Float(0.),
                ))
                .unwrap();
        }
        runtime.collect_temporary_collections();
        assert_eq!(runtime.object_store.len(), 1);
        assert_eq!(runtime.object_varip_fields.len(), 1);
        let checkpoint = runtime.clone();
        let PineValue::ChartPointRef(id) = root else {
            panic!("point identity")
        };
        runtime
            .set_chart_point_field(id, 2, PineValue::Int(20))
            .unwrap();
        assert_eq!(
            checkpoint.object_field(id, 2).unwrap(),
            PineValue::Float(10.)
        );
        assert_eq!(runtime.object_field(id, 2).unwrap(), PineValue::Float(20.));
        assert_eq!(runtime.next_object_id, 30001);
    }
}
