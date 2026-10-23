use pine_sema::analyze_source;
use pine_syntax::SourceFile;

#[test]
fn request_mode_is_lowered_with_version_defaults_and_declaration_overrides() {
    for (version, declaration, expected) in [
        (4, "study(\"mode\")", false),
        (5, "indicator(\"mode\")", false),
        (6, "indicator(\"mode\")", true),
        (5, "indicator(\"mode\", dynamic_requests=true)", true),
        (6, "indicator(\"mode\", dynamic_requests=false)", false),
        (6, "strategy(\"mode\", dynamic_requests=false)", false),
    ] {
        let source = format!("//@version={version}\n{declaration}\nplot(close)\n");
        let analysis = analyze_source(&SourceFile::new("mode.pine", source));
        let hir = analysis
            .hir
            .unwrap_or_else(|| panic!("{:?}", analysis.diagnostics));
        assert_eq!(hir.dynamic_requests, expected, "{version}: {declaration}");
    }
}

#[test]
fn nested_admission_preserves_mutable_capture_boundaries() {
    for version in [5, 6] {
        let source = format!(
            "//@version={version}\nindicator(\"static dependency\", dynamic_requests=false)\ninner = request.security(\"\", \"\", timeframe.multiplier)\nplot(request.security(\"NYSE:IBM\", \"4\", inner))\n"
        );
        let analysis = analyze_source(&SourceFile::new("static.pine", source));
        assert!(analysis.hir.is_some(), "{:?}", analysis.diagnostics);
    }

    let analysis = analyze_source(&SourceFile::new(
        "mutable.pine",
        "//@version=6\nindicator(\"nested mutable capture\")\nvar value = 0.0\nvalue := close\nplot(request.security(\"NYSE:IBM\", \"4\", request.security(\"\", \"\", value)))\n",
    ));
    assert!(analysis.hir.is_none());
    assert!(analysis.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "E_UNSUPPORTED_FEATURE"
            && diagnostic.message.contains("request.security")
    }));
}

#[test]
fn named_nested_requests_preserve_selector_and_merge_arguments() {
    for version in [5, 6] {
        for dynamic in [false, true] {
            let source = format!(
                "//@version={version}\nindicator(\"named nested\", dynamic_requests={dynamic})\ninner=request.security(expression=timeframe.multiplier,symbol=syminfo.tickerid,timeframe=\"\",gaps=barmerge.gaps_on,lookahead=barmerge.lookahead_off)\nplot(request.security(expression=inner,timeframe=\"4\",symbol=syminfo.tickerid))\n"
            );
            let analysis = analyze_source(&SourceFile::new("named_nested.pine", source));
            assert!(analysis.hir.is_some(), "{:?}", analysis.diagnostics);
        }
    }
}

#[test]
fn named_request_binding_preserves_source_argument_order_errors() {
    for request in ["request.security", "request.security_lower_tf"] {
        for args in [
            "symbol=syminfo.tickerid, \"1\", close",
            "syminfo.tickerid, timeframe=\"1\", close",
            "symbol=syminfo.tickerid, timeframe=\"1\", expression=close, barmerge.gaps_off",
        ] {
            let source =
                format!("//@version=6\nindicator(\"argument order\")\nvalue={request}({args})\n");
            let analysis = analyze_source(&SourceFile::new("order.pine", source));
            assert!(
                analysis.hir.is_none(),
                "positional argument after named argument was admitted: {request}({args})"
            );
            assert!(
                analysis
                    .diagnostics
                    .iter()
                    .any(|diagnostic| { diagnostic.code == "E_CALL_ARG_ORDER" }),
                "{request}({args}): {:?}",
                analysis.diagnostics
            );
        }
    }
}
