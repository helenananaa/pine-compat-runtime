use pine_sema::analyze_source;
use pine_syntax::SourceFile;

#[test]
fn declaration_constraints_infer_only_parameters_that_can_satisfy_simple() {
    for version in [5, 6] {
        for body in [
            "average(float source, int length)=>ta.ema(source,length)\nplot(average(close,3))",
            "identity(value)=>value\naverage(int length)=>ta.ema(close,identity(length))\nplot(average(3))",
            "average(int length)=>\n    adjusted=length+1\n    ta.ema(close,adjusted)\nplot(average(3))",
            "method average(int self)=>ta.ema(close,self)\nlength=3\nplot(length.average())",
            "constant(int ignored)=>3\nplot(ta.ema(close,constant(bar_index)))",
            "constant(value)=>3\naverage(int length)=>ta.ema(close,constant(length))\nplot(average(bar_index))",
        ] {
            let analysis = analyze_source(&SourceFile::new(
                "inferred.pine",
                format!("//@version={version}\nindicator(\"qualifier\")\n{body}\n"),
            ));
            assert!(
                analysis.hir.is_some(),
                "{version}: {body}\n{:?}",
                analysis.diagnostics
            );
        }
        for body in [
            "average(int length)=>ta.ema(close,length)\nplot(average(bar_index+1))",
            "average(int length,bool enabled)=>enabled?ta.ema(close,length):close\nplot(average(bar_index+1,false))",
            "identity(int value)=>value\nplot(ta.ema(close,identity(3)))",
            "identity(int value)=>value\naverage(int length)=>ta.ema(close,identity(length))\nplot(average(3))",
            "need(const int value)=>value\nidentity(int value)=>need(value)\nplot(identity(3))",
            "choose(bool enabled)=>enabled?bar_index:3\nplot(ta.ema(close,choose(false)))",
        ] {
            let analysis = analyze_source(&SourceFile::new(
                "rejected.pine",
                format!("//@version={version}\nindicator(\"qualifier\")\n{body}\n"),
            ));
            assert!(
                analysis.hir.is_none(),
                "unexpected acceptance {version}: {body}"
            );
        }
    }
}

#[test]
fn supplied_library_methods_preserve_inferred_and_explicit_scalar_boundaries() {
    use pine_sema::{AnalysisInput, analyze_input};
    for parameter in ["int length", "simple int length", "series int length"] {
        for argument in ["3", "bar_index+1"] {
            let library = SourceFile::new(
                "library.pine",
                format!(
                    "//@version=6\nlibrary(\"qualifiers\")\nexport type Point\n    float value\nmethod average(Point self,{parameter})=>ta.ema(self.value,length)\n"
                ),
            );
            let root = SourceFile::new(
                "root.pine",
                format!(
                    "//@version=6\nimport test/qualifiers/1 as lib\nindicator(\"qualifiers\")\np=lib.Point.new(close)\nplot(p.average({argument}))\nplot(lib.average(p,{argument}))\n"
                ),
            );
            let input = AnalysisInput::with_library_sources(
                root,
                vec![("test/qualifiers/1".into(), library)],
            )
            .unwrap();
            let report = analyze_input(&input);
            let accepted = argument == "3" && !parameter.starts_with("series");
            assert_eq!(
                report.hir.is_some(),
                accepted,
                "{parameter}: {argument}: {:?}",
                report.diagnostics
            );
        }
    }
}
