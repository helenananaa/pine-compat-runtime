use pine_runtime::{CurrencyRateError, CurrencyRateProvider, currency_rates_from_json};

#[test]
fn schema_preserves_effective_times_and_distinguishes_unavailable_from_unsupplied() {
    let input = r#"{"schemaVersion":1,"pairs":[{"from":"USD","to":"EUR","rates":[{"effectiveTime":100,"rate":0.9},{"effectiveTime":200,"rate":0.8}]},{"from":"USD","to":"ZZZ","unavailable":true}]}"#;
    let provider = currency_rates_from_json(input).unwrap();
    assert!(matches!(
        provider.rate("USD", "EUR", 99),
        Err(CurrencyRateError::MissingHistory { .. })
    ));
    assert_eq!(provider.rate("USD", "EUR", 199), Ok(0.9));
    assert_eq!(provider.rate("USD", "EUR", 200), Ok(0.8));
    assert!(matches!(
        provider.rate("USD", "ZZZ", 200),
        Err(CurrencyRateError::UnavailablePair { .. })
    ));
    assert!(matches!(
        provider.rate("USD", "JPY", 200),
        Err(CurrencyRateError::MissingPair { .. })
    ));
}

#[test]
fn schema_rejects_ambiguous_unknown_or_incomplete_pair_inputs() {
    for input in [
        r#"{"schemaVersion":2,"pairs":[]}"#,
        r#"{"schemaVersion":1,"pairs":[],"guessRates":true}"#,
        r#"{"schemaVersion":1,"pairs":[{"from":"USD","to":"EUR"}]}"#,
        r#"{"schemaVersion":1,"pairs":[{"from":"USD","to":"EUR","unavailable":true,"rates":[]}]}"#,
        r#"{"schemaVersion":1,"pairs":[{"from":"USD","to":"EUR","rates":[],"rate":1}]}"#,
        r#"{"schemaVersion":1,"pairs":[{"from":"USD","to":"EUR","rates":[{"effectiveTime":1.5,"rate":1}]}]}"#,
    ] {
        assert!(currency_rates_from_json(input).is_err(), "{input}");
    }
}

#[test]
fn schema_validates_values_order_and_pair_uniqueness_before_use() {
    for input in [
        r#"{"schemaVersion":1,"pairs":[{"from":"USD","to":"EUR","rates":[{"effectiveTime":1,"rate":0}]}]}"#,
        r#"{"schemaVersion":1,"pairs":[{"from":"USD","to":"EUR","rates":[{"effectiveTime":1,"rate":1},{"effectiveTime":1,"rate":2}]}]}"#,
        r#"{"schemaVersion":1,"pairs":[{"from":"USD","to":"EUR","rates":[]},{"from":"USD","to":"EUR","unavailable":true}]}"#,
    ] {
        assert!(currency_rates_from_json(input).is_err(), "{input}");
    }
}

#[test]
fn an_explicit_empty_stream_is_warmup_history_not_a_missing_pair() {
    let provider = currency_rates_from_json(
        r#"{"schemaVersion":1,"pairs":[{"from":"USD","to":"EUR","rates":[]}]}"#,
    )
    .unwrap();
    assert!(matches!(
        provider.rate("USD", "EUR", 0),
        Err(CurrencyRateError::MissingHistory { .. })
    ));
}
