use pine_sema::analyze_source;
use pine_syntax::SourceFile;

#[test]
fn legacy_tuple_request_udfs_admit_local_dependencies_and_named_security_arguments() {
    for selector in [
        "symbol=\"B\",resolution=\"2\"",
        "symbol=syminfo.tickerid,resolution=timeframe.period",
    ] {
        let source = format!(
            "//@version=4\nstudy(\"tuple\")\nf(src,scale)=>\n    value=src*scale\n    [value,value>open,time]\n[a,b,c]=security({selector},expression=f(close,2))\nplot(a)\n"
        );
        let analysis = analyze_source(&SourceFile::new("tuple.pine", source));
        assert!(analysis.hir.is_some(), "{:?}", analysis.diagnostics);
    }
}

#[test]
fn legacy_tuple_request_udfs_keep_unsafe_capture_and_side_effect_rejections() {
    for declarations in [
        "var value=0.0\nvalue:=close\nf()=>[value,high]",
        "f()=>\n    label.new(bar_index,close)\n    [open,close]",
        "f()=>f()",
    ] {
        let source = format!(
            "//@version=4\nstudy(\"unsafe tuple\")\n{declarations}\n[a,b]=security(\"B\",\"2\",f())\nplot(a)\n"
        );
        let analysis = analyze_source(&SourceFile::new("unsafe.pine", &source));
        assert!(analysis.hir.is_none(), "{source}");
    }
}

#[test]
fn legacy_security_tuple_expressions_require_v4() {
    for version in [1, 2, 3] {
        let source = format!(
            "//@version={version}\nstudy(\"tuple boundary\")\nf()=>[open,high]\n[a,b]=security(tickerid,\"D\",f())\nplot(a+b)\n"
        );
        assert!(
            analyze_source(&SourceFile::new("boundary.pine", source))
                .hir
                .is_none()
        );
    }
}
