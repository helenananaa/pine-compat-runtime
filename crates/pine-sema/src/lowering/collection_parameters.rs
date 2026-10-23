use super::*;

impl Analyzer {
    pub(super) fn lower_collection_parameter(
        &mut self,
        binding: Option<&FunctionParamInfo>,
        value: HirExpr,
        span: Span,
    ) -> Option<HirExpr> {
        let Some(binding) = binding else {
            return Some(value);
        };
        let callee = if let Some(map) = binding.map_type {
            format!(
                "$parameter.map.{}.{}",
                value_kind_name(map.key_kind),
                value_kind_name(map.value_kind)
            )
        } else if binding.pine_type.kind == ValueKind::FloatMatrix {
            "$parameter.matrix.float".to_owned()
        } else {
            return Some(value);
        };
        if !self.record_lowering_node(span) {
            return None;
        }
        Some(HirExpr {
            pine_type: binding.pine_type,
            series_id: self.series_id_for_type(binding.pine_type),
            kind: HirExprKind::Call {
                callee,
                call_site_id: self.alloc_call_site(),
                args: vec![HirCallArg { name: None, value }],
            },
        })
    }
}
