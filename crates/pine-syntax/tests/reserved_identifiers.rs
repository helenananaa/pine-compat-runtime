use pine_syntax::{SourceFile, parse_source};

const RESERVED: [&str; 13] = [
    "catch", "class", "do", "ellipse", "in", "is", "polygon", "range", "return", "struct", "text",
    "throw", "try",
];

#[test]
fn migration_reserved_names_are_versioned_at_declarations() {
    for version in [4, 5, 6] {
        for name in RESERVED {
            let source = SourceFile::new(
                "versioned.pine",
                format!("//@version={version}\nstudy(\"probe\")\n{name}=7\nplot({name})\n"),
            );
            let parsed = parse_source(&source);
            let errors = parsed
                .diagnostics
                .iter()
                .filter(|d| d.code == "E_RESERVED_IDENTIFIER")
                .collect::<Vec<_>>();
            assert_eq!(
                errors.len(),
                usize::from(version >= 5),
                "v{version} {name}: {:?}",
                parsed.diagnostics
            );
            for error in errors {
                assert_eq!(&source.text()[error.span.start..error.span.end], name);
            }
            if version == 4 {
                assert!(
                    parsed.diagnostics.is_empty(),
                    "{name}: {:?}",
                    parsed.diagnostics
                );
            }
        }
    }
}

#[test]
fn reserved_declarations_are_rejected_even_in_unused_bodies() {
    for body in [
        "text(value)=>value\nplot(close)",
        "f(text)=>text\nplot(close)",
        "f(float text=1)=>text\nplot(close)",
        "f()=>\n    text=11\n    text\nplot(close)",
        "var float text=17\nplot(text)",
        "for text=0 to 0\n    plot(text)",
        "type Payload\n    int text\nplot(close)",
        "type text\n    int value\nplot(close)",
        "method text(float self)=>self\nplot(close)",
        "import TradingView/ta/8 as text\nplot(close)",
    ] {
        let source = SourceFile::new(
            "binding.pine",
            format!("//@version=6\nindicator(\"probe\")\n{body}\n"),
        );
        let parsed = parse_source(&source);
        let errors = parsed
            .diagnostics
            .iter()
            .filter(|d| d.code == "E_RESERVED_IDENTIFIER")
            .collect::<Vec<_>>();
        assert_eq!(errors.len(), 1, "{body}: {:?}", parsed.diagnostics);
        assert_eq!(
            &source.text()[errors[0].span.start..errors[0].span.end],
            "text"
        );
    }
}

#[test]
fn native_destructuring_and_foreach_exceptions_remain_legal() {
    for version in [5, 6] {
        for name in RESERVED.into_iter().filter(|name| *name != "in") {
            for body in [
                format!("pair()=>[1,2]\n[{name},other]=pair()\nplot({name}+other)"),
                format!("total=0\nfor {name} in array.from(1)\n    total+={name}\nplot(total)"),
            ] {
                let source = SourceFile::new(
                    "exception.pine",
                    format!("//@version={version}\nindicator(\"probe\")\n{body}\n"),
                );
                let parsed = parse_source(&source);
                assert!(
                    parsed.diagnostics.is_empty(),
                    "v{version} {body}: {:?}",
                    parsed.diagnostics
                );
            }
        }
        for body in [
            "pair()=>[1,2]\n[in,other]=pair()\nplot(other)",
            "for in in array.from(1)\n    plot(close)",
        ] {
            let source = SourceFile::new(
                "grammar.pine",
                format!("//@version={version}\nindicator(\"probe\")\n{body}\n"),
            );
            let parsed = parse_source(&source);
            assert!(
                parsed
                    .diagnostics
                    .iter()
                    .any(|d| d.code == "E_RESERVED_IDENTIFIER"),
                "{:?}",
                parsed.diagnostics
            );
        }
    }
}

#[test]
fn named_arguments_members_and_case_distinctions_are_not_declarations() {
    let source = SourceFile::new(
        "names.pine",
        "//@version=6\nindicator(\"probe\")\nmyText=7\nText=3\nitem=label.new(bar_index,close,text=\"text\",style=label.style_none)\nplot(label.get_text(item)==\"text\"?myText+Text:0)\n",
    );
    let parsed = parse_source(&source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
}

#[test]
fn inferred_import_namespace_is_left_to_library_resolution() {
    let source = SourceFile::new(
        "namespace.pine",
        "//@version=6\nimport TradingView/text/1\nindicator(\"probe\")\nplot(close)\n",
    );
    let parsed = parse_source(&source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
}

#[test]
fn modern_reserved_variable_has_a_precise_native_matching_rejection() {
    let source = SourceFile::new(
        "reserved.pine",
        "//@version=5\nindicator(\"probe\")\ntext=7\nplot(text)\n",
    );
    let parsed = parse_source(&source);
    let errors = parsed
        .diagnostics
        .iter()
        .filter(|d| d.code == "E_RESERVED_IDENTIFIER")
        .collect::<Vec<_>>();
    assert_eq!(errors.len(), 1, "{:?}", parsed.diagnostics);
    let span = errors[0].span;
    assert_eq!(&source.text()[span.start..span.end], "text");
    assert_eq!(
        (
            source.line_col(span.start).line,
            source.line_col(span.start).column
        ),
        (3, 1)
    );
}
