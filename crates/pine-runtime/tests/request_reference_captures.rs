use pine_runtime::{Bar, BarUpdate, PineValue, RealtimeRuntime, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn bar(i: i64) -> Bar {
    Bar {
        time: i * 60000,
        open: 10.,
        high: 10.,
        low: 10.,
        close: 10.,
        volume: 1.,
    }
}

#[test]
fn native_same_context_reference_arguments_preserve_chart_side_effects() {
    let cases = [
        (
            "p=chart.point.from_index(1,10)\nf(chart.point v)=>\n    v.price+=5\n    v.price",
            "f(p)",
            15.,
        ),
        (
            "var p=chart.point.from_index(1,10)\nf(chart.point v)=>\n    v.price+=5\n    v.price",
            "f(p)",
            25.,
        ),
        (
            "p=chart.point.from_index(1,10)\np.price:=20",
            "p.price",
            20.,
        ),
        (
            "p=chart.point.from_index(1,10)\np:=chart.point.from_index(1,20)\nf(chart.point v)=>\n    v.price+=5\n    v.price",
            "f(p)",
            25.,
        ),
        (
            "p=chart.point.from_index(1,10)\nf()=>\n    p.price+=5\n    p.price",
            "f()",
            15.,
        ),
        (
            "type Wrap\n    chart.point point\np=chart.point.from_index(1,10)\nw=Wrap.new(p)\nf(Wrap v)=>\n    v.point.price+=5\n    v.point.price",
            "f(w)",
            15.,
        ),
        (
            "p=chart.point.from_index(1,10)\nvalues=array.from(p)\nf(array<chart.point> values)=>\n    v=values.first()\n    v.price+=5\n    v.price",
            "f(values)",
            15.,
        ),
    ];
    for version in [5, 6] {
        for (body, value, expected) in cases {
            let text = format!(
                "//@version={version}\nindicator(\"request refs\")\n{body}\nr=request.security(syminfo.tickerid,timeframe.period,{value})\nplot(r)\nplot(p.price)\n"
            );
            let analysis = analyze_source(&SourceFile::new("request.pine", text));
            assert!(
                analysis.diagnostics.is_empty(),
                "{:?}",
                analysis.diagnostics
            );
            let result = run_historical(&analysis.hir.unwrap(), &[bar(0), bar(1), bar(2)]).unwrap();
            for plot in result.plots {
                assert_eq!(
                    plot.values.last().and_then(PineValue::as_f64),
                    Some(expected)
                );
            }
        }
    }
}

#[test]
fn ordinary_point_capture_rolls_back_between_forming_updates() {
    for version in [5, 6] {
        let text = format!(
            "//@version={version}\nindicator(\"forming reference\")\nvar p=chart.point.from_index(1,10)\nf(chart.point v)=>\n    v.price+=5\n    v.price\nr=request.security(syminfo.tickerid,timeframe.period,f(p))\nplot(r)\nplot(p.price)\n"
        );
        let analysis = analyze_source(&SourceFile::new("forming.pine", text));
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
            for plot in &result.plots {
                assert_eq!(plot.values.last().and_then(PineValue::as_f64), Some(25.));
            }
        }
        let result = runtime.update(BarUpdate::confirmed(bar(2))).unwrap();
        for plot in &result.plots {
            assert_eq!(plot.values.last().and_then(PineValue::as_f64), Some(25.));
        }
    }
}

#[test]
fn reference_capture_routes_that_require_private_replay_remain_explicit() {
    for version in [5, 6] {
        for selectors in [
            "syminfo.tickerid,\"W\",f(p)",
            "\"NASDAQ:AAPL\",timeframe.period,f(p)",
        ] {
            let source = format!(
                "//@version={version}\nindicator(\"private reference replay\")\np=chart.point.from_index(1,10)\nf(chart.point v)=>\n    v.price+=5\n    v.price\nplot(request.security({selectors}))\n"
            );
            let analysis = analyze_source(&SourceFile::new("private.pine", source));
            assert!(
                analysis
                    .diagnostics
                    .iter()
                    .any(|d| d.code == "E_UNSUPPORTED_FEATURE"),
                "{:?}",
                analysis.diagnostics
            );
        }
    }
}

#[test]
fn reference_admission_preserves_external_scalar_and_output_restrictions() {
    for version in [5, 6] {
        for body in [
            "x=10\nx:=20\nf()=>x",
            "var x=close\nf()=>x",
            "p=chart.point.from_index(1,10)\nf()=>\n    label.new(1,p.price)\n    p.price",
            "p=chart.point.from_index(1,10)\nf()=>\n    runtime.error(\"side effect\")\n    p.price",
            "p=chart.point.from_index(1,10)\nf()=>\n    request.security(syminfo.tickerid,\"W\",p.price)",
        ] {
            let source = format!(
                "//@version={version}\nindicator(\"request restrictions\")\n{body}\nplot(request.security(syminfo.tickerid,timeframe.period,f()))\n"
            );
            let analysis = analyze_source(&SourceFile::new("restriction.pine", source));
            assert!(
                !analysis.diagnostics.is_empty(),
                "incorrectly admitted: {body}"
            );
        }
    }
}

#[test]
fn complete_native_sources_preserve_references_and_rollback() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            root.join("docs/evidence/strategy-library-20261010/request-reference-oracles-43.json"),
        )
        .unwrap(),
    )
    .unwrap();
    for case in manifest["records"].as_array().unwrap() {
        let source = std::fs::read_to_string(root.join(case["source"].as_str().unwrap())).unwrap();
        let analysis = analyze_source(&SourceFile::new("native.pine", source));
        if case["native_status"] == "analysis_rejected" {
            assert!(
                analysis
                    .diagnostics
                    .iter()
                    .any(|d| d.code == "E_UNSUPPORTED_FEATURE"),
                "{}: {:?}",
                case["name"],
                analysis.diagnostics
            );
            continue;
        }
        assert!(
            analysis.diagnostics.is_empty(),
            "{}: {:?}",
            case["name"],
            analysis.diagnostics
        );
        let program = analysis.hir.unwrap();
        let expected: Vec<_> = case["historical_expected"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64())
            .collect();
        let result = run_historical(&program, &[bar(0), bar(1), bar(2)]).unwrap();
        assert_eq!(
            result
                .plots
                .iter()
                .map(|p| p.values.last().and_then(PineValue::as_f64))
                .collect::<Vec<_>>(),
            expected,
            "{}",
            case["name"]
        );
        let mut realtime = RealtimeRuntime::new(&program);
        realtime.seed_historical(&[bar(0), bar(1)]).unwrap();
        for _ in 0..3 {
            let result = realtime.update(BarUpdate::forming(bar(2))).unwrap();
            assert_eq!(
                result
                    .plots
                    .iter()
                    .map(|p| p.values.last().and_then(PineValue::as_f64))
                    .collect::<Vec<_>>(),
                expected,
                "{}",
                case["name"]
            );
        }
        let result = realtime.update(BarUpdate::confirmed(bar(2))).unwrap();
        assert_eq!(
            result
                .plots
                .iter()
                .map(|p| p.values.last().and_then(PineValue::as_f64))
                .collect::<Vec<_>>(),
            expected,
            "{}",
            case["name"]
        );
    }
}
