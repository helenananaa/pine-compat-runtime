use crate::prelude::*;

impl Analyzer {
    pub(crate) fn type_of_expr_with_params(
        &self,
        expr: &Expr,
        param_types: &HashMap<String, PineType>,
    ) -> Option<PineType> {
        let cached = self.expr_types.get(&self.expr_key(expr.span)).copied();
        // Scalar expressions must follow this call's arguments. Reference
        // identities and legacy call translations still require analyzed types.
        if param_types.is_empty()
            || cached.is_some_and(|ty| !crate::analyzer::requests::is_request_scalar_type(ty))
        {
            return cached.or_else(|| self.type_of_expr_in_param_context(expr, param_types));
        }
        self.type_of_expr_in_param_context(expr, param_types)
            .or(cached)
    }
}

pub(super) fn map_new_template_types(name: &str) -> Option<(&str, &str)> {
    let inner = name.strip_prefix("map.new<")?.strip_suffix('>')?;
    inner.split_once(',')
}

pub(super) fn selected_branch_type(
    condition_qualifier: Qualifier,
    then_type: PineType,
    else_type: PineType,
    condition_value: bool,
) -> Option<PineType> {
    let selected_type = if condition_value {
        then_type
    } else {
        else_type
    };
    Some(PineType::new(
        strongest_qualifier(condition_qualifier, selected_type.qualifier),
        common_kind(then_type.kind, else_type.kind)?,
    ))
}

pub(super) fn merge_tuple_element_types(
    current: Option<Vec<PineType>>,
    next: Vec<PineType>,
) -> Option<Vec<PineType>> {
    let Some(current) = current else {
        return Some(next);
    };
    if current.len() != next.len() {
        return None;
    }
    current
        .into_iter()
        .zip(next)
        .map(|(current, next)| merge_result_types(Some(current), next))
        .collect()
}

pub(super) fn selected_tuple_branch_types(
    condition_qualifier: Qualifier,
    then_types: Vec<PineType>,
    else_types: Vec<PineType>,
    condition_value: bool,
) -> Option<Vec<PineType>> {
    if then_types.len() != else_types.len() {
        return None;
    }

    let selected_types = if condition_value {
        then_types.iter()
    } else {
        else_types.iter()
    };

    selected_types
        .zip(then_types.iter().zip(else_types.iter()))
        .map(|(selected_type, (then_type, else_type))| {
            Some(PineType::new(
                strongest_qualifier(condition_qualifier, selected_type.qualifier),
                common_kind(then_type.kind, else_type.kind)?,
            ))
        })
        .collect()
}

pub(super) fn promote_tuple_element_qualifiers(
    types: Vec<PineType>,
    qualifier: Qualifier,
) -> Vec<PineType> {
    types
        .into_iter()
        .map(|pine_type| {
            PineType::new(
                strongest_qualifier(qualifier, pine_type.qualifier),
                pine_type.kind,
            )
        })
        .collect()
}
