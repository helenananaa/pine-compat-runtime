use super::*;
use pine_ir::SeriesId;

fn expr(kind: HirExprKind) -> HirExpr {
    HirExpr {
        kind,
        pine_type: PineType::new(Qualifier::Const, ValueKind::Bool),
        series_id: None,
    }
}

fn symbol(id: u32) -> HirExpr {
    expr(HirExprKind::Symbol(SymbolId(id)))
}

#[test]
fn return_dependencies_follow_long_flat_bindings_and_terminate_cycles() {
    let statements = (1..=12_000)
        .map(|id| HirStmt {
            kind: HirStmtKind::Decl {
                symbol: SymbolId(id),
                value: symbol(id - 1),
            },
        })
        .collect();
    let mut body = expr(HirExprKind::Block {
        statements,
        result: Box::new(symbol(12_000)),
    });
    let params = HashMap::from([("condition".to_owned(), symbol(0))]);
    assert!(result_depends_on_parameters(&body, &params));
    let HirExprKind::Block { statements, .. } = &mut body.kind else {
        unreachable!()
    };
    statements[0] = HirStmt {
        kind: HirStmtKind::Decl {
            symbol: SymbolId(1),
            value: symbol(12_000),
        },
    };
    assert!(!result_depends_on_parameters(&body, &params));
}

#[test]
fn unused_parameter_effects_do_not_change_return_materialization() {
    let body = expr(HirExprKind::Block {
        statements: vec![HirStmt {
            kind: HirStmtKind::Expr(symbol(0)),
        }],
        result: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(false)))),
    });
    let params = HashMap::from([("unused".to_owned(), symbol(0))]);
    assert!(!result_depends_on_parameters(&body, &params));
    let nested = boolean_unary(HirUnaryOp::FunctionBoolReturn { materialize: false }, body);
    assert!(result_depends_on_parameters(&nested, &HashMap::new()));
}

#[test]
fn removing_return_provenance_restores_block_shape_and_series_identity() {
    let declaration = HirStmt {
        kind: HirStmtKind::Decl {
            symbol: SymbolId(0),
            value: expr(HirExprKind::Literal(HirLiteral::Bool(true))),
        },
    };
    let inner = expr(HirExprKind::Block {
        statements: vec![declaration.clone()],
        result: Box::new(symbol(0)),
    });
    let mut boundary = boolean_unary(HirUnaryOp::FunctionBoolReturn { materialize: true }, inner);
    boundary.series_id = Some(SeriesId(17));
    let mut outer = expr(HirExprKind::Block {
        statements: vec![],
        result: Box::new(boundary),
    });
    outer.series_id = Some(SeriesId(17));
    let mut statements = vec![HirStmt {
        kind: HirStmtKind::Expr(outer),
    }];
    strip_boolean_return_boundaries(&mut statements);
    let mut expected = expr(HirExprKind::Block {
        statements: vec![declaration],
        result: Box::new(symbol(0)),
    });
    expected.series_id = Some(SeriesId(17));
    assert_eq!(
        statements,
        vec![HirStmt {
            kind: HirStmtKind::Expr(expected),
        }]
    );
}
