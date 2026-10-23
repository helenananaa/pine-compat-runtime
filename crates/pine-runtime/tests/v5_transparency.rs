use pine_runtime::{Bar, PineValue, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn run(source: &str) -> pine_runtime::RuntimeResult {
    let analysis = analyze_source(&SourceFile::new("outputs.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    run_historical(
        &analysis.hir.unwrap(),
        &[Bar {
            time: 0,
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 1.0,
        }],
    )
    .unwrap()
}

#[test]
fn v5_named_transparency_reaches_colors_and_overlay_metadata() {
    let legacy = run(r#"//@version=5
indicator("outputs")
t=input.int(40)
bgcolor(color.green,transp=t,force_overlay=true)
plotshape(true,title="shape",color=color.blue,transp=30,display=display.all,force_overlay=true)
plotchar(true,title="char",color=color.red,transp=20,force_overlay=true)
plotarrow(1,title="arrow",colorup=color.blue,colordown=color.red,transp=10,force_overlay=true)
plot(1,title="line",color=color.blue,transp=50,force_overlay=true)
"#);
    let canonical = run(r#"//@version=6
indicator("outputs")
bgcolor(color.new(color.green,40),force_overlay=true)
plotshape(true,title="shape",color=color.new(color.blue,30),display=display.all,force_overlay=true)
plotchar(true,title="char",color=color.new(color.red,20),force_overlay=true)
plotarrow(1,title="arrow",colorup=color.new(color.blue,10),colordown=color.new(color.red,10),force_overlay=true)
plot(1,title="line",color=color.new(color.blue,50),force_overlay=true)
"#);
    assert_eq!(legacy.bg_colors[0].values, canonical.bg_colors[0].values);
    assert_eq!(
        legacy.plot_shapes[0].colors,
        canonical.plot_shapes[0].colors
    );
    assert_eq!(legacy.plot_chars[0].colors, canonical.plot_chars[0].colors);
    assert_eq!(
        legacy.plot_arrows[0].color_ups,
        canonical.plot_arrows[0].color_ups
    );
    assert_eq!(
        legacy.plot_arrows[0].color_downs,
        canonical.plot_arrows[0].color_downs
    );
    assert_eq!(legacy.plots[0].colors, canonical.plots[0].colors);
    for metadata in [
        &legacy.bg_colors[0].metadata,
        &legacy.plot_shapes[0].metadata,
        &legacy.plot_chars[0].metadata,
        &legacy.plot_arrows[0].metadata,
        &legacy.plots[0].metadata,
    ] {
        assert_eq!(metadata.force_overlay, PineValue::Bool(true));
    }
    assert_eq!(
        legacy.plot_chars[0].metadata.title,
        PineValue::String("char".into())
    );
}
