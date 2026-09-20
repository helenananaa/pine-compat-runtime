# Fixed historical horizon sessions

This is historical execution with a fixed supplied OHLCV horizon, not a realtime append stream.
Only bars advanced to the cursor are evaluated; last-bar flags refer to the fixed full horizon.
The final output must match the original complete historical execution. Backwards movement uses
an explicit saved state or rebuild; input acquisition, processes and durable checkpoints belong to callers.

Use `program.historical_session(bars, input_overrides=..., chart_symbol=..., chart_timeframe=...)`; `advance(target)` returns the native output and `fork()` clones process-local state. Optional `request_bars` and `magnifier_bars` use the same frozen input contracts as Program.run. All Pine bar and execution timestamps are milliseconds. Errors invalidate the host session; recreate it before continuing.
