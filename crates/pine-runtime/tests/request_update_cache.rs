use std::sync::Arc;

use pine_runtime::{
    Bar, BarUpdate, ChartContext, InMemoryRequestDataProvider, PineValue, RealtimeRuntime,
    RequestEnvironment, RequestKey, RequestTimeframe,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn bar(index: i64, value: f64) -> Bar {
    Bar {
        time: index * 60_000,
        open: value,
        high: value,
        low: value,
        close: value,
        volume: 1.0,
    }
}

#[test]
fn confirmed_inner_updates_invalidate_direct_and_static_parent_fallback_caches() {
    for dynamic in ["true", "false"] {
        for reference in [true, false] {
            let body = if reference {
                "inner=request.security(\"B\",\"1\",close)\nplot(request.security(\"C\",\"1\",inner))"
            } else {
                "plot(request.security(\"C\",\"1\",request.security(\"B\",\"1\",close)))"
            };
            let analysis = analyze_source(&SourceFile::new(
                "transitive",
                format!(
                    "//@version=6\nindicator(\"transitive\",dynamic_requests={dynamic})\n{body}"
                ),
            ));
            assert!(analysis.diagnostics.is_empty());
            let hir = analysis.hir.unwrap();
            let timeframe = RequestTimeframe::parse("1").unwrap();
            let key = RequestKey::new("B", timeframe.clone());
            let provider = InMemoryRequestDataProvider::from_streams([
                (key.clone(), vec![bar(0, 100.0)]),
                (
                    RequestKey::new("C", timeframe.clone()),
                    vec![bar(0, 1.0), bar(1, 1.0)],
                ),
            ])
            .unwrap();
            let environment =
                RequestEnvironment::new(ChartContext::new("A", timeframe), Arc::new(provider));
            let mut runtime = RealtimeRuntime::with_request_environment(&hir, environment);
            runtime.update(BarUpdate::historical(bar(0, 1.0))).unwrap();
            runtime
                .apply_request_update(key.clone(), BarUpdate::confirmed(bar(1, 200.0)))
                .unwrap();
            runtime.update(BarUpdate::confirmed(bar(1, 1.0))).unwrap();
            assert_eq!(
                runtime.result().plots[0].values,
                vec![PineValue::Float(100.0), PineValue::Float(200.0)],
                "dynamic={dynamic}/reference={reference}"
            );
            let before = runtime.result();
            let revision = runtime.revision();
            assert!(
                runtime
                    .apply_request_update(key, BarUpdate::confirmed(bar(1, 999.0)))
                    .is_err()
            );
            assert_eq!(runtime.result(), before);
            assert_eq!(runtime.revision(), revision);
        }
    }
}
