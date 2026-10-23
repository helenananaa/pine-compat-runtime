//! Declaration-level qualifier constraints. Typed parameters form a qualifier
//! boundary; only untyped parameters substitute the caller's qualifier flow.
mod calls;

use super::*;

#[derive(Clone, PartialEq, Eq)]
pub(super) struct Flow {
    fixed: Qualifier,
    deps: BTreeSet<usize>,
    kind: ValueKind,
    hint: Option<String>,
}

impl Default for Flow {
    fn default() -> Self {
        Self::typed(PineType::new(Qualifier::Const, ValueKind::Na))
    }
}

fn reference(kind: ValueKind) -> bool {
    is_collection_kind(kind)
        || matches!(
            kind,
            ValueKind::UserType
                | ValueKind::Label
                | ValueKind::Line
                | ValueKind::LineFill
                | ValueKind::Box
                | ValueKind::Polyline
                | ValueKind::Table
                | ValueKind::ChartPoint
        )
}

impl Flow {
    fn typed(ty: PineType) -> Self {
        Self {
            fixed: if reference(ty.kind) {
                Qualifier::Series
            } else {
                ty.qualifier
            },
            deps: BTreeSet::new(),
            kind: ty.kind,
            hint: Some(value_kind_name(ty.kind).to_owned()),
        }
    }
    fn unknown() -> Self {
        Self::typed(UNKNOWN)
    }
    pub(super) fn parameter(index: usize, param: Option<&FunctionParamInfo>) -> Self {
        let mut flow = param.map_or_else(Self::unknown, |p| Self::typed(p.pine_type));
        if let Some(p) = param
            && let Some(name) = &p.user_type_name
        {
            flow.hint = Some(name.clone());
        }
        if param.is_none_or(super::auto_scalar) {
            flow.deps.insert(index);
            flow.fixed = Qualifier::Const;
            if param.is_none() {
                flow.hint = Some(format!("@{index}"));
            }
        }
        flow
    }
    fn merge(mut self, other: Self) -> Self {
        self.fixed = strongest_qualifier(self.fixed, other.fixed);
        self.deps.extend(other.deps);
        if self.kind == ValueKind::Na {
            self.kind = other.kind;
            self.hint = other.hint;
        } else if other.kind != ValueKind::Na && self.kind != other.kind {
            self.kind = common_kind(self.kind, other.kind).unwrap_or(ValueKind::Na);
            self.hint = Some(value_kind_name(self.kind).to_owned());
        } else if self.hint != other.hint && other.kind != ValueKind::Na {
            self.hint = None;
        }
        self
    }
    fn controlled(mut self, gate: Self) -> Self {
        self.fixed = strongest_qualifier(self.fixed, gate.fixed);
        self.deps.extend(gate.deps);
        self
    }
    fn qualifier(&self, params: &[Option<FunctionParamInfo>]) -> Qualifier {
        self.deps.iter().fold(self.fixed, |q, i| {
            strongest_qualifier(
                q,
                params
                    .get(*i)
                    .and_then(Option::as_ref)
                    .map_or(Qualifier::Series, |p| p.pine_type.qualifier),
            )
        })
    }
    fn pine_type(&self, params: &[Option<FunctionParamInfo>]) -> PineType {
        PineType::new(self.qualifier(params), self.kind)
    }
}

pub(super) struct Walker<'a> {
    analyzer: &'a Analyzer,
    units: &'a HashMap<String, Unit>,
    functions: &'a HashMap<String, Vec<String>>,
    methods: &'a HashMap<(String, String), String>,
    summaries: &'a HashMap<String, Summary>,
    params: Vec<Option<FunctionParamInfo>>,
    pub(super) env: HashMap<String, Flow>,
    pub(super) simple: BTreeSet<usize>,
    depth: u32,
    nodes: usize,
    guards: Vec<Flow>,
}

impl<'a> Walker<'a> {
    pub(super) fn new(
        analyzer: &'a Analyzer,
        units: &'a HashMap<String, Unit>,
        functions: &'a HashMap<String, Vec<String>>,
        methods: &'a HashMap<(String, String), String>,
        summaries: &'a HashMap<String, Summary>,
        params: Vec<Option<FunctionParamInfo>>,
    ) -> Self {
        Self {
            analyzer,
            units,
            functions,
            methods,
            summaries,
            params,
            env: HashMap::new(),
            simple: BTreeSet::new(),
            depth: 0,
            nodes: 0,
            guards: Vec::new(),
        }
    }
    pub(super) fn body(&mut self, body: &FunctionBody) -> Flow {
        match body {
            FunctionBody::Expr(expr) => self.expr(expr),
            FunctionBody::Block(stmts) => self.statements(stmts),
        }
    }
    pub(super) fn statements(&mut self, stmts: &[Stmt]) -> Flow {
        let mut result = Flow::default();
        for stmt in stmts {
            result = self.statement(stmt);
        }
        result
    }
    fn lookup(&self, name: &str) -> Flow {
        self.env
            .get(name)
            .cloned()
            .or_else(|| {
                self.analyzer
                    .scope
                    .resolve(name)
                    .map(|s| Flow::typed(s.pine_type))
            })
            .unwrap_or_else(Flow::unknown)
    }
    fn statement(&mut self, stmt: &Stmt) -> Flow {
        if self.nodes >= 65536 || self.depth >= MAX_SEMA_EXPR_DEPTH {
            return Flow::unknown();
        }
        self.nodes += 1;
        self.depth += 1;
        let result = match &stmt.kind {
            StmtKind::Expr(expr) => self.expr(expr),
            StmtKind::Decl {
                name,
                value,
                declared_type,
                ..
            } => {
                let mut flow = self.expr(value);
                if let Some(ty) = declared_type {
                    flow.hint = Some(ty.canonical_name());
                }
                self.env.insert(name.clone(), flow.clone());
                flow
            }
            StmtKind::Reassign { name, value } => {
                let flow = self.expr(value);
                let flow = self.guards.iter().cloned().fold(flow, Flow::controlled);
                self.env.insert(name.clone(), flow.clone());
                flow
            }
            StmtKind::TupleDecl { names, value } => {
                let flow = self.expr(value);
                for name in names {
                    self.env.insert(name.clone(), flow.clone());
                }
                flow
            }
            StmtKind::FieldReassign { value, .. } => self.expr(value),
            StmtKind::ArrayFieldReassign {
                array,
                index,
                value,
                ..
            } => {
                self.expr(array);
                self.expr(index);
                self.expr(value)
            }
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.branches(condition, then_branch, else_branch),
            StmtKind::For {
                counter,
                from,
                to,
                step,
                body,
            } => self.for_loop(counter, from, to, step.as_ref(), body),
            StmtKind::ForIn {
                index,
                value,
                iterable,
                body,
            } => self.for_in(index.as_deref(), value, iterable, body),
            StmtKind::While { condition, body } => {
                let gate = self.expr(condition);
                self.loop_body(body, gate)
            }
            _ => Flow::default(),
        };
        self.depth -= 1;
        result
    }
    fn controlled_env(
        &mut self,
        before: HashMap<String, Flow>,
        branches: &[HashMap<String, Flow>],
        gate: &Flow,
    ) {
        self.env = before;
        for (name, old) in &mut self.env {
            let mut changed = false;
            for branch in branches {
                if let Some(value) = branch.get(name)
                    && value != old
                {
                    changed = true;
                    *old = old.clone().merge(value.clone());
                }
            }
            if changed {
                *old = old.clone().controlled(gate.clone());
            }
        }
    }
    fn branches(&mut self, condition: &Expr, then_branch: &[Stmt], else_branch: &[Stmt]) -> Flow {
        let gate = self.expr(condition);
        let before = self.env.clone();
        self.guards.push(gate.clone());
        let left = self.statements(then_branch);
        let left_env = self.env.clone();
        self.env = before.clone();
        let right = self.statements(else_branch);
        let right_env = self.env.clone();
        self.guards.pop();
        self.controlled_env(before, &[left_env, right_env], &gate);
        left.merge(right).controlled(gate)
    }
    fn loop_body(&mut self, body: &[Stmt], gate: Flow) -> Flow {
        let before = self.env.clone();
        self.guards.push(gate.clone());
        let result = self.statements(body);
        self.guards.pop();
        let after = self.env.clone();
        self.controlled_env(before, &[after], &gate);
        result.controlled(gate)
    }
    fn for_loop(
        &mut self,
        counter: &str,
        from: &Expr,
        to: &Expr,
        step: Option<&Expr>,
        body: &[Stmt],
    ) -> Flow {
        let mut flow = self.expr(from).merge(self.expr(to));
        if let Some(step) = step {
            flow = flow.merge(self.expr(step));
        }
        let old = self.env.insert(counter.to_owned(), flow.clone());
        let result = self.loop_body(body, flow);
        if let Some(old) = old {
            self.env.insert(counter.to_owned(), old);
        } else {
            self.env.remove(counter);
        }
        result
    }
    fn for_in(&mut self, index: Option<&str>, value: &str, iterable: &Expr, body: &[Stmt]) -> Flow {
        let flow = self.expr(iterable);
        let before = self.env.clone();
        if let Some(index) = index {
            self.env.insert(
                index.to_owned(),
                Flow::typed(PineType::new(Qualifier::Series, ValueKind::Int)),
            );
        }
        self.env.insert(value.to_owned(), Flow::unknown());
        let result = self.statements(body);
        let after = self.env.clone();
        self.controlled_env(before, &[after], &flow);
        result.controlled(flow)
    }
    fn expr(&mut self, expr: &Expr) -> Flow {
        if self.nodes >= 65536 || self.depth >= MAX_SEMA_EXPR_DEPTH {
            return Flow::unknown();
        }
        self.depth += 1;
        self.nodes += 1;
        let result = match &expr.kind {
            ExprKind::Literal(literal) => Flow::typed(literal_type(literal)),
            ExprKind::Identifier(name) => self.lookup(name),
            ExprKind::QualifiedName(parts) => {
                self.env.get(&parts.join(".")).cloned().unwrap_or_else(|| {
                    if parts.len() > 1 && self.env.contains_key(&parts[0]) {
                        let mut flow = self.lookup(&parts[0]);
                        flow.fixed = Qualifier::Series;
                        flow.deps.clear();
                        flow.kind = ValueKind::Na;
                        flow.hint = None;
                        flow
                    } else {
                        self.lookup(&parts.join("."))
                    }
                })
            }
            ExprKind::Member { receiver, .. } => {
                self.expr(receiver);
                Flow::unknown()
            }
            ExprKind::Group(inner) => self.expr(inner),
            ExprKind::Unary { op, expr } => {
                let mut flow = self.expr(expr);
                if *op == UnaryOp::Not {
                    flow.kind = ValueKind::Bool;
                    flow.hint = Some("bool".into());
                }
                flow
            }
            ExprKind::Binary { op, left, right } => {
                let mut flow = self.expr(left).merge(self.expr(right));
                if matches!(
                    op,
                    BinaryOp::Eq
                        | BinaryOp::NotEq
                        | BinaryOp::Gt
                        | BinaryOp::Gte
                        | BinaryOp::Lt
                        | BinaryOp::Lte
                        | BinaryOp::And
                        | BinaryOp::Or
                ) {
                    flow.kind = ValueKind::Bool;
                    flow.hint = Some("bool".into());
                }
                flow
            }
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                let condition = self.expr(condition);
                let mut flow = self.expr(then_expr).merge(self.expr(else_expr));
                flow.fixed = strongest_qualifier(flow.fixed, condition.fixed);
                flow.deps.extend(condition.deps);
                flow
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.branches(condition, then_branch, else_branch),
            ExprKind::For {
                counter,
                from,
                to,
                step,
                body,
            } => self.for_loop(counter, from, to, step.as_deref(), body),
            ExprKind::ForIn {
                index,
                value,
                iterable,
                body,
            } => self.for_in(index.as_deref(), value, iterable, body),
            ExprKind::While { condition, body } => {
                let flow = self.expr(condition);
                self.loop_body(body, flow)
            }
            ExprKind::Switch { selector, arms } => {
                let before = self.env.clone();
                let mut flow = selector
                    .as_ref()
                    .map_or_else(Flow::default, |e| self.expr(e));
                let mut envs = Vec::new();
                for arm in arms {
                    self.env = before.clone();
                    if let Some(condition) = &arm.condition {
                        flow = flow.merge(self.expr(condition));
                    }
                    flow = flow.merge(match &arm.result {
                        SwitchArmResult::Expr(e) => self.expr(e),
                        SwitchArmResult::Block(stmts) => self.statements(stmts),
                    });
                    envs.push(self.env.clone());
                }
                self.controlled_env(before, &envs, &flow);
                flow
            }
            ExprKind::Tuple(values) => values
                .iter()
                .fold(Flow::default(), |flow, e| flow.merge(self.expr(e))),
            ExprKind::History { expr, offset } => {
                let mut flow = self.expr(expr);
                self.expr(offset);
                flow.fixed = Qualifier::Series;
                flow.deps.clear();
                flow
            }
            ExprKind::Call { callee, args } => self.call(callee, args),
        };
        self.depth -= 1;
        result
    }
    fn require_simple(&mut self, flow: &Flow) {
        for index in &flow.deps {
            if self
                .params
                .get(*index)
                .is_some_and(|p| p.as_ref().is_none_or(super::auto_scalar))
            {
                self.simple.insert(*index);
            }
        }
    }
}
