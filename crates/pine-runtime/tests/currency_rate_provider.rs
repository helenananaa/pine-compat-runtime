use std::sync::Arc;

use pine_runtime::{
    ChartContext, CurrencyRateError, CurrencyRateFrame, CurrencyRateProvider,
    InMemoryCurrencyRateProvider, NoCurrencyRateProvider, NoRequestDataProvider,
    RequestEnvironment, RequestTimeframe,
};

fn frame(effective_time: i64, rate: f64) -> CurrencyRateFrame {
    CurrencyRateFrame {
        effective_time,
        rate,
    }
}

#[test]
fn lookup_uses_only_confirmed_frames_available_at_the_requested_time() {
    let mut provider = InMemoryCurrencyRateProvider::default();
    provider
        .insert("USD", "EUR", vec![frame(100, 0.9), frame(200, 0.8)])
        .unwrap();
    assert!(matches!(
        provider.rate("USD", "EUR", 99),
        Err(CurrencyRateError::MissingHistory { time: 99, .. })
    ));
    assert_eq!(provider.rate("USD", "EUR", 100), Ok(0.9));
    assert_eq!(provider.rate("USD", "EUR", 199), Ok(0.9));
    assert_eq!(provider.rate("USD", "EUR", 200), Ok(0.8));
    assert_eq!(provider.rate("USD", "EUR", i64::MAX), Ok(0.8));
    // Looking backward after a later read must not leak the later frame.
    assert_eq!(provider.rate("USD", "EUR", 100), Ok(0.9));
}

#[test]
fn missing_stream_history_and_inverse_are_distinct_from_identity() {
    let mut provider = InMemoryCurrencyRateProvider::default();
    provider.insert("USD", "EUR", vec![]).unwrap();
    assert!(matches!(
        provider.rate("USD", "EUR", 0),
        Err(CurrencyRateError::MissingHistory { .. })
    ));
    assert!(matches!(
        provider.rate("EUR", "USD", 0),
        Err(CurrencyRateError::MissingPair { .. })
    ));
    assert!(matches!(
        NoCurrencyRateProvider.rate("USD", "USDT", 0),
        Err(CurrencyRateError::MissingPair { .. })
    ));
    assert_eq!(NoCurrencyRateProvider.rate("USD", "USD", i64::MIN), Ok(1.0));
}

#[test]
fn unavailable_pair_is_explicit_and_cannot_replace_supplied_or_missing_history() {
    let mut provider = InMemoryCurrencyRateProvider::default();
    provider.insert_unavailable("USD", "ZZZ").unwrap();
    assert!(matches!(
        provider.rate("USD", "ZZZ", 0),
        Err(CurrencyRateError::UnavailablePair { .. })
    ));
    assert!(matches!(
        provider.rate("USD", "EUR", 0),
        Err(CurrencyRateError::MissingPair { .. })
    ));
    assert!(matches!(
        provider.insert("USD", "ZZZ", vec![frame(0, 1.)]),
        Err(CurrencyRateError::DuplicatePair { .. })
    ));
    provider.insert("USD", "EUR", vec![]).unwrap();
    assert!(matches!(
        provider.insert_unavailable("USD", "EUR"),
        Err(CurrencyRateError::DuplicatePair { .. })
    ));
}

#[test]
fn invalid_stream_insertion_is_atomic_and_does_not_replace_existing_history() {
    let mut provider = InMemoryCurrencyRateProvider::default();
    for rate in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0, 0.0] {
        assert_eq!(
            provider.insert("USD", "EUR", vec![frame(10, 0.9), frame(20, rate)]),
            Err(CurrencyRateError::InvalidRate { index: 1 })
        );
        assert!(matches!(
            provider.rate("USD", "EUR", 10),
            Err(CurrencyRateError::MissingPair { .. })
        ));
    }
    provider.insert("USD", "EUR", vec![frame(10, 0.9)]).unwrap();
    assert!(matches!(
        provider.insert("USD", "EUR", vec![frame(10, 0.5)]),
        Err(CurrencyRateError::DuplicatePair { .. })
    ));
    assert_eq!(provider.rate("USD", "EUR", 10), Ok(0.9));
}

#[test]
fn repeated_and_descending_effective_times_are_rejected() {
    for second in [10, 9] {
        assert_eq!(
            InMemoryCurrencyRateProvider::default().insert(
                "USD",
                "EUR",
                vec![frame(10, 0.9), frame(second, 0.8)]
            ),
            Err(CurrencyRateError::UnorderedTimes { index: 1 })
        );
    }
}

#[test]
fn currency_codes_are_validated_even_for_same_currency_requests() {
    for code in ["", "usd", "USD/EUR", "U$D", " USD"] {
        assert_eq!(
            NoCurrencyRateProvider.rate(code, code, 0),
            Err(CurrencyRateError::InvalidCurrency(code.into()))
        );
        assert_eq!(
            InMemoryCurrencyRateProvider::default().insert(code, "EUR", vec![]),
            Err(CurrencyRateError::InvalidCurrency(code.into()))
        );
    }
}

struct HostRate(f64);
impl CurrencyRateProvider for HostRate {
    fn rate(&self, _from: &str, _to: &str, _time: i64) -> Result<f64, CurrencyRateError> {
        Ok(self.0)
    }
}

fn environment(provider: Arc<dyn CurrencyRateProvider>) -> RequestEnvironment {
    RequestEnvironment::new(
        ChartContext::new("HOST:SYMBOL", RequestTimeframe::parse("D").unwrap()),
        Arc::new(NoRequestDataProvider),
    )
    .with_currency_rates(provider)
}

#[test]
fn environment_rejects_invalid_callback_rates_and_keeps_identity_host_independent() {
    for rate in [f64::NAN, f64::INFINITY, -1.0, 0.0] {
        let env = environment(Arc::new(HostRate(rate)));
        assert_eq!(
            env.currency_rate("USD", "EUR", 0),
            Err(CurrencyRateError::InvalidResolvedRate)
        );
        assert_eq!(env.currency_rate("USD", "USD", 0), Ok(1.0));
    }
}

#[test]
fn cloned_environment_retains_the_explicit_rate_provider_and_calculation_time() {
    let mut provider = InMemoryCurrencyRateProvider::default();
    provider
        .insert("USDT", "EUR", vec![frame(100, 0.9), frame(200, 0.8)])
        .unwrap();
    let env = environment(Arc::new(provider));
    let cloned = env.clone();
    let requested = env.for_chart(ChartContext::new(
        "OTHER:SYMBOL",
        RequestTimeframe::parse("60").unwrap(),
    ));
    assert_eq!(requested.currency_rate("USDT", "EUR", 150), Ok(0.9));
    assert_eq!(cloned.currency_rate("USDT", "EUR", 150), Ok(0.9));
    assert_eq!(env.currency_rate("USDT", "EUR", 250), Ok(0.8));
    assert_eq!(cloned.currency_rate("USDT", "EUR", 150), Ok(0.9));
}
