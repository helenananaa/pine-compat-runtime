use crate::analyzer::maps::map_kind_from_template_name;
use crate::prelude::*;

/// Reference qualifiers do not turn a collection ID into a const/simple value.
/// Templates still constrain the element domains, including int-to-float widening.
pub(crate) fn collection_parameter_info(name: &str) -> Option<(PineType, Option<MapTypeInfo>)> {
    if let Some(element) = name
        .strip_prefix("array<")
        .and_then(|n| n.strip_suffix('>'))
        && let Some(kind) = array_kind_from_element_type_name(element)
    {
        return Some((PineType::new(Qualifier::Series, kind), None));
    }
    let kind = match name
        .strip_prefix("matrix<")
        .and_then(|n| n.strip_suffix('>'))
    {
        Some("float") => ValueKind::FloatMatrix,
        Some("int") => ValueKind::IntMatrix,
        Some("bool") => ValueKind::BoolMatrix,
        Some("string") => ValueKind::StringMatrix,
        Some("color") => ValueKind::ColorMatrix,
        _ => {
            let (key, value) = name
                .strip_prefix("map<")?
                .strip_suffix('>')?
                .split_once(',')?;
            return Some((
                PineType::new(Qualifier::Series, ValueKind::Map),
                Some(MapTypeInfo {
                    key_kind: map_kind_from_template_name(key)?,
                    value_kind: map_kind_from_template_name(value)?,
                }),
            ));
        }
    };
    Some((PineType::new(Qualifier::Series, kind), None))
}

pub(crate) fn map_parameter_accepts(expected: MapTypeInfo, actual: MapTypeInfo) -> bool {
    let accepts = |expected, actual| {
        expected == actual || (expected == ValueKind::Float && actual == ValueKind::Int)
    };
    accepts(expected.key_kind, actual.key_kind) && accepts(expected.value_kind, actual.value_kind)
}

impl FunctionParamInfo {
    pub(crate) fn accepts_argument(&self, actual: PineType, map: Option<MapTypeInfo>) -> bool {
        let matrix_widening =
            self.pine_type.kind == ValueKind::FloatMatrix && actual.kind == ValueKind::IntMatrix;
        (can_assign(self.pine_type, actual) || matrix_widening)
            && self.map_type.is_none_or(|expected| {
                actual.kind == ValueKind::Na
                    || map.is_some_and(|actual| map_parameter_accepts(expected, actual))
            })
    }
}
