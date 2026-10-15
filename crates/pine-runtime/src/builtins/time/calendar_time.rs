use super::{
    BarTimeFunctionArgs,
    session::{TimeSession, TimeSessionTimezone},
    session_grid::virtual_session_time,
};
use crate::{HistoricalRuntime, PineValue, RequestTimeframe, RuntimeError};

impl HistoricalRuntime<'_> {
    pub(super) fn eval_calendar_bar_time(
        &mut self,
        args: &BarTimeFunctionArgs,
        requested: &RequestTimeframe,
        session: Option<&TimeSession>,
        timezone: TimeSessionTimezone,
        close_time: bool,
    ) -> Result<PineValue, RuntimeError> {
        let Some(time) = self.current_builtin_i64("time") else {
            return Ok(PineValue::Na);
        };
        let chart = self.request_environment.chart();
        let calendar = self
            .request_environment
            .chart_calendar()
            .expect("calendar dispatch");
        let err = |err: crate::BarCalendarError| RuntimeError {
            message: err.to_string(),
        };
        calendar.exact(chart.timeframe(), time).map_err(err)?;
        let base = calendar
            .select(chart.timeframe(), time, args.bars_back)
            .map_err(err)?
            .expect("validated chart opening");
        if let Some(session) = session {
            let chart_seconds = chart.timeframe().seconds();
            return virtual_session_time(
                base.open(),
                requested,
                chart_seconds,
                session,
                timezone,
                args.timeframe_bars_back,
                || self.charge_execution_steps(1),
            )
            .map(|bounds| {
                bounds.map_or(PineValue::Na, |(open, close)| {
                    PineValue::Int(if close_time { close } else { open })
                })
            });
        }
        let Some(selected) = calendar
            .select(requested, base.open(), args.timeframe_bars_back)
            .map_err(err)?
        else {
            return Ok(PineValue::Na);
        };
        let mut open = selected.open();
        let mut close = selected.close();
        // Native time-based D/W/M charts expose the daily session's end for
        // intraday time_close queries. Offset in requested interval space first.
        if requested.seconds() < 86_400 && chart.timeframe().seconds() >= 86_400 {
            let daily = RequestTimeframe::parse("D").expect("daily timeframe");
            let daily = calendar
                .select(&daily, selected.open(), 0)
                .map_err(err)?
                .ok_or_else(|| RuntimeError {
                    message: "requested intraday interval is outside the supplied daily calendar"
                        .into(),
                })?;
            open = daily.open();
            close = daily.close();
        }
        Ok(PineValue::Int(if close_time { close } else { open }))
    }
}
