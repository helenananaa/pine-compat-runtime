# External account feedback and order intents

Available in the official Python wheel and Rust core starting with `0.3.1`.
The earlier implementation lived on `codex/external-broker-v1` and was absent
from the official `0.3.0` release. Upgrade the unpublished `0.3.0rc1` extension
by artifact identity, rather than version ordering alone.

```python
program.run_external(bars, accounts, input_overrides=None, chart_symbol=None,
                     chart_timeframe=None, execution_passes=None, request_bars=None)
```

Rust exposes `HistoricalRuntime::with_external_accounts` or
`with_external_accounts_and_passes`, followed by historical execution and
`external_intents()`. Bind feedback to a fresh runtime before execution.

The caller owns matching and accounting. Exactly one account frame is required
per supplied bar, with matching integer millisecond `time`, finite
`position_size`, `equity`, `initial_capital`, `netprofit`, `openprofit`, and
optional positive `position_avg_price` (required for an open position).
Unknown fields fail admission. Native broker phases do not execute. Owned and
borrowed results omit the native strategy ledger. Python returns
`{protocol: "external-broker/1", pyramiding: ..., intents: [...], output: {...}}`.
Account fields and their history come from feedback. Unsupported account fields
and order arguments raise `E_EXTERNAL_UNSUPPORTED`.

## Supported order subset

- Fixed-quantity `strategy.entry`, optionally with absolute limit/stop.
  Both prices describe a stop-limit order.
- `strategy.close` with optional qty or qty_percent, and `close_all`.
- `strategy.exit` with optional from_entry, absolute limit/stop, positive
  profit/loss distances or trailing activation and offset. Two exit prices
  describe an OCO bracket. Trailing requires exactly one of trail_price /
  trail_points, plus trail_offset.
- `strategy.cancel` and `cancel_all`, and the supported legacy `when` argument.

Intents preserve quantities, prices, distances, IDs, direction and bar_index;
order expressions execute once. The result preserves declared pyramiding.
The host owns admission, replacements, OCO cancellation, tick conversion,
entry allocation and fills. Native commission, slippage, magnifier, on-close
execution, every-tick recalculation and non-fixed default quantities reject.
This is a specific external profile, not native broker parity.

## Execution passes and requested data

`execution_passes` supplies one nonempty group per chart bar, at most 65 passes
per group, and requires `calc_on_order_fills=true`. Without passes that setting
rejects. Each pass contains visible bar, authoritative account, ordered
`event_time_ms`, and confirmed. A confirmed pass must be last; only the final
group may end unconfirmed. Bar/account times match their group's chart bar.
Intents add pass_index.

Passes execute against the prior committed evaluation checkpoint. Ordinary var
state rolls back; varip survives. Only final series values commit per bar.
barstate.isconfirmed and barstate.isnew reflect the pass. All passes share the
existing per-bar resource allowance.

`request_bars` uses the native SYMBOL:TIMEFRAME mapping. Optional pass-local
`request_data` replaces the provider with supplied {symbol, timeframe, bars}
streams and clears evaluation caches. A known empty stream aligns to na; an
absent stream fails. This exception is scoped to external mode.

Hosts freeze earlier input/account/pass transcripts, supply only decision-
visible data, and reject changed prior intents before accepting a suffix.
Reconstruction uses that transcript, not native broker snapshots. Acquisition,
scheduling, process lifetime, persistence and application policy remain outside
the core. There are no application or network dependencies.
