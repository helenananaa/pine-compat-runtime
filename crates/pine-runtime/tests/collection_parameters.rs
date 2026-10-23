use pine_runtime::{
    Bar, BarUpdate, HistoricalRuntime, PineValue, PreparedProgram, RealtimeRuntime,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;
use std::sync::Arc;

fn bar(i: i64) -> Bar {
    Bar {
        time: i * 60_000,
        open: 10.,
        high: 10.,
        low: 10.,
        close: 10.,
        volume: 1.,
    }
}

#[test]
fn native_collection_parameters_preserve_templates_and_shared_state() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest: serde_json::Value =
        serde_json::from_str(
            &std::fs::read_to_string(root.join(
                "docs/evidence/strategy-library-20261010/collection-parameter-oracles-45.json",
            ))
            .unwrap(),
        )
        .unwrap();
    let bars = [bar(0), bar(1), bar(2)];
    for case in manifest["records"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let source = std::fs::read_to_string(root.join(case["source"].as_str().unwrap())).unwrap();
        let analysis = analyze_source(&SourceFile::new(name, source));
        if case["native_status"] == "analysis_rejected" {
            assert!(analysis.hir.is_none(), "{name}");
            assert!(
                analysis
                    .diagnostics
                    .iter()
                    .any(|d| d.code == "E_FUNCTION_ARG_TYPE"),
                "{name}: {:?}",
                analysis.diagnostics
            );
            continue;
        }
        assert!(
            analysis.diagnostics.is_empty(),
            "{name}: {:?}",
            analysis.diagnostics
        );
        let program = analysis.hir.unwrap();
        let prepared = PreparedProgram::from_shared_hir(Arc::new(program.clone()));
        let expected: Vec<_> = case["local_three_bar_values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64())
            .collect();
        for mode in 0..3 {
            let mut runtime = if mode == 0 {
                HistoricalRuntime::new(&program)
            } else {
                HistoricalRuntime::from_prepared(&prepared)
            };
            if mode < 2 {
                runtime
                    .append_bars(&bars)
                    .unwrap_or_else(|e| panic!("{name}/{mode}: {e:?}"));
            } else {
                for bar in bars {
                    runtime
                        .append_bar(bar)
                        .unwrap_or_else(|e| panic!("{name}/{mode}: {e:?}"));
                }
            }
            let values: Vec<_> = runtime
                .result()
                .plots
                .iter()
                .map(|p| p.values.last().and_then(PineValue::as_f64))
                .collect();
            assert_eq!(values, expected, "{name}/{mode}");
        }
        let mut runtime = RealtimeRuntime::new(&program);
        runtime.seed_historical(&bars[..2]).unwrap();
        for _ in 0..3 {
            let result = runtime.update(BarUpdate::forming(bars[2])).unwrap();
            let values: Vec<_> = result
                .plots
                .iter()
                .map(|p| p.values.last().and_then(PineValue::as_f64))
                .collect();
            assert_eq!(values, expected, "{name}/forming");
        }
        let result = runtime.update(BarUpdate::confirmed(bars[2])).unwrap();
        assert_eq!(
            result
                .plots
                .iter()
                .map(|p| p.values.last().and_then(PineValue::as_f64))
                .collect::<Vec<_>>(),
            expected,
            "{name}/confirmed"
        );
    }
}

#[test]
fn promoted_collections_keep_aliases_and_rollback_forming_updates() {
    for version in [5, 6] {
        let source = format!(
            "//@version={version}\nindicator(\"promoted aliases\")\nvar m=map.new<string,int>()\nvar x=matrix.new<int>(1,1,1)\nif barstate.isfirst\n    m.put(\"x\",1)\nf(map<string,float> v)=>\n    v.put(\"x\",v.get(\"x\")+0.5)\n    v.get(\"x\")\ng(matrix<float> v)=>\n    v.set(0,0,v.get(0,0)+0.5)\n    v.get(0,0)\nplot(f(m))\nplot(m.get(\"x\"))\nplot(g(x))\nplot(x.get(0,0))\n"
        );
        let analysis = analyze_source(&SourceFile::new("rollback.pine", source));
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let program = analysis.hir.unwrap();
        let mut runtime = RealtimeRuntime::new(&program);
        runtime.seed_historical(&[bar(0), bar(1)]).unwrap();
        for i in 2..4 {
            for _ in 0..3 {
                let result = runtime.update(BarUpdate::forming(bar(i))).unwrap();
                for plot in &result.plots {
                    assert_eq!(
                        plot.values.last().and_then(PineValue::as_f64),
                        Some(1. + (i + 1) as f64 * 0.5)
                    );
                }
            }
            runtime.update(BarUpdate::confirmed(bar(i))).unwrap();
        }
    }
}

#[test]
fn native_simple_array_reference_parameters_preserve_mutation() {
    for version in [5, 6] {
        let source = format!(
            "//@version={version}\nindicator(\"simple array\")\nf(simple array<float> v)=>\n    v.push(20)\n    v.size()\nvalues=array.new<float>(1,10)\nplot(f(values))\nplot(values.size())\n"
        );
        let analysis = analyze_source(&SourceFile::new("simple-array.pine", source));
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let program = analysis.hir.unwrap();
        let mut runtime = RealtimeRuntime::new(&program);
        runtime.seed_historical(&[bar(0), bar(1)]).unwrap();
        for _ in 0..3 {
            let result = runtime.update(BarUpdate::forming(bar(2))).unwrap();
            for p in &result.plots {
                assert_eq!(
                    p.values.iter().map(PineValue::as_f64).collect::<Vec<_>>(),
                    [Some(2.); 3]
                );
            }
        }
        runtime.update(BarUpdate::confirmed(bar(2))).unwrap();
    }
}

#[test]
fn imported_parameter_mutation_retains_aliases_and_fractional_values() {
    for version in [5, 6] {
        let library = format!(
            "//@version={version}\nlibrary(\"CollectionContract\")\nexport bump(map<string,float> v)=>\n    v.put(\"x\",v.get(\"x\")+0.5)\n    v.get(\"x\")\nexport step(matrix<float> v)=>\n    v.set(0,0,v.get(0,0)+0.5)\n    v.get(0,0)\n"
        );
        let root = format!(
            "//@version={version}\nimport local/CollectionContract/1 as c\nindicator(\"imported collection\")\nvar m=map.new<string,int>()\nvar x=matrix.new<int>(1,1,1)\nif barstate.isfirst\n    m.put(\"x\",1)\nplot(c.bump(m))\nplot(m.get(\"x\"))\nplot(c.step(x))\nplot(x.get(0,0))\n"
        );
        let input = pine_sema::AnalysisInput::with_library_sources(
            SourceFile::new("root.pine", root),
            vec![(
                "local/CollectionContract/1".to_owned(),
                SourceFile::new("library.pine", library),
            )],
        )
        .unwrap();
        let analysis = pine_sema::analyze_input(&input);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let program = analysis.hir.unwrap();
        let mut runtime = RealtimeRuntime::new(&program);
        runtime.seed_historical(&[bar(0), bar(1)]).unwrap();
        for _ in 0..3 {
            let result = runtime.update(BarUpdate::forming(bar(2))).unwrap();
            for p in &result.plots {
                assert_eq!(
                    p.values.iter().map(PineValue::as_f64).collect::<Vec<_>>(),
                    [Some(1.5), Some(2.), Some(2.5)]
                );
            }
        }
        runtime.update(BarUpdate::confirmed(bar(2))).unwrap();
    }
}

#[test]
fn library_parameter_mutation_does_not_admit_global_captures_or_plot_calls() {
    for version in [5, 6] {
        for body in [
            "g=map.new<string,float>()\nexport value()=>\n    g.put(\"x\",1)\n    g.get(\"x\")",
            "export value()=>\n    plot(1)\n    1",
            "export value()=>input.float(1)",
        ] {
            let root = format!(
                "//@version={version}\nimport local/Contract/1 as c\nindicator(\"invalid library\")\nplot(c.value())\n"
            );
            let library = format!("//@version={version}\nlibrary(\"Contract\")\n{body}\n");
            let input = pine_sema::AnalysisInput::with_library_sources(
                SourceFile::new("root.pine", root),
                vec![(
                    "local/Contract/1".to_owned(),
                    SourceFile::new("library.pine", library),
                )],
            )
            .unwrap();
            let analysis = pine_sema::analyze_input(&input);
            assert!(
                analysis.hir.is_none() && !analysis.diagnostics.is_empty(),
                "{body}"
            );
        }
    }
}

#[test]
fn native_valid_array_libraries_mutate_shared_parameters_with_rollback() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join(
            "docs/evidence/strategy-library-20261010/collection-import-supplementary-45.json",
        ))
        .unwrap(),
    )
    .unwrap();
    let cases: Vec<_> = manifest["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| {
            c["name"]
                .as_str()
                .unwrap()
                .starts_with("library-mutable-array-")
        })
        .collect();
    assert_eq!(cases.len(), 2);
    for case in cases {
        let version = case["version"].as_u64().unwrap();
        let library = std::fs::read_to_string(root.join(case["source"].as_str().unwrap())).unwrap();
        let source = format!(
            "//@version={version}\nimport local/Contract/1 as c\nindicator(\"array import\")\nvar values=array.new<float>(1,10)\nplot(c.step(values))\nplot(values.size())\n"
        );
        let input = pine_sema::AnalysisInput::with_library_sources(
            SourceFile::new("root.pine", source),
            vec![(
                "local/Contract/1".to_owned(),
                SourceFile::new("library.pine", library),
            )],
        )
        .unwrap();
        let analysis = pine_sema::analyze_input(&input);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let program = analysis.hir.unwrap();
        let mut runtime = RealtimeRuntime::new(&program);
        runtime.seed_historical(&[bar(0), bar(1)]).unwrap();
        for _ in 0..3 {
            let result = runtime.update(BarUpdate::forming(bar(2))).unwrap();
            for p in &result.plots {
                assert_eq!(
                    p.values.iter().map(PineValue::as_f64).collect::<Vec<_>>(),
                    [Some(2.), Some(3.), Some(4.)]
                );
            }
        }
        runtime.update(BarUpdate::confirmed(bar(2))).unwrap();
    }
}

#[test]
fn native_library_parameter_aliases_and_rebinding_execute_with_stable_state() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest: serde_json::Value =
        serde_json::from_str(
            &std::fs::read_to_string(root.join(
                "docs/evidence/strategy-library-20261010/collection-library-ownership-45.json",
            ))
            .unwrap(),
        )
        .unwrap();
    for case in manifest["records"].as_array().unwrap() {
        let library = std::fs::read_to_string(root.join(case["source"].as_str().unwrap())).unwrap();
        let body = if case["kind"] == "loop" {
            "var values=array.new<int>()\nr=c.f(values)\nplot(values.size())\nplot(r.size())"
        } else {
            "var values=array.new<float>(1,10)\nr=c.f(values)\nplot(r)\nplot(values.size())"
        };
        let source = format!(
            "//@version=6\nimport local/Ownership/1 as c\nindicator(\"ownership execution\")\n{body}\n"
        );
        let input = pine_sema::AnalysisInput::with_library_sources(
            SourceFile::new("root.pine", source),
            vec![(
                "local/Ownership/1".to_owned(),
                SourceFile::new("library.pine", library),
            )],
        )
        .unwrap();
        let analysis = pine_sema::analyze_input(&input);
        assert!(
            analysis.diagnostics.is_empty(),
            "{}: {:?}",
            case["name"],
            analysis.diagnostics
        );
        let program = analysis.hir.unwrap();
        let mut runtime = RealtimeRuntime::new(&program);
        runtime.seed_historical(&[bar(0), bar(1)]).unwrap();
        for _ in 0..3 {
            let result = runtime.update(BarUpdate::forming(bar(2))).unwrap();
            for (i, plot) in result.plots.iter().enumerate() {
                assert_eq!(
                    plot.values
                        .iter()
                        .map(PineValue::as_f64)
                        .collect::<Vec<_>>(),
                    case["local_three_bar_plots"][i]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_f64())
                        .collect::<Vec<_>>(),
                    "{}",
                    case["name"]
                );
            }
        }
        runtime.update(BarUpdate::confirmed(bar(2))).unwrap();
    }
}
