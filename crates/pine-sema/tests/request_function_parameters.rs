use pine_sema::analyze_source;
use pine_syntax::SourceFile;

#[test]
fn official_value_graph_probe_is_admitted() {
    let source = SourceFile::new(
        "official-value-graph.pine",
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/request-value-graph-v5.pine"
        ),
    );
    let analysis = analyze_source(&source);
    assert!(analysis.hir.is_some(), "{:?}", analysis.diagnostics);
}

#[test]
fn request_scope_changes_keep_future_global_mutations_and_inner_mutable_requests_rejected() {
    for version in [5, 6] {
        for body in [
            "value=close\nplot(request.security(\"B\",\"2\",value))\nvalue:=open\n",
            "value=close\nif close>0\n    value:=open\nplot(request.security(\"B\",\"2\",value))\n",
            "f(src)=>\n    var value=0.0\n    value:=src\n    request.security(\"B\",\"2\",value)\nplot(request.security(\"B\",\"2\",f(close)))\n",
        ] {
            let source = format!(
                "//@version={version}\nindicator(\"capture boundary\",dynamic_requests=false)\n{body}"
            );
            let analysis = analyze_source(&SourceFile::new("capture.pine", source));
            assert!(analysis.hir.is_none(), "mutable request admitted: {body}");
            assert!(
                analysis
                    .diagnostics
                    .iter()
                    .any(|d| d.code == "E_UNSUPPORTED_FEATURE")
            );
        }
    }
}

#[test]
fn legacy_same_context_requests_reject_global_mutable_values() {
    for expression in ["value", "value[1]", "f()"] {
        let source = format!(
            "//@version=4\nstudy(\"mutable boundary\")\nvar value=0.0\nvalue:=close\nf()=>value\nplot(security(syminfo.tickerid,timeframe.period,{expression}))\n"
        );
        let analysis = analyze_source(&SourceFile::new("mutable.pine", source));
        assert!(
            analysis.hir.is_none(),
            "mutable dependency admitted: {expression}"
        );
        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|d| d.code == "E_UNSUPPORTED_FEATURE")
        );
    }
}

#[test]
fn requests_reconstruct_immutable_function_parameters_and_aliases() {
    for version in [5, 6] {
        for expression in ["close", "10", "close+open"] {
            let source = format!(
                "//@version={version}\nindicator(\"parameter requests\")\nf(src)=>\n    alias=src+timeframe.multiplier\n    request.security(\"B\",\"2\",alias)\ng(src)=>f(src)\nplot(g({expression}))\nplot(f(src=20))\n"
            );
            let analysis = analyze_source(&SourceFile::new("parameters.pine", source));
            assert!(analysis.hir.is_some(), "{:?}", analysis.diagnostics);
        }
    }
}

#[test]
fn request_parameters_preserve_external_mutable_capture_rejection() {
    for argument in ["value", "value+close"] {
        let source = format!(
            "//@version=6\nindicator(\"mutable caller\")\nvar value=0.0\nvalue:=close\nf(src)=>\n    alias=src+1\n    request.security(\"B\",\"2\",alias)\nplot(f({argument}))\n"
        );
        let analysis = analyze_source(&SourceFile::new("mutable.pine", source));
        assert!(
            analysis.hir.is_none(),
            "external mutable argument was admitted"
        );
    }
}

#[test]
fn request_local_loops_and_scalar_casts_are_admitted() {
    for version in [5, 6] {
        let source = format!(
            "//@version={version}\nindicator(\"loops and casts\")\nf(src)=>\n    value=0.0\n    for i=0 to 2\n        value+=src[i]\n    float(value)\nplot(request.security(\"B\",\"2\",f(close)))\n[a,b,c]=request.security(\"B\",\"2\",[int(close),float(time),bool(close)])\nplot(a)\nplot(b)\nplot(c ? 1 : 0)\n"
        );
        let analysis = analyze_source(&SourceFile::new("loops.pine", source));
        assert!(analysis.hir.is_some(), "{:?}", analysis.diagnostics);
    }
}

#[test]
fn request_loops_do_not_admit_external_mutable_values_or_drawing_effects() {
    for statement in ["value+=external[i]", "label.new(bar_index,close)"] {
        let source = format!(
            "//@version=6\nindicator(\"unsafe loop capture\")\nvar external=0.0\nexternal:=close\nf()=>\n    value=0.0\n    for i=0 to 2\n        {statement}\n    value\nplot(request.security(\"B\",\"2\",f()))\n"
        );
        let analysis = analyze_source(&SourceFile::new("unsafe.pine", source));
        assert!(analysis.hir.is_none());
        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|d| d.code == "E_UNSUPPORTED_FEATURE")
        );
    }
}
