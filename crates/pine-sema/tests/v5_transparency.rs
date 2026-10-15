use pine_sema::analyze_source;
use pine_syntax::SourceFile;

#[test]
fn v5_named_transparency_keeps_modern_parameters_and_v6_rejects_it() {
    let body = r#"indicator("outputs")
t=input.int(40)
bgcolor(color.green,transp=t,force_overlay=true)
plotshape(true,color=color.blue,transp=30,display=display.all,force_overlay=true)
plotchar(true,color=color.red,transp=20,force_overlay=true)
plotarrow(1,colorup=color.blue,colordown=color.red,transp=10,force_overlay=true)
plot(1,color=color.blue,transp=50,force_overlay=true)
"#;
    for version in [5, 6] {
        let analysis = analyze_source(&SourceFile::new(
            "transparency.pine",
            format!("//@version={version}\n{body}"),
        ));
        if version == 5 {
            assert!(
                analysis.diagnostics.is_empty(),
                "{:?}",
                analysis.diagnostics
            );
            assert!(analysis.hir.is_some());
        } else {
            assert!(analysis.hir.is_none());
            assert_eq!(
                analysis
                    .diagnostics
                    .iter()
                    .filter(|d| d.message.contains("transp"))
                    .count(),
                5,
                "{:?}",
                analysis.diagnostics
            );
        }
    }
}

#[test]
fn v5_transparency_preserves_type_order_duplicate_and_modern_argument_checks() {
    for (body, message) in [
        ("bgcolor(color.green,transp=bar_index%100)", "input integer"),
        ("bgcolor(color.green,transp=1.5)", "input integer"),
        ("plotshape(true,transp=30,transp=40)", "more than once"),
        ("bgcolor(transp=30,color.green)", "positional arguments"),
        (
            "bgcolor(color.green,transp=30,force_overlay=close)",
            "force_overlay",
        ),
        ("plot(1,transp=30,typo=1)", "invalid argument"),
        ("barcolor(color.green,transp=30)", "no argument named"),
    ] {
        let analysis = analyze_source(&SourceFile::new(
            "negative.pine",
            format!("//@version=5\nindicator(\"negative\")\n{body}\nplot(1)\n"),
        ));
        assert!(analysis.hir.is_none(), "{body}");
        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|d| d.message.contains(message)),
            "{body}: {:?}",
            analysis.diagnostics
        );
    }
}
