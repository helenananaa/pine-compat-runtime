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

- `strategy.entry` with explicit or supported default quantity, optionally with absolute limit/stop.
  Both prices describe a stop-limit order.
- `strategy.close` with optional qty or qty_percent, and `close_all`.
- `strategy.exit` with optional from_entry, absolute limit/stop, positive
  profit/loss distances or trailing activation and offset. Two exit prices
  describe an OCO bracket. Trailing requires exactly one of trail_price /
  trail_points, plus trail_offset.
- `strategy.cancel` and `cancel_all`, and the supported legacy `when` argument.

Development builds exposing `EXTERNAL_ORDER_SCHEMA_VERSION == 2` also emit
`strategy.order` with fixed positive quantities and optional limit/stop prices.
The host must transact that quantity directly, without entry pyramiding or
automatic reversal sizing. Both `entry` and `order` accept `oca_name` and
`oca_type`; intents preserve the name and normalize the type to `none`, `cancel`
or `reduce`. Omitted fields are absent from the wire. The host owns group
matching, partial-fill cancellation/reduction and all account changes. The
interpreter only emits intentions. Fixed defaults also apply when explicit
`qty=na` is used, with each expression evaluated once.

This additive development contract is exported by the Rust core and Python
module. WASM exports `externalOrderSchemaVersion()` as schema metadata; this
does not add a WASM external-account execution API. Package version remains
`0.3.2`; identify new wheels by source revision and hash. Tests cover external
batch/append equality and preservation of the caller's account authority.
Order comments/alert metadata require the separately negotiated development
contract below; they are not enabled by the order schema alone.

Development source also permits explicit non-na quantities when the strategy
declares cash/equity defaults: the explicit value overrides the declaration.
Zero explicit quantities and zero fixed defaults are no-ops and emit no intent,
including no replacement or cancellation of an earlier same-ID intent. Negative
and nonfinite quantities remain errors. Batch and incremental evaluation use the same rules;
an execution error still poisons incremental sessions.

These fixes do not change the order wire schema. A host must adopt the matching
capability in its admission and worker build before enabling defaults.
Tests: `crates/pine-runtime/tests/external_quantity_overrides.rs` and
`python/tests/test_external_quantity_overrides.py`. The explicit-quantity
precedence follows TradingView's [position-sizing documentation](https://www.tradingview.com/pine-script-docs/concepts/strategies/#position-sizing);
local tests are not captured official broker-output comparisons.

The 2026-10-20 focused checkpoint passed 33 Rust external/session/quantity
checks and 164 Python checks against a freshly built, isolated Windows wheel.
Wheel SHA256: `317eb3bd9047f83b9412786856b7dc0aa8d01860b500120c8c9a924d1424e439`.
The retained reproduction had three failing Rust cases before the repair.
Receipts and the wheel remain in `.local/host-completion/default-quantity-overrides/`.

## Evaluated order annotations (development)

Source work adds `EXTERNAL_ORDER_METADATA_SCHEMA_VERSION == 1`. Rust callers
enable `ExternalExecutionOptions.order_metadata`; Python uses the keyword-only
`order_metadata=True` on `run_external` or `external_session`. It defaults to
false. Executed annotation arguments reject without opt-in, so updating a core
wheel cannot make an older Host silently discard annotations. A false legacy
`when` does not evaluate annotations. Immediate-order matching remains outside
this contract and rejects independently.

With opt-in, entry/order/close/close_all preserve evaluated `comment`,
`alert_message` and `disable_alert`. Exit additionally preserves
`comment_profit`, `comment_loss`, `comment_trailing`, `alert_profit`,
`alert_loss` and `alert_trailing`. The optional intent `metadata` object contains
non-na string fields and a boolean `disable_alert`; empty strings remain
distinct from absent fields. Entirely default metadata is omitted, preserving
the previous wire for unannotated orders. Native annotation evaluators are
shared; expressions execute once. Template-like text remains literal here.

The concrete Host must bind metadata to the accepted order, retain it across
replacement/partial fills/recovery, select the actual exit leg's value with
generic fallback, and apply `disable_alert` to order-fill alert records without
dropping comments or financial fills. The core never fabricates Host fills,
chooses a future fill leg or performs external delivery. WASM exports
`externalOrderMetadataSchemaVersion()` as metadata only, without an external
account executor. On 2026-10-21, the focused Rust gate passed 14 external
metadata/default/session checks. A freshly built Windows wheel passed all 1,863
Python tests, including 25 metadata cases. Workspace/all-target Clippy,
formatting, source structure, host-parity guards and generated WASM/Node smoke
also passed. These are core/binding checks, separate from the preceding
default-sizing checkpoint's full Rust workspace result. The first focused
metadata test exposed a normalized exit-slot admission error; its fix preserves
the unsupported exit OCA-name boundary. Logs, diagnostic HIR and artifacts are
retained under `.local/host-completion/order-metadata/`.

## Authoritative default sizing (development)

`EXTERNAL_SIZING_SCHEMA_VERSION == 1` adds cash and percent-of-equity defaults
for omitted/na entry/order quantities and `strategy.default_entry_qty(fill_price)`.
Rust and Python export the constant; WASM exports `externalSizingSchemaVersion()`
as capability metadata only. The Host-requirements schema remains 3, with an
additive `account.defaultQuantity` containing `kind` (`fixed`, `cash`, or
`percent_of_equity`) and the compiled numeric `value` as a decimal string.
Hosts can freeze this declaration without executing the strategy.

Sizing reads equity from the authoritative account of the active execution
pass, or the current bar frame without explicit passes. The script's declared
initial capital never replaces that feedback. An explicit non-na quantity
still overrides defaults. Quantities follow the configured chart decimal grid;
negative values reject before truncation, and zero emits no intent. Negative
equity with a positive percentage budget and positive price rejects using the
existing native rule. Account expressions inside requested contexts remain
unsupported by semantic admission.

Entry/order defaults use the placement bar's close, matching this runtime's
native broker semantics even if the next fill gaps. The public default-quantity
query instead uses its supplied fill price. Both use the shared native formula,
currency-rate metadata and quantity rounding. This does not introduce future
fill-price knowledge, Host execution costs into the formula, nonunit point
values, or new order-account matching. Hosts retain fee/slippage/margin/fill
authority. The currently supported external profile still rejects native
commission/slippage declarations. These tests establish local/native agreement,
not newly captured TradingView broker-output equivalence.

The 2026-10-20 default-sizing checkpoint passed all 7,929 Rust workspace tests,
zero ignored, and all 1,838 checks against a freshly built Windows wheel
(including 216 focused checks).
The focused checks cover placement/fill gaps, explicit override, na/zero,
quantity grids, active-pass equity, FX input, batch/append equality, requested
account admission and poisoned-session errors. Clippy passed across all targets.
Wheel SHA256: `b1e0cac313b461874d702583948809f5c3fb0c72e086841ed1ecd341f3db49a4`.
All 172 tooling checks, source structure/host-parity guards, formatting and
generated WASM/Node smoke also passed. The first manual Node invocation used
a relative module path that a nested require could not resolve; the retained
absolute-path rerun passed without rebuilding or changing source. Evidence and
artifacts remain under `.local/host-completion/default-sizing/`.

Intents preserve quantities, prices, distances, IDs, direction and bar_index;
order expressions execute once. The result preserves declared pyramiding.
The host owns admission, replacements, OCO cancellation, tick conversion,
entry allocation and fills. Native commission, slippage, magnifier, default on-close
execution and every-tick recalculation reject without their separate contracts.
This is a specific external profile, not native broker parity.

## Declarative risk configuration (development)

`STRATEGY_RISK_CONFIGURATION_SCHEMA_VERSION == 1` adds
`Program.strategy_risk_configuration(input_overrides=None, chart_symbol=None,
chart_timeframe=None, request_bars=None)`. It discovers all six risk command
kinds in lowered HIR, including calls in unexecuted branches, without running
the script. The result contains `schemaVersion`, `rules`, and
`unresolvedCallSiteIds`. Each rule retains its `callSiteId`, `function`, resolved
numeric `value`/`valueType` or `direction`, optional `alertMessage`, and `resolved`.
Duplicate rules remain separate; a host must enforce every applicable rule.

Resolution is bounded to immutable scalar expressions, input overrides, known
chart metadata, and supported pure inlined functions. Mutable/series values,
side-effecting expressions and unsupported evaluation nodes remain unresolved.
Current numeric admission requires positive finite values, integral counts up to
`u32::MAX`, and percentage loss values at most 100. This is a bounded contract,
not acceptance of every valid Pine simple expression or every numeric boundary.
Original loops, requests, orders and side effects never execute during discovery.

External batch and session constructors accept keyword-only `risk_rules=True`.
Without it, any discovered risk call rejects, even in an unreachable branch.
With it, all arguments must resolve, risk calls become declarations and results
include a `risk_rules` configuration beside `intents`. No-risk output is unchanged.
This option delegates enforcement to the caller: it does **not** cap quantities,
create native fills, stop the account, or validate the caller's risk implementation.
Native-account execution is unchanged; this patch does not repair its conditional
risk registration or same-kind rule aggregation gaps.

Rust exports `strategy_risk_configuration` and `ExternalExecutionOptions.risk_rules`.
WASM exports `strategyRiskConfigurationSchemaVersion()` and
`Program.strategyRiskConfiguration(input_overrides_json, request_bars_json)` for
configuration discovery only. It does not add a WASM external-account executor.
Development artifacts retain package version `0.3.2`; identify them by hash.

On 2026-10-19 the complete Windows verification gate passed 7,912 Rust tests,
1,731 tests against a newly built/installed Python wheel, 172 tooling tests,
formatting, Clippy, source structure, host parity and generated WASM/Node smoke.
A separate generated Node check exercised risk schema 1, input overrides,
chart-grid resolution and discovery without executing an infinite script loop.
These are core/binding checks; Host risk execution and official output parity
require separate qualification. Earlier failed attempts are retained locally:
one changed constant-boolean identity semantics and was reverted; another hit
the Python facade line budget, resolved by extracting metadata bindings.

## Execution passes and requested data

The optional [explicit schedule API](EXTERNAL_SCHEDULE_V1.md) separates historical
or realtime script state from observation commit. The legacy `execution_passes`
rules below remain unchanged; using a new schedule requires explicit opt-in.

`execution_passes` supplies one nonempty group per chart bar, at most 65 passes
per group, and requires `calc_on_order_fills=true`. Without passes that setting
rejects. Each pass contains visible bar, authoritative account, ordered
`event_time_ms`, and confirmed. A confirmed pass must be last; only the final
group may end unconfirmed. Bar/account times match their group's chart bar.
Intents add pass_index.

Development builds exposing `EXTERNAL_CLOSING_TICK_SCHEMA_VERSION == 1` accept
the keyword-only `closing_ticks=True` on `Program.run_external` and
`Program.external_session`. Rust callers use
`with_external_accounts_and_options(..., ExternalExecutionOptions { closing_ticks: true })`.
Without this explicit opt-in, `process_orders_on_close=true` still rejects.
The option enables that declaration but never matches orders or invents account
feedback. With fill recalculation, consecutive confirmed passes are allowed
only when the declaration is enabled and their bars, timestamps and request
streams are identical. Accounts may change after Host fills. A confirmed pass
cannot become unconfirmed or revise market data. The 65-pass limit, rollback,
`varip` and resource constraints remain in force.

The ordinary rule that a confirmed pass must be last has this explicit closing-
tick exception. The host owns tick timing, fill prices, liquidity assumptions,
callbacks, feedback authority and transcript reconstruction. WASM exports
`externalClosingTickSchemaVersion()` as capability metadata; this does not add
a WASM external-account execution API. The opt-in is a development contract at
package version `0.3.2`; identify local builds by source/artifact hashes.

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
