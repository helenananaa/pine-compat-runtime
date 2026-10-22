//! Atomic provider-only transactions for a subsequent chart observation.
use super::*;
use crate::request::RequestFeed;
use serde::{Deserialize, Serialize};

pub const REALTIME_REQUEST_FEED_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequestFeedUpdate {
    pub symbol: String,
    pub timeframe: String,
    pub bar: Bar,
    pub kind: BarUpdateKind,
}

impl RealtimeRuntime<'_> {
    /// Stage the complete transaction against confirmed and forming providers.
    /// This does not evaluate Pine, advance its revision or replace its output.
    /// The caller owns availability and the following chart observation clock.
    /// Existing apply_request_update keeps its automatic recalculation behavior.
    pub fn apply_request_feed_updates(
        &mut self,
        updates: &[RequestFeedUpdate],
    ) -> Result<(), RuntimeError> {
        if updates.is_empty() {
            return Ok(());
        }
        let confirmed = stage_feed(&self.confirmed, updates)?;
        let forming = self
            .forming
            .as_ref()
            .map(|runtime| stage_feed(runtime, updates))
            .transpose()?;
        self.confirmed.request_feed = confirmed;
        self.confirmed.request_cache.clear();
        if let (Some(runtime), Some(feed)) = (&mut self.forming, forming) {
            runtime.request_feed = feed;
            runtime.request_cache.clear();
        }
        Ok(())
    }

    /// Resolve a provider interval using the same calendar as request merging.
    pub fn request_bar_close(&self, key: &RequestKey, open: i64) -> Result<i64, RuntimeError> {
        self.confirmed
            .request_environment
            .nominal_close(key.symbol(), key.timeframe(), open)
    }
}

fn stage_feed(
    runtime: &HistoricalRuntime<'_>,
    updates: &[RequestFeedUpdate],
) -> Result<RequestFeed, RuntimeError> {
    let mut feed = runtime.request_feed.clone();
    for update in updates {
        let bar = update.bar;
        if update.symbol.is_empty()
            || update.symbol.trim() != update.symbol
            || update.timeframe.is_empty()
            || update.timeframe.trim() != update.timeframe
            || [bar.open, bar.high, bar.low, bar.close, bar.volume]
                .iter()
                .any(|value| !value.is_finite())
            || bar.high < bar.open.max(bar.close)
            || bar.low > bar.open.min(bar.close)
            || bar.volume < 0.0
        {
            return Err(RuntimeError {
                message: "invalid requested feed update".to_owned(),
            });
        }
        let timeframe =
            RequestTimeframe::parse(&update.timeframe).map_err(|error| RuntimeError {
                message: error.to_string(),
            })?;
        let key = RequestKey::new(&update.symbol, timeframe);
        let provider_last = match runtime.request_environment.provider().bars(&key) {
            Ok(bars) => bars.last().map(|bar| bar.time),
            Err(RequestDataError::MissingData { .. }) => None,
            Err(error) => {
                return Err(RuntimeError {
                    message: error.to_string(),
                });
            }
        };
        feed.apply(
            key,
            BarUpdate {
                bar,
                kind: update.kind,
            },
            provider_last,
        )
        .map_err(crate::request::RequestFeedError::runtime_error)?;
    }
    Ok(feed)
}
