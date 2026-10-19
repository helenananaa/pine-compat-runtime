use pine_sema::{AnalysisInput, analyze_input};
use pine_syntax::SourceFile;

#[test]
fn complete_native_function_body_admission_controls() {
    let cases: &[(&str, bool, u16, bool)] = &[
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/unused-map-body-unused-v5-46.pine"
            ),
            false,
            5,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/unused-map-body-called-v5-46.pine"
            ),
            false,
            5,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/unused-map-body-exported-v5-46.pine"
            ),
            false,
            5,
            true,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/unused-map-body-unused-v6-46.pine"
            ),
            false,
            6,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/unused-map-body-called-v6-46.pine"
            ),
            false,
            6,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/unused-map-body-exported-v6-46.pine"
            ),
            false,
            6,
            true,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-contrast-valid-unused-v5-46.pine"
            ),
            true,
            5,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-contrast-partial-unused-v5-46.pine"
            ),
            true,
            5,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-contrast-unknown-unused-v5-46.pine"
            ),
            false,
            5,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-contrast-global-after-unused-v5-46.pine"
            ),
            false,
            5,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-contrast-valid-unused-v6-46.pine"
            ),
            true,
            6,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-contrast-partial-unused-v6-46.pine"
            ),
            true,
            6,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-contrast-unknown-unused-v6-46.pine"
            ),
            false,
            6,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-contrast-global-after-unused-v6-46.pine"
            ),
            false,
            6,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-boundary-partial-wrong-key-v5-46.pine"
            ),
            true,
            5,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-boundary-known-wrong-arity-v5-46.pine"
            ),
            false,
            5,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-boundary-parameter-reassignment-v5-46.pine"
            ),
            false,
            5,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-boundary-untyped-receiver-v5-46.pine"
            ),
            true,
            5,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-boundary-typed-helper-v5-46.pine"
            ),
            true,
            5,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-boundary-method-wrong-value-v5-46.pine"
            ),
            false,
            5,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-boundary-partial-wrong-key-v6-46.pine"
            ),
            true,
            6,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-boundary-known-wrong-arity-v6-46.pine"
            ),
            false,
            6,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-boundary-parameter-reassignment-v6-46.pine"
            ),
            false,
            6,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-boundary-untyped-receiver-v6-46.pine"
            ),
            true,
            6,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-boundary-typed-helper-v6-46.pine"
            ),
            true,
            6,
            false,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-boundary-method-wrong-value-v6-46.pine"
            ),
            false,
            6,
            false,
        ),
    ];
    for &(text, expected, version, library) in cases {
        let name = text.lines().nth(1).unwrap();
        let input = if library {
            AnalysisInput::with_library_sources(
                SourceFile::new("root.pine", format!("//@version={}\nimport local/Unused/1 as c\nindicator(\"unused library\")\nplot(1)\n", version)),
                vec![("local/Unused/1".to_owned(), SourceFile::new("library.pine", text.to_owned()))],
            ).unwrap()
        } else {
            AnalysisInput::new(SourceFile::new(name, text.to_owned()))
        };
        let analysis = analyze_input(&input);
        assert_eq!(
            analysis.hir.is_some(),
            expected,
            "{name}: {:?}",
            analysis.diagnostics
        );
        if !expected {
            assert!(!analysis.diagnostics.is_empty(), "{name}");
        }
        if library {
            assert!(
                analysis.diagnostics.iter().any(|d| d
                    .source
                    .as_ref()
                    .is_some_and(|s| s.library_key.as_deref() == Some("local/Unused/1"))),
                "{name}: {:?}",
                analysis.diagnostics
            );
        }
    }
}

#[test]
fn declaration_check_does_not_specialize_generic_calls_or_escape_bindings() {
    for version in [5, 6] {
        let source = format!(
            "//@version={version}\nindicator(\"generic isolation\")\nf(x)=>x\na=f(1)\nb=f(\"text\")\nplot(a)\nplot(str.length(b))\n"
        );
        let result = analyze_input(&AnalysisInput::new(SourceFile::new("generic.pine", source)));
        assert!(result.hir.is_some(), "{:?}", result.diagnostics);
    }
}

#[test]
fn imported_nested_fields_keep_the_receivers_alias_during_declaration_checks() {
    let library = "//@version=6\nlibrary(\"NestedFields\")\nexport type Point\n    float value\nexport type Wrapper\n    Point nested\nmethod moved(Wrapper self,float amount)=>\n    local=Wrapper.new(Point.new(self.nested.value))\n    local.nested:=Point.new(local.nested.value+amount)\n    local\nexport value(Wrapper self)=>self.nested.value\n";
    let root = "//@version=6\nimport local/Nested/1 as left\nimport local/Nested/1 as right\nindicator(\"independent aliases\")\na=left.Wrapper.new(left.Point.new(1))\nb=right.Wrapper.new(right.Point.new(2))\na.nested:=left.Point.new(3)\nb.nested:=right.Point.new(4)\nplot(left.value(a)+right.value(b))\n";
    let input = AnalysisInput::with_library_sources(
        SourceFile::new("root.pine", root),
        vec![(
            "local/Nested/1".to_owned(),
            SourceFile::new("nested.pine", library),
        )],
    )
    .unwrap();
    let result = analyze_input(&input);
    assert!(result.hir.is_some(), "{:?}", result.diagnostics);
}

#[test]
fn declaration_dependency_walk_bounds_recursive_fanout() {
    let result = analyze_input(&AnalysisInput::new(SourceFile::new(
        "recursive.pine",
        "//@version=6\nindicator(\"recursive fanout\")\nf(x)=>f(x)+f(x)\nplot(1)\n",
    )));
    assert!(result.hir.is_none());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == "E_RECURSIVE_FUNCTION")
    );
}

#[test]
fn generic_forwarding_to_typed_defaults_defers_until_specialization() {
    for version in [5, 6] {
        let body = "g(bool flag,int offset=0)=>flag?offset:0\nf(x)=>g(x)\n";
        let source =
            format!("//@version={version}\nindicator(\"generic defaults\")\n{body}plot(1)\n");
        let result = analyze_input(&AnalysisInput::new(SourceFile::new(
            "defaults.pine",
            source,
        )));
        assert!(
            result.hir.is_some(),
            "{version}/{body}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn generic_values_forward_through_methods_without_weakening_actual_calls() {
    for version in [5, 6] {
        for method_call in ["self.g(x)", "g(self,x)"] {
            for (output, valid) in [
                ("plot(1)", true),
                ("plot(f(Anchor.new(3),true))", true),
                ("plot(f(Anchor.new(3),\"bad\"))", false),
            ] {
                let source = format!(
                    "//@version={version}\nindicator(\"forwarding\")\ntype Anchor\n    int value\nh(bool flag)=>flag\nmethod g(Anchor self,x)=>h(x)?self.value:0\nf(Anchor self,x)=>{method_call}\n{output}\n"
                );
                let result = analyze_input(&AnalysisInput::new(SourceFile::new(
                    "forwarding.pine",
                    source,
                )));
                assert_eq!(
                    result.hir.is_some(),
                    valid,
                    "{version}/{method_call}/{output}: {:?}",
                    result.diagnostics
                );
                if !valid {
                    assert!(
                        result
                            .diagnostics
                            .iter()
                            .any(|d| d.code == "E_FUNCTION_ARG_TYPE")
                    );
                }
            }
        }
    }
}

#[test]
fn generic_method_arguments_do_not_defer_structural_recursion() {
    for version in [5, 6] {
        let source = format!(
            "//@version={version}\nindicator(\"recursive method\")\ntype Anchor\n    int value\nmethod g(Anchor self,x)=>self.g(x)+self.value\nplot(1)\n"
        );
        let result = analyze_input(&AnalysisInput::new(SourceFile::new(
            "recursive.pine",
            source,
        )));
        assert!(result.hir.is_none());
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == "E_RECURSIVE_METHOD"),
            "{:?}",
            result.diagnostics
        );
    }
}

#[test]
fn known_receivers_require_the_method_name_even_with_generic_arguments() {
    for version in [5, 6] {
        let source = format!(
            "//@version={version}\nindicator(\"unknown method\")\ntype Anchor\n    int value\nf(Anchor self,x)=>self.absent(x)+self.value\nplot(1)\n"
        );
        let result = analyze_input(&AnalysisInput::new(SourceFile::new("unknown.pine", source)));
        assert!(result.hir.is_none());
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == "E_UNKNOWN_METHOD"),
            "{:?}",
            result.diagnostics
        );
    }
}

#[test]
fn unchanged_legacy_forward_global_fixtures_match_native_rejection() {
    for source in [
        include_str!("../../../tests/fixtures/sema/supported_map_udf_method_returns.pine"),
        include_str!(
            "../../../tests/fixtures/sema/supported_local_user_method_map_call_result_reads.pine"
        ),
        include_str!(
            "../../../tests/fixtures/sema/supported_user_method_matrix_call_result_reads.pine"
        ),
    ] {
        let result = analyze_input(&AnalysisInput::new(SourceFile::new("legacy.pine", source)));
        assert!(result.hir.is_none());
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == "E_UNKNOWN_SYMBOL")
        );
    }
}

#[test]
fn native_typed_conditional_return_rejections_are_checked_without_execution() {
    for source in [
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-shape-typed-mixed-branches-unused-v5-46.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-shape-typed-mixed-branches-called-v5-46.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-shape-typed-udt-arrays-unused-v5-46.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-shape-typed-mixed-branches-unused-v6-46.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-shape-typed-mixed-branches-called-v6-46.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/function-bodies-46/body-shape-typed-udt-arrays-unused-v6-46.pine"
        ),
    ] {
        let result = analyze_input(&AnalysisInput::new(SourceFile::new(
            "branches.pine",
            source,
        )));
        assert!(result.hir.is_none(), "{source}");
        assert!(result.diagnostics.iter().any(|d| d.code == "E_BRANCH_TYPE"));
    }
}

#[test]
fn call_result_coverage_uses_receiver_inputs_and_lexically_bound_names() {
    for source in [
        include_str!("../../../tests/fixtures/sema/legal-map-call-results-v5-46.pine"),
        include_str!("../../../tests/fixtures/sema/legal-map-call-results-v6-46.pine"),
        include_str!("../../../tests/fixtures/sema/legal-matrix-call-results-v5-46.pine"),
        include_str!("../../../tests/fixtures/sema/legal-matrix-call-results-v6-46.pine"),
    ] {
        let result = analyze_input(&AnalysisInput::new(SourceFile::new(
            "call-results.pine",
            source,
        )));
        assert!(result.hir.is_some(), "{:?}", result.diagnostics);
    }
}

#[test]
fn library_declarations_do_not_capture_the_callers_namespaces() {
    let root = "//@version=6\nimport local/A/1 as lib\nimport local/B/1 as other\nindicator(\"library scope\")\ncaller_only=1\nplot(1)\n";
    for body in ["caller_only+1", "other.Point.new(1)"] {
        let input = AnalysisInput::with_library_sources(
            SourceFile::new("root.pine", root),
            vec![
                (
                    "local/A/1".to_owned(),
                    SourceFile::new(
                        "a.pine",
                        format!("//@version=6\nlibrary(\"A\")\nexport value()=>{body}\n"),
                    ),
                ),
                (
                    "local/B/1".to_owned(),
                    SourceFile::new(
                        "b.pine",
                        "//@version=6\nlibrary(\"B\")\nexport type Point\n    int value\n",
                    ),
                ),
            ],
        )
        .unwrap();
        let result = analyze_input(&input);
        assert!(result.hir.is_none(), "library captured caller name: {body}");
    }
}

#[test]
fn unused_library_bodies_keep_their_own_language_version() {
    let input = AnalysisInput::with_library_sources(
        SourceFile::new(
            "root.pine",
            "//@version=6\nimport local/Legacy/1 as lib\nindicator(\"mixed versions\")\nplot(1)\n",
        ),
        vec![(
            "local/Legacy/1".to_owned(),
            SourceFile::new(
                "legacy.pine",
                "//@version=5\nlibrary(\"Legacy\")\nexport value()=>not 1\n",
            ),
        )],
    )
    .unwrap();
    let result = analyze_input(&input);
    assert!(result.hir.is_some(), "{:?}", result.diagnostics);
}
