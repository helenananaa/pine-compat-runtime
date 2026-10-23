use pine_runtime::{
    Bar, BarUpdate, HistoricalRuntime, PineValue, PreparedProgram, RealtimeRuntime,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;
use std::{path::Path, sync::Arc};

fn bar(index: i64) -> Bar {
    Bar {
        time: index * 60_000,
        open: 10.,
        high: 10.,
        low: 10.,
        close: 10.,
        volume: 1.,
    }
}

#[test]
fn native_legal_call_results_and_series_matrix_dimensions_across_execution_modes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cases: Vec<serde_json::Value> = [
        "function-body-positive-migration-oracles-46.json",
        "function-body-method-forwarding-oracles-46.json",
    ]
    .into_iter()
    .flat_map(|name| {
        let manifest: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                root.join("docs/evidence/strategy-library-20261010")
                    .join(name),
            )
            .unwrap(),
        )
        .unwrap();
        manifest["records"].as_array().unwrap().clone()
    })
    .collect();
    let bars: Vec<_> = (0..12).map(bar).collect();
    for case in &cases {
        let name = case["name"].as_str().unwrap();
        let source = std::fs::read_to_string(root.join(case["source"].as_str().unwrap())).unwrap();
        let analysis = analyze_source(&SourceFile::new(name, source));
        if case["native_status"] == "analysis_rejected" {
            assert!(analysis.hir.is_none(), "{name}");
            let expected_code = match case["kind"].as_str().unwrap() {
                "wrong-kind" => "E_FUNCTION_ARG_TYPE",
                "recursive" => "E_RECURSIVE_METHOD",
                "unknown-method" => "E_UNKNOWN_METHOD",
                _ => "E_CALL_ARG_TYPE",
            };
            assert!(
                analysis.diagnostics.iter().any(|d| d.code == expected_code),
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
        let expected: Vec<_> = case["native_values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64())
            .collect();
        let mut baseline = None;
        for mode in 0..3 {
            let mut runtime = if mode == 0 {
                HistoricalRuntime::new(&program)
            } else {
                HistoricalRuntime::from_prepared(&prepared)
            };
            if mode < 2 {
                runtime.append_bars(&bars).unwrap();
            } else {
                for bar in &bars {
                    runtime.append_bar(*bar).unwrap();
                }
            }
            let result = runtime.result();
            let values: Vec<_> = result
                .plots
                .iter()
                .map(|p| p.values.last().and_then(PineValue::as_f64))
                .collect();
            assert_eq!(values, expected, "{name}/{mode}");
            if let Some(baseline) = &baseline {
                assert_eq!(&result, baseline, "{name}/{mode}");
            } else {
                baseline = Some(result);
            }
        }
        let mut realtime = RealtimeRuntime::new(&program);
        for bar in &bars[..11] {
            realtime.update(BarUpdate::historical(*bar)).unwrap();
        }
        for update in [
            BarUpdate::forming(bars[11]),
            BarUpdate::forming(bars[11]),
            BarUpdate::confirmed(bars[11]),
        ] {
            realtime.update(update).unwrap();
            let result = realtime.result();
            let values: Vec<_> = result
                .plots
                .iter()
                .map(|p| p.values.last().and_then(PineValue::as_f64))
                .collect();
            assert_eq!(values, expected, "{name}/realtime");
        }
        assert_eq!(realtime.result(), baseline.unwrap(), "{name}/confirmed");
    }
}
