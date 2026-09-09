use pine_sema::{AnalysisInput, analyze_input};
use pine_syntax::SourceFile;

fn analyze(root: &str, libraries: &[(&str, &str)]) -> pine_sema::Analysis {
    let root = SourceFile::new("root.pine", format!("//@version=6\n{root}"));
    let libraries = libraries
        .iter()
        .map(|(key, text)| {
            (
                key.to_string(),
                SourceFile::new(*key, format!("//@version=6\n{text}")),
            )
        })
        .collect();
    analyze_input(&AnalysisInput::with_library_sources(root, libraries).unwrap())
}

#[test]
fn nested_exports_resolve_and_private_names_stay_private() {
    let inner = "library(\"inner\")\nsecret(float x) => x+1\nexport value(float x) => secret(x)\n";
    let outer = "library(\"outer\")\nimport test/inner/1 as inner\nexport value(float x) => inner.value(x)\n";
    let root = "import test/outer/1 as outer\nindicator(\"nested\")\nplot(outer.value(close))\n";
    let good = analyze(root, &[("test/outer/1", outer), ("test/inner/1", inner)]);
    assert!(good.hir.is_some(), "{:?}", good.diagnostics);
    let private_outer = outer.replace("inner.value(x)", "inner.secret(x)");
    let bad = analyze(
        root,
        &[("test/outer/1", &private_outer), ("test/inner/1", inner)],
    );
    assert!(bad.hir.is_none());
    assert!(
        bad.diagnostics
            .iter()
            .any(|d| d.code == "E_IMPORT_PRIVATE_SYMBOL")
    );
    let missing = analyze(root, &[("test/outer/1", outer)]);
    assert!(missing.hir.is_none());
    assert!(
        missing
            .diagnostics
            .iter()
            .any(|d| d.code == "E_IMPORT_MISSING_LIBRARY")
    );
}

#[test]
fn imported_namespace_keeps_builtin_fallback_and_export_precedence() {
    let library = "library(\"ta\")\nexport adjusted(float x) => ta.sma(x,2)\n";
    let root = "import test/ta/1 as ta\nindicator(\"fallback\")\nplot(ta.adjusted(close))\nplot(ta.sma(close,2))\n";
    let result = analyze(root, &[("test/ta/1", library)]);
    assert!(result.hir.is_some(), "{:?}", result.diagnostics);
    let bad = analyze(
        &root.replace("ta.sma(close,2)", "ta.not_an_export(close)"),
        &[("test/ta/1", library)],
    );
    assert!(bad.hir.is_none());
    assert!(
        bad.diagnostics
            .iter()
            .any(|d| d.code == "E_IMPORT_UNKNOWN_EXPORT")
    );
}

#[test]
fn nested_cycles_are_diagnosed_without_recursive_expansion() {
    let root = "import test/a/1 as a\nindicator(\"cycle\")\nplot(close)\n";
    let a = "library(\"a\")\nimport test/b/1 as b\nexport f(float x) => b.f(x)\n";
    let b = "library(\"b\")\nimport test/a/1 as a\nexport f(float x) => a.f(x)\n";
    let result = analyze(root, &[("test/a/1", a), ("test/b/1", b)]);
    assert!(result.hir.is_none());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == "E_IMPORT_CYCLE")
    );
}

#[test]
fn nested_alias_declarations_are_checked_before_binding() {
    let root = "import test/outer/1 as outer\nindicator(\"aliases\")\nplot(outer.value(close))\n";
    let inner = "library(\"inner\")\nexport value(float x) => x\n";
    for (outer, code) in [
        (
            "library(\"outer\")\nimport test/inner/1\nexport value(float x) => inner.value(x)\n",
            "E_IMPORT_ALIAS_REQUIRED",
        ),
        (
            "library(\"outer\")\nimport test/inner/1 as inner\nimport test/other/1 as inner\nexport value(float x) => inner.value(x)\n",
            "E_IMPORT_DUPLICATE_ALIAS",
        ),
    ] {
        let result = analyze(
            root,
            &[
                ("test/outer/1", outer),
                ("test/inner/1", inner),
                ("test/other/1", inner),
            ],
        );
        assert!(result.hir.is_none());
        assert!(
            result.diagnostics.iter().any(|d| d.code == code),
            "{:?}",
            result.diagnostics
        );
    }
}
