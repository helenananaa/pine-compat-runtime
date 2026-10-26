use super::*;
mod provenance;

#[derive(Clone, Default)]
pub(super) struct RequestScope {
    names: HashSet<String>,
    pub(super) types: HashMap<String, PineType>,
    pub(super) references: bool,
}
impl RequestScope {
    pub(super) fn references() -> Self {
        Self::with_references(true)
    }
    pub(super) fn with_references(references: bool) -> Self {
        Self {
            references,
            ..Self::default()
        }
    }
    pub(super) fn contains(&self, name: &str) -> bool {
        self.names.contains(name)
    }
    pub(super) fn insert(&mut self, name: String) {
        self.names.insert(name);
    }
    pub(super) fn insert_typed(&mut self, name: String, ty: Option<PineType>) {
        if let Some(ty) = ty {
            self.types.insert(name.clone(), ty);
        }
        self.names.insert(name);
    }
    pub(super) fn extend(&mut self, names: impl IntoIterator<Item = String>) {
        self.names.extend(names);
    }
}

pub(super) fn reference_kind(kind: ValueKind) -> bool {
    kind.array_element_kind().is_some()
        || matches!(
            kind,
            ValueKind::ChartPoint
                | ValueKind::UserType
                | ValueKind::Map
                | ValueKind::FloatMatrix
                | ValueKind::IntMatrix
                | ValueKind::BoolMatrix
                | ValueKind::StringMatrix
                | ValueKind::ColorMatrix
                | ValueKind::UserTypeMatrix
        )
}

impl Analyzer {
    pub(super) fn reference_request_call_supported(
        &self,
        callee: &Expr,
        call: &Expr,
        args: &[CallArg],
        locals: &RequestScope,
        walk: &mut RequestWalk,
    ) -> bool {
        let Some(name) = expr_name(callee) else {
            return false;
        };
        let constructor = matches!(
            name.as_str(),
            "chart.point.new"
                | "chart.point.now"
                | "chart.point.from_index"
                | "chart.point.from_time"
                | "chart.point.copy"
        ) || self.direct_user_type_constructor_name(call).is_some()
            || ["array.new<", "map.new<", "matrix.new<"]
                .iter()
                .any(|prefix| name.starts_with(prefix) && name.ends_with('>'));
        let namespace = name.split('.').next().unwrap_or("");
        let collection = matches!(namespace, "array" | "map" | "matrix")
            && pine_builtins::get_phase_1_builtin(&name).is_some();
        if constructor || collection {
            return args
                .iter()
                .all(|arg| self.modern_request_expr(&arg.value, locals, walk));
        }
        let ExprKind::QualifiedName(parts) = &callee.kind else {
            return false;
        };
        if parts.len() != 2 {
            return false;
        }
        let receiver = Expr {
            kind: ExprKind::Identifier(parts[0].clone()),
            span: Span::new(callee.span.start, callee.span.start + parts[0].len()),
        };
        let Some(ty) = self.type_of_expr_with_params(&receiver, &locals.types) else {
            return false;
        };
        let method = &parts[1];
        if ty.kind == ValueKind::ChartPoint && method == "copy" {
            return self.modern_request_expr(&receiver, locals, walk) && args.is_empty();
        }
        let canonical = if ty.kind.array_element_kind().is_some() {
            crate::prelude::array_method_builtin_name(method)
        } else if ty.kind == ValueKind::Map {
            crate::prelude::map_method_builtin_name(method)
        } else {
            crate::types::matrix_method_builtin_name(ty.kind, method)
        };
        canonical.is_some()
            && self.modern_request_expr(&receiver, locals, walk)
            && args
                .iter()
                .all(|arg| self.modern_request_expr(&arg.value, locals, walk))
    }
}
