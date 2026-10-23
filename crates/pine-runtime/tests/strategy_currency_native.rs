use pine_runtime::{
    Bar, ChartContext, NoRequestDataProvider, RequestEnvironment, RequestTimeframe,
    currency_rates_from_json, run_historical_with_request_environment,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;
use std::{path::Path, sync::Arc};

fn inputs(timeframe: &str) -> (Vec<Bar>, RequestEnvironment) {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/evidence/strategy-library-20261010");
    let csv = std::fs::read_to_string(dir.join(format!(
        "fx-native-chart-{}-31.csv",
        timeframe.to_lowercase()
    )))
    .unwrap();
    let mut bars = Vec::new();
    for line in csv.lines().skip(1) {
        let cells: Vec<_> = line.split(',').collect();
        let time = cells[0].parse().unwrap();
        bars.push(Bar {
            time,
            open: cells[1].parse().unwrap(),
            high: cells[2].parse().unwrap(),
            low: cells[3].parse().unwrap(),
            close: cells[4].parse().unwrap(),
            volume: 0.,
        });
    }
    let rates = currency_rates_from_json(
        &std::fs::read_to_string(dir.join("fx-native-rates-input-31.json")).unwrap(),
    )
    .unwrap();
    let chart = ChartContext::new(
        "BINANCE:BTCUSDT",
        RequestTimeframe::parse(timeframe).unwrap(),
    )
    .with_currency("USDT")
    .unwrap()
    .with_quantity_precision(5)
    .unwrap();
    (
        bars,
        RequestEnvironment::new(chart, Arc::new(NoRequestDataProvider))
            .with_currency_rates(Arc::new(rates)),
    )
}

#[test]
fn native_account_conversion_quantity_and_ledger_snapshots_match_exported_inputs() {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/evidence/strategy-library-20261010");
    let records: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("fx-native-ledger-oracles-31.json")).unwrap(),
    )
    .unwrap();
    for record in records.as_array().unwrap() {
        let name = record["stem"].as_str().unwrap();
        let source = std::fs::read_to_string(dir.join(format!("{name}.pine"))).unwrap();
        let analysis = analyze_source(&SourceFile::new(name, source));
        assert!(
            analysis.diagnostics.is_empty(),
            "{name}: {:?}",
            analysis.diagnostics
        );
        let (bars, environment) = inputs(record["chart_timeframe"].as_str().unwrap());
        let result =
            run_historical_with_request_environment(&analysis.hir.unwrap(), &bars, environment)
                .unwrap();
        let expected = record["native_values"].as_array().unwrap();
        assert_eq!(result.plots.len(), expected.len(), "{name}");
        for (index, (plot, expected)) in result.plots.iter().zip(expected).enumerate() {
            let value = plot.values.last().unwrap().as_f64();
            if expected.is_null() {
                assert_eq!(value, None, "{name} plot {index}");
            } else {
                let actual = value.unwrap();
                let expected = expected.as_f64().unwrap();
                assert!(
                    (actual - expected).abs() <= 5.1e-9,
                    "{name} plot {index}: {actual} != {expected}"
                );
            }
        }
    }
}

#[test]
fn historical_rates_do_not_use_a_later_execution_wall_clock() {
    let source =
        include_str!("../../../docs/evidence/strategy-library-20261010/fx-query-v6-oracle-31.pine");
    let analysis = analyze_source(&SourceFile::new("clock.pine", source));
    assert!(analysis.diagnostics.is_empty());
    let (bars, environment) = inputs("D");
    let execution_times = vec![1792195200000; bars.len()];
    let result=pine_runtime::run_historical_with_request_environment_and_input_overrides_and_execution_times(&analysis.hir.unwrap(),&bars,environment,pine_runtime::InputOverrides::default(),&execution_times).unwrap();
    assert_eq!(
        result.plots[0].values.last().unwrap().as_f64(),
        Some(0.8874)
    );
    assert_eq!(result.plots[1].values.last().unwrap().as_f64(), Some(88.74));
}

#[test]
fn native_realtime_week_and_month_queries_keep_the_chart_open_rate() {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/evidence/strategy-library-20261010");
    let records: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("fx-native-realtime-oracles-31.json")).unwrap(),
    )
    .unwrap();
    for record in records.as_array().unwrap() {
        let name = record["stem"].as_str().unwrap();
        let source = std::fs::read_to_string(dir.join(format!("{name}.pine"))).unwrap();
        let analysis = analyze_source(&SourceFile::new(name, source));
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let expected = record["native_values"].as_array().unwrap();
        let open = expected[4].as_f64().unwrap() as i64;
        let clock = expected[5].as_f64().unwrap() as i64;
        let (_, environment) = inputs(record["chart_timeframe"].as_str().unwrap());
        let hir = analysis.hir.unwrap();
        let mut runtime =
            pine_runtime::RealtimeRuntime::with_request_environment(&hir, environment);
        // These query-only sources never read OHLC or volume; quantity uses
        // an explicit 100 quote-unit fill price and there are no orders.
        let bar = Bar {
            time: open,
            open: 100.,
            high: 100.,
            low: 100.,
            close: 100.,
            volume: 0.,
        };
        for execution_time in [clock, clock + 60000] {
            let result = runtime
                .update_with_execution_time(pine_runtime::BarUpdate::forming(bar), execution_time)
                .unwrap();
            for (index, plot) in result.plots.iter().enumerate() {
                let actual = plot.values.last().unwrap().as_f64().unwrap();
                let expected = if index == 5 {
                    execution_time as f64
                } else {
                    expected[index].as_f64().unwrap()
                };
                assert!(
                    (actual - expected).abs() <= 5.1e-9,
                    "{name} plot {index}: {actual} != {expected}"
                );
            }
        }
    }
}
