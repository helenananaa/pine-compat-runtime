use pine_sema::{AnalysisInput, analyze_input};
use pine_syntax::SourceFile;

#[test]
fn imported_array_admission_retains_effect_and_ownership_rejections() {
    for body in [
        "    array.push(values,1)\n    1",
        "    local = values\n    array.clear(local)\n    1",
        "    local = array.new<float>()\n    local := values\n    array.push(local,1)\n    1",
        "    var local = array.new<float>()\n    array.push(local,1)\n    local := values\n    1",
        "    var local = array.new<float>()\n    array.push(local,1)\n    if true\n        local := values\n    1",
        "    local = array.new<float>()\n    array.push(local,1)\n    plot(1)\n    1",
        "    local = array.new<float>()\n    array.push(local,plot(1))\n    1",
        "    local = array.new<float>()\n    array.push(local,1)\n    switch\n        true =>\n            for i = 0 to 1\n                plot(i)\n            1\n        => 0",
    ] {
        let input = AnalysisInput::with_library_sources(
            SourceFile::new("root.pine", "//@version=6\nimport test/arrays/1 as lib\nindicator(\"test\")\nplot(close)\n"),
            vec![("test/arrays/1".into(),SourceFile::new("library.pine",format!("//@version=6\nlibrary(\"arrays\")\nexport f(array<float> values) =>\n{body}\n")))],
        ).unwrap();
        let analysis = analyze_input(&input);
        assert!(analysis.hir.is_none(), "accepted {body}");
        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|d| d.code == "E_IMPORT_FUNCTION_SIDE_EFFECT"),
            "{body}: {:?}",
            analysis.diagnostics
        );
    }
}

#[test]
fn local_array_support_does_not_admit_global_receivers() {
    for receiver in ["global", "alias"] {
        let source = SourceFile::new(
            "root.pine",
            format!(
                "//@version=6\nindicator(\"test\")\nvar global=array.new<float>()\nf() =>\n    alias=global\n    array.push({receiver},1)\n    array.size({receiver})\nplot(f())\n"
            ),
        );
        let analysis = pine_sema::analyze_source(&source);
        assert!(analysis.hir.is_none(), "accepted {receiver}");
        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|d| d.message.contains("inside user-defined functions")),
            "{:?}",
            analysis.diagnostics
        );
    }
}
