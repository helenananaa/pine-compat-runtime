# Language Scope

The project should start with an indicator-focused subset. Scope discipline is
more important than broad, incomplete support.

## Version Policy

The parser recognizes the standard `//@version=N` form for the closed v1
through v6 range:

```pine
//@version=1
//@version=2
//@version=3
//@version=4
//@version=5
//@version=6
```

A missing directive selects v1 with origin `implicit`; it is not treated as the
latest language. Leading comments, blank lines, and indentation before the
directive are accepted. Horizontal whitespace before or after `=` and trailing
whitespace are accepted, including the corpus-proven `//@version = 4`
spelling. The `//@version` prefix remains exact, so `// @version=6` is an
ordinary comment. A second recognized directive or a recognized directive
after a source statement is rejected before ordinary semantic analysis.
Versions outside v1-v6 and root/library version mismatches are also rejected
before lowering.

The analyzer carries the validated dialect into HIR so the runtime can select
version-specific behavior. For v1-v4, script-mode classification runs before
ordinary symbol and call diagnostics. The fixture-backed v1-v4 `study()`
subsets are executable through versioned declaration, input, alias, output,
and request translation. `strategy()` and any `strategy.*` use in v1-v4 stop with
one `E_LEGACY_STRATEGY_OUT_OF_SCOPE` diagnostic; legacy strategies are not in
scope. Explicit v5/v6 `indicator()` and `strategy()` continue through the
existing modern paths, and legacy-only declaration names are not activated for
modern sources.

The v1-v4 compatibility front-end uses version-ranged exact rules only after
source-visible lexical/user resolution fails. From v3 onward, globals declared
after a UDF body do not retroactively shadow fallback calls in that body;
earlier lexical values and user functions retain precedence. Exact
translations preserve their original span in `legacyTranslations` and lower to
canonical HIR names. The implemented legacy subset includes the bounded v1/v2
scalar declaration graph, historical bool/numeric arithmetic, comparison, and
condition conversions, v3 untyped-`na` inference, v1-v4 `study()` and focused
`input()` binding, the
conformance-listed exact aliases, the initial ten output families with
versioned transparency/style semantics, proven `plot()` style-enum domains
whose last evaluated value is the emitted metadata, v4/v5 series output
offsets whose final evaluated value applies to the complete rendered output,
strict `iff`,
structural history `offset`, the type-directed legacy `rsi` overload, weekday
session defaults, and pre-v6 strict logical evaluation. Recognized
multi-timeframe request forms that still need legacy execution semantics fail
as supported-known work instead of silently selecting modern behavior.

The v1/v2 graph distinguishes declarations that actually require
self/forward predeclaration from earlier source-order inference prerequisites.
A prior scalar `input()` may therefore feed a later self-history chain without
moving the input call, while current-value forward edges remain forbidden from
crossing input/request/output/mutation declarations. Unqualified `rising` and
`falling` are exact v1-v4 aliases of `ta.rising` and `ta.falling`; v5/v6 require
the namespace.

`timenow` is a `series int` in every dialect and reads an explicit
host-provided UNIX millisecond timestamp for the current script execution. A
historical batch supplies exactly one timestamp per bar; incremental and
realtime hosts supply one timestamp with each update. Missing input fails when
execution reaches `timenow`, and a supplied batch with a count different from
the bar count is rejected before execution. The deterministic core never reads
the process wall clock or substitutes `time`/`last_bar_time`.

Tuple declarations retain concrete destination element types for downstream
semantic analysis when their initializer has a concrete tuple type and every
initializer error is `E_UNSUPPORTED_FEATURE`. This suppresses
failure-derived unknown-symbol and operator-type diagnostics without claiming
execution support: the producer diagnostics remain and no HIR is emitted.
Recursive producers and initializers with any other error do not enter this
recovery path.

The released indicator profiles are evidence-ranked independently from their
individual conformance rows:

| Profile | Maturity | Fixed eligible corpus | Stable gate status |
| --- | --- | ---: | --- |
| v4 indicator | preview | 12 | below the provisional 50-script evidence gate |
| v3 indicator | preview | 7 | below the provisional 50-script evidence gate |
| v2 indicator | experimental | 2 | below the provisional 50-script evidence gate |
| implicit v1 indicator | experimental | 1 | below the provisional 50-script evidence gate |

All 22 fixed original indicators parse, analyze, lower, and run historically,
but that small non-representative corpus does not justify a stable or
language-wide claim. Feature admission remains defined by
`tests/fixtures/conformance.tsv`; execution-mode release coverage is frozen in
`tests/fixtures/legacy/release_profiles.tsv`. No maturity label enables legacy
strategies or an unlisted feature.

## Initial Supported Syntax

The first executable subset should be smaller than the first parseable subset.
The parser may recognize more syntax than the runtime accepts, but unsupported
runtime behavior must become diagnostics during semantic analysis.

Statements:

- version declaration comments
- `indicator(...)`
- variable declarations
- `var` declarations
- reassignment with `:=`
- scalar-variable compound reassignment with `+=`, `-=`, `*=`, `/=`, and `%=`
- `if` statements
- local blocks
- user-defined functions with positional and named arguments
- simple `for` loops only after the expression runtime and callsite state are
  stable

Expressions:

- literals: int, float, bool, string literals delimited by matching ASCII
  quotation marks or apostrophes, including deprecated single-line string
  wrapping whose space-indented continuation lines collapse to one space, v6
  triple-delimited multiline strings with literal line breaks and indentation,
  a shared 40,960-decoded-character literal limit, and color literals where
  applicable
- identifiers
- function calls
- named arguments
- tuple expressions and tuple assignment
- arithmetic operators: numeric `+`, `-`, `*`, `/`, `%`, plus string `+`
  concatenation with strongest-operand qualifier and `na` propagation
- parenthesized expressions, calls, and parameter declarations may wrap across
  physical lines with zero or more spaces of continuation indentation
- expressions outside parentheses may use legacy physical-line wrapping when
  each continuation is deeper than its local block and not indented to a
  multiple-of-four column
- as a narrowly corpus-backed implicit-v1 exception, a top-level ternary may
  continue on exactly four ASCII spaces when the physical boundary ends with
  `?`/`:` or the next line begins with `?`/`:`; explicit v1-v6 sources, tabs,
  local-block continuations, and ordinary multiple-of-four indentation retain
  structural layout
- comparison operators: `==`, `!=`, `>`, `>=`, `<`, `<=`
- logical operators: `and`, `or`, `not`
- ternary operator: `condition ? a : b`
- history operator: `expr[offset]`

Phase 1 executable subset:

- global declarations
- `if`/`else` blocks for expression statements, plot calls, reassignment, and
  block-local tuple declarations
- `for i = start to end` loops over inclusive integer ranges with optional
  `by step`, `break`, `continue`, and loop result assignment
- partial `while condition` statement loops with bool/`na` conditions, `break`,
  `continue`, local `var`, nested loops, and a runtime iteration guard
- partial `switch` expressions with condition arms, selector/case arms,
  expression results, and expression statement-block arms that end in a result
  expression, plus statement-context switch block arms for selected-arm side
  effects, outer reassignment, and loop-control propagation
- branch/loop interactions for `if` inside loops, loops inside `if`, `switch`
  inside loops, and loops inside UDF block bodies
- normal and tuple declarations scoped to an `if`/`else` branch
- global and local scalar `varip` declarations for int, float, bool, string,
  color, and `na`, plus scalar typed-array `varip` declarations for float, int,
  bool, string, and color arrays using either `array<type>` or `type[]`
  declaration syntax; scalar declarations have local declaration-site storage,
  UDF callsite-local storage, and realtime intrabar persistence, while supported
  array ids retain their backing contents across repeated forming updates; the
  closed Phase I boundary is summarized in
  `docs/PHASE_I_AUDIT.md`
- user-defined functions lowered by inlining
- arithmetic, comparison, logical, and ternary expressions
- constant history offsets and guarded dynamic integer history offsets
- `indicator`
- `strategy(...)` as a Phase G declaration subset with strategy-mode runtime
  output, positive const numeric `initial_capital`, and Phase L fixed default
  quantity settings through `default_qty_type=strategy.fixed` plus positive
  const numeric `default_qty_value`, plus positive integer const `pyramiding`
  for the accepted same-direction long market-entry subset; const bool
  `calc_on_order_fills` re-executes the script after historical fills, resumes
  the current bar's remaining open-high-low-close or open-low-high-close path
  from the fill mark, and can fill later price ticks on the same bar;
  historical execution remains one script pass per bar when that flag is false,
  with internal scheduler bar/tick/pass identity, profile pass counts, and a
  bounded extra-pass guardrail; forming-bar realtime updates restore the
  confirmed broker checkpoint so abandoned intrabar orders, cancellations,
  activations, fills, and alerts do not leak; const bool `calc_on_every_tick`
  executes strategy code on each host-provided forming update with `var`
  rollback and `varip` persistence and does not change historical bars;
  host-owned bar-magnifier lower-timeframe input is keyed by chart bar with
  explicit standard-OHLC fallback; named const bool `use_bar_magnifier` is
  accepted for v5/v6 historical fill wiring with host-owned lower-timeframe
  bars, chart-scoped public fill identity, and standard-OHLC fallback;
  ordinary-chart and magnifier host sequences share one gap point event at
  the next open when the previous host close differs from that open, so
  gapped-through price orders fill at the open rather than at the trigger
- `strategy.entry(id, strategy.long, qty=...)` in strategy-mode scripts only,
  filled through the supported historical broker model for long market entries
  up to the configured `pyramiding` limit
- `strategy.entry(id, strategy.short, qty=...)` in strategy-mode scripts only,
  filled as a next-bar-open market short while flat or already short, or as a
  reversal that first flattens an opposite long at the reverse fill price then
  opens the requested short quantity
- `strategy.entry(id, strategy.short, qty=..., limit=price)` in strategy-mode
  scripts only, filled at the limit price on a later historical bar when
  `high >= limit` or above the configured verified limit threshold while flat
  or already short, or as a reversal that first flattens an opposite long then
  opens the requested short quantity
- `strategy.entry(id, strategy.short, qty=..., stop=price)` in strategy-mode
  scripts only, filled at the stop price on a later historical bar when
  `low <= stop` while flat or already short, or as a reversal that first
  flattens an opposite long then opens the requested short quantity
- `strategy.entry(id, strategy.short, qty=..., stop=price, limit=price)` in
  strategy-mode scripts only, activated when the selected historical path
  visits a price at or below the stop; short and low-first stop-limit fills
  stay fail-closed until a later historical bar when the path visits a price
  at or above the limit or above the configured verified limit threshold while
  flat or already short, or as a reversal that first flattens an opposite long
  then opens the requested short quantity
- price-based long `strategy.entry` limit, stop, and stop-limit reversals in
  strategy-mode scripts that first flatten an opposite short then open the
  requested long quantity; pyramiding applies to the new side, not the flatten
  quantity
- `strategy.risk.allow_entry_in(strategy.direction.all|long|short)` in
  strategy-mode scripts only; allowed `strategy.entry` directions keep current
  open, add, and reversal behavior; a disallowed opposite `strategy.entry`
  against an open allowed position flattens without opening prohibited
  exposure; a disallowed opposite `strategy.entry` while flat is a no-op;
  last call wins; pending opposite `strategy.entry` intents are cancelled
  while flat or converted to market close-only against an open allowed
  position; `strategy.order` is not bound by this rule
- `strategy.risk.max_position_size(contracts)` in strategy-mode scripts only
  with simple positive finite numeric contracts; `strategy.entry` quantity is
  reduced so post-fill exposure does not exceed the limit; an entry that cannot
  fit a positive quantity is a no-op; reversal flattens then opens at most the
  limit on the new side; pyramiding may add until the size limit;
  `strategy.order` is not bound by this rule
- `strategy.risk.max_drawdown(value, type)` in strategy-mode scripts only with
  simple positive finite numeric value and `strategy.cash` or
  `strategy.percent_of_equity`; when the peak-equity drawdown including open
  adverse excursion reaches the threshold, the broker cancels pending orders,
  flattens through a risk-owned market close, and permanently blocks later
  `strategy.entry` and `strategy.order` actions
- host-neutral intraday risk windows keyed by UTC day from bar time when the
  chart timeframe is at or below 1D, and by bar time when the timeframe is
  higher than 1D; missing bars start a new window; non-positive timeframes fail
  closed to the UTC-day key; optional host session window input may replace
  those UTC keys with per-bar `windowId` and `tradingDayId`; missing input
  keeps the UTC subset; this runtime does not maintain an exchange calendar
- `strategy.risk.max_intraday_loss(value, type)` in strategy-mode scripts only
  with simple positive finite numeric value and `strategy.cash` or
  `strategy.percent_of_equity`; loss is measured from maximum window equity
  including open adverse excursion; on trigger the broker cancels pending
  orders, flattens, and blocks later trades until the next intraday window
- `strategy.risk.max_intraday_filled_orders(count)` in strategy-mode scripts
  only with a simple positive finite integer count; each public filled order
  in the current window increments the count; on reaching the limit the broker
  cancels pending orders, flattens, and blocks later trades until the next
  window
- `strategy.risk.max_cons_loss_days(count)` in strategy-mode scripts only with
  a simple positive finite integer count; each completed window with negative
  realized closed-trade profit counts as a loss day; a profitable or no-trade
  window resets the streak; after `count` consecutive observed loss windows
  the broker cancels pending orders, flattens, and permanently blocks later
  trades
- `strategy.entry(id, strategy.long)` in strategy-mode scripts only when the
  declaration configures the supported fixed default quantity subset; explicit
  `qty` continues to override the declaration default
- same-id `strategy.order` replacement of pending market, limit, stop, and
  stop-limit generic orders in the same direction; opposite-direction same-id
  replacement cancels the old intent then places the new one;
  `strategy.cancel(id)` clears matching pending generic orders, pending
  exits, deferred relative exits, and pending closes, including when those
  families share a public id; generic-order reductions allocate FIFO, or id-specific ANY when
  `close_entries_rule` is ANY and the order id matches an open entry;
  const/simple `oca_name` with explicit `strategy.oca.none` keeps grouped
  `strategy.entry` and `strategy.order` intents independent; `strategy.oca.cancel`
  cancels still-pending same-group entry and generic-order peers after a fill;
  `strategy.oca.reduce` reduces same-group peer remaining quantity by the
  filled quantity and removes peers reduced to zero, including mixed
  entry/order/exit members that share the same name and reduce type; const/simple
  `strategy.exit` `oca_name` maps onto that implicit reduce reservation model
- `strategy.close(id)`, `strategy.close(id, qty=...)`, and
  `strategy.close(id, qty_percent=...)` in strategy-mode scripts only, closing
  all or part of the matching long or short position at the next historical bar
  open, or at the current bar close when const/simple `immediately=true`, and
  recording closed trades; short closes record signed quantity and cover PnL;
  fixed `qty` wins over `qty_percent`; series or non-bool `immediately` remains
  unsupported
- strategy equity snapshots with per-bar `cash`, `marketValue`, `equity`, and
  `netProfit` for the supported long-only subset
- `strategy.position_size` and `strategy.position_avg_price` in strategy-mode
  historical scripts only, as read-only series floats that update after
  supported entry fills and after next-bar close fills; average price is `na`
  when flat
- `strategy.openprofit`, `strategy.netprofit`, and `strategy.equity` in
  strategy-mode historical scripts only, as read-only series floats for the
  long-only broker subset; open profit uses current close mark-to-market,
  net profit is realized closed-trade profit only, and equity is initial
  capital plus realized and open profit
- the closed Phase L strategy usability boundary is summarized in
  `docs/PHASE_L_AUDIT.md`
- fixture-backed strategy state variable interactions in already-supported
  expression contexts: `if`, `switch`, `for`, `while`, pure UDF arguments, and
  constant history references
- `input.*`
- `plot`, `plotchar`, `plotshape`, `plotarrow`, `plotbar`, `plotcandle`,
  `bgcolor`, `barcolor`, `hline`, and `fill`
- `alertcondition(condition, title, message)` with bool-compatible conditions
  and const-string title/message values, including `{{open}}`, `{{high}}`,
  `{{low}}`, `{{close}}`, `{{volume}}`, `{{ticker}}`, `{{interval}}`, and
  `{{exchange}}` placeholders plus UTC-formatted triggering-bar `{{time}}` in
  the message only
- `alert(message, freq?)` with string-compatible dynamic messages and a
  const-string frequency subset limited to `alert.freq_once_per_bar`,
  `alert.freq_all`, and `alert.freq_once_per_bar_close`; TradingView-style
  `{{...}}` placeholder interpolation remains unsupported outside the
  supported `alertcondition` message subset
- `na`, `nz`
- common `ta.*` helpers listed in
  [`BUILTIN_SIGNATURES.md`](BUILTIN_SIGNATURES.md), including moving averages,
  rolling statistics, momentum/history helpers, crosses, extremes, trend
  checks, value lookups, true range, volume flow helpers, and the fixed-metadata
  UTC-daily/explicit-anchor VWAP contract
- partial float, int, bool, string, color, label-id, line-id, linefill-id, box-id, and table-id arrays with `array.new_float`,
  `array.new_int`, `array.new_bool`, `array.new_string`, `array.new_color`,
  `array.new_label`, `array.new_line`, `array.new_linefill`, `array.new_box`,
  `array.new_table`, official `array.new<type>` syntax for those scalar and
  drawing-object element types, `array.new<chart.point>`,
  `array.from`, `array.push`, `array.get`, `array.set`, `array.size`, `array.pop`,
  `array.insert` with integer-compatible indexes including `series int`,
  `array.remove`, `array.shift`, `array.unshift`,
  `array.fill`, `array.first`, `array.last`, `array.copy`, `array.slice`,
  `array.concat`, `array.includes`, `array.every`, `array.some`,
  `array.indexof`, `array.lastindexof`, numeric `array.binary_search*`,
  `array.clear`, numeric `array.abs`, `array.min`, `array.max`, `array.sum`,
  `array.avg`, `array.range`, `array.median`, `array.mode`,
  `array.percentile_nearest_rank`, `array.percentile_linear_interpolation`,
  `array.percentrank`, `array.covariance`, `array.standardize`,
  `array.variance`, `array.stdev`, numeric/string `array.sort` and
  `array.sort_indices`, same-local/same-imported scalar-tree UDT
  `array.sort`/`array.sort_indices` by default field zero or a const int/string
  root-field selector, `array.reverse`, scalar-array
  `array.join`, and
  equivalent method-call syntax such as
  `values.push(close)` and `values.get(0)`

For v6 scripts, `and` and `or` use lazy evaluation: the right operand is skipped
when the left operand already determines the result. Earlier-version scripts
keep the pre-v6 strict operand evaluation used by this runtime's legacy subset.

Stateful calls inside `if` blocks advance their callsite state only when the
branch executes. Series values not evaluated on a bar are committed as `na` to
keep history buffers bar-aligned.

Stateful calls inside `switch` arms follow the same conditional callsite rule:
only the selected arm result executes. For supported block arms, only the
selected block's statements and final result expression execute.
Selector-form switches evaluate the selector once per bar, compare cases in
source order, and return `na` when no arm matches and no default arm is present.

Normal and tuple block-local declarations inside `if` blocks are scoped to the
branch and do not leak outside it. A tuple declaration in a local scope shadows
outer variables with the same names; use reassignment syntax for scalar updates
to existing variables. `var` declarations in local blocks initialize the first
time their declaration site is reached, then preserve state across later
executions.
A value-producing `if` local block can end with a complete nested
`if`/`else-if`/`else` statement. The selected nested leaf becomes the enclosing
branch result, with analysis, type inference, and lowering applied recursively.
Every reachable leaf must still end in a value-producing expression; a final
reassignment or other non-value statement is rejected with `E_BRANCH_RETURN`.
Switch statement-block arm declarations follow the same branch-local no-leak
rule, and supported block arms may reassign already-visible outer variables.
Expression-context block arms still need a final result expression.
Statement-context switch block arms can omit that result expression. When a
selected statement-block arm executes inside a loop body, `break` and
`continue` propagate to the nearest enclosing loop. The `switch` expression does
not consume loop-control statements as its own control flow.
Tuple declaration/destructuring from selected statement-block arms is supported
when each arm's final expression has compatible tuple arity and element types.
Same-local UDT constructor results and block-local UDT aliases are supported
when all selected arm result identities resolve to the same local UDT. Mismatched
UDT identities across arms remain rejected. Same-imported-identity UDT
constructor or block-local alias results are supported for expression
statement-block arms; local/imported identity mismatches remain rejected by
semantic fixture.

`for` loops support inclusive integer ranges with an optional explicit `by`
step. The runtime increments when `from <= to` and decrements when `from > to`;
the absolute step magnitude is used, so signed step values do not override the
range direction. The `from` bound and `by` step are evaluated once when the loop
is reached. In v6 scripts, the `to` bound is re-evaluated before each iteration;
earlier-version scripts evaluate the `to` bound once. If a runtime range bound
or step evaluates to `na`, the loop body is skipped and a loop expression
returns `na`. The loop counter is scoped to the loop body. Step values must be
non-zero ints.
`break` exits the nearest enclosing loop and `continue` skips to its next
iteration. `break` or `continue` outside a loop is rejected with
`E_LOOP_CONTROL`.
`x = for ...` and tuple assignment from `for` results are supported when the
loop body ends with an expression. The assigned value is the latest iteration
result that reached that expression, or `na` if no iteration reaches it.

`while` loops support statement form and a scalar expression subset. Conditions
must type-check as bool; a runtime `na` condition exits the loop like false.
`break` and `continue` target the nearest enclosing loop. A deterministic
iteration guard prevents runaway loops. Fixture-backed `while` bodies include
history reads and pure UDF calls. `while` expressions return the latest reached
final body expression, or `na` if no iteration produces a value, with
fixture-backed stateful callsite advancement, body-local declarations, and
local `var` declaration-site persistence. When evaluated inside an outer loop,
`break` and `continue` in the expression body are contained by the nearest
`while` expression. Tuple declaration/destructuring, same-local UDT constructor
or block-local alias results, and scalar-array results with caller-side reads
and mutation from while expressions are fixture-backed for fresh arrays and
existing-array alias returns, including zero-iteration `na`, break/continue
result preservation, and fresh historical copies from committed history reads;
imported UDT identity plus nested collection variants remain outside the current
subset, with nested-array while-expression results and imported UDT constructor
results rejected by semantic fixtures.

`switch` support is expression-only in condition and selector forms. Expression
arms are supported in both forms. Statement-block arms are supported when the
block ends with a result expression. Block arms without a final expression
remain rejected.

User-defined functions support single-expression and multi-statement block
bodies:

```pine
smooth(src, len) => ta.sma(src, len)
spread(hi, lo) => hi - lo
plot(spread(lo=low, hi=high))

range2(hi, lo) =>
    value = hi - lo
    value * 2
```

Named arguments are supported for user-defined functions. A block body's final
statement determines its result: an expression, local declaration, local
reassignment, conditional, or loop can supply the return. A final conditional
without `else` yields `na` when its condition is false; final loops may produce
`void` when their body is side-effect-only. Pine v4 additionally admits the
exact namespace-call subset `array.set/pop/unshift/clear`,
`label.new/delete`, and `line.new/delete` inside UDF bodies. Recursive
functions, all other collection/drawing side effects, output/alert side
effects, global reassignment inside functions, and side-effecting calls as UDF
arguments are rejected in the current executable subset. UDF arguments are
evaluated once into callsite-local temporaries.

## Initial Built-Ins

This is the first broad target set, not the first executable milestone. The
minimal executable Phase 1 built-ins are defined in
[`BUILTIN_SIGNATURES.md`](BUILTIN_SIGNATURES.md).

Global values:

- `open`
- `high`
- `low`
- `close`
- `volume`
- `time`
- `hl2`
- `hlc3`
- `ohlc4`
- `bar_index`
- `na`

Input namespace:

- `input`
- `input.int`
- `input.float`
- `input.bool`
- `input.source`
- `input.color`
- `input.string`
- `input.price`
- `input.time`
- `input.symbol`
- `input.timeframe`
- `input.session`
- `input.text_area`

TA namespace:

- common indicator helpers listed in
  [`BUILTIN_SIGNATURES.md`](BUILTIN_SIGNATURES.md)

Plotting:

- `alertcondition`
- `alert`
- `plot`
- `plotchar`
- `plotshape`
- `plotarrow`
- `plotbar`
- `plotcandle`
- `hline`
- `fill`
- `bgcolor`
- `barcolor`

For Pine v4/v5 only, the `offset` argument of `plot`, `plotchar`, `plotshape`,
`plotarrow`, `bgcolor`, and `barcolor` may be `series int`. The runtime retains
the last evaluated offset as metadata for the complete output, matching the
pre-v6 behavior. Pine v3 and v6 require a simple-or-weaker integer. This visual
offset rule is independent from the `expr[offset]` history operator.

`plot` `style` accepts a string-compatible expression when the analyzer can
prove every runtime value is a documented `plot.style_*` enum. Unbounded
strings, mixed invalid branches, and series integer ordinals stay rejected.
The emitted plot metadata keeps one style field and uses the last evaluated
value for the complete output. `plotshape` `style` and `hline` `linestyle`
use the same proven-domain rule; plotshape emits per-bar styles.

Color namespace:

- common named colors
- `color.new`
- `color.rgb`
- color component helpers
- `color.from_gradient`
- hex color parsing

Utility:

- `na`
- `nz`
- selected `math.*`, `str.*`, and UTC time helpers listed in
  [`BUILTIN_SIGNATURES.md`](BUILTIN_SIGNATURES.md)

Request data:

- `request.security(syminfo.tickerid, timeframe.period, expression)` for the
  current chart context only. The requested expression must be side-effect-free;
  scalar expressions, tuple literals made from side-effect-free elements, and
  selected tuple-returning calls destructured directly from the request are
  supported. The selected tuple-returning calls currently include `ta.macd`,
  `ta.bb`, `ta.kc`, `ta.supertrend`, `ta.dmi`, and
  `ta.vwap(source, anchor, stdev_mult)`.
- `request.security("SYMBOL", timeframe, expression)` and
  `request.security(syminfo.tickerid, timeframe, expression)` for host-provided
  same-or-higher-timeframe bars. The provider expression subset includes
  requested-context `syminfo.tickerid`/`timeframe.period`, direct OHLCV/time
  sources, pure arithmetic and ternaries, history references, `na`,
  `nz`, `time(timeframe)` and `time_close(timeframe)` function calls including
  documented named arguments, `barstate.isfirst`/`islast`/`islastconfirmedhistory`/`isnew`/`isconfirmed`/`ishistory`/`isrealtime`, selected stateless `math.*` calls, fixed-mintick
  `math.round_to_mintick`, `math.sum`, `ta.cum`, `ta.sma`, `ta.ema`,
  `ta.dema`, `ta.tema`, `ta.rma`, `ta.rsi`, `ta.tsi`, `ta.cmo`, `ta.cci`,
  `ta.cog`, `ta.bop`, `ta.ao`, `ta.accdist`, `ta.iii`, `ta.nvi`, `ta.obv`, `ta.pvi`, `ta.pvt`, `ta.wvad`, `ta.max`, `ta.min`, `ta.mfi`,
  `ta.stoch`, `ta.wpr`, `ta.sar`,
  `ta.tr` function calls, `ta.atr`, `ta.highest`, `ta.lowest`,
  `ta.highestbars`, `ta.lowestbars`, `ta.change`, `ta.mom`, `ta.roc`, `ta.range`, `ta.dev`, `ta.vwap`, `ta.rising`,
  `ta.bbw`, `ta.kcw`, `ta.pivothigh`, `ta.pivotlow`, `ta.correlation`,
  `ta.covariance`, `ta.median`, `ta.mode`, `ta.percentile_nearest_rank`,
  `ta.percentile_linear_interpolation`,
  `ta.percentrank`, `ta.stdev`, `ta.variance`, `ta.wma`, `ta.vwma`,
  `ta.swma`, `ta.hma`, `ta.alma`, `ta.linreg`, `ta.falling`, `ta.barssince`,
  `ta.valuewhen`, `ta.cross`, `ta.crossover`, and `ta.crossunder`; local variable aliases, the
  `ta.tr` variable form, named arguments on other requested calls, session
  boundary flags, and stateful math
  calls such as `math.random` are not part of this subset.
  Requested-context rolling callsite state is isolated from chart state.
  Provider-backed tuple literals whose elements are in the supported scalar
  subset are supported when destructured directly from the request. Selected
  provider-backed tuple-returning calls are also supported, currently
  `ta.macd`, `ta.bb`, `ta.kc`, `ta.supertrend`, `ta.dmi`, and
  `ta.vwap(source, anchor, stdev_mult)`. Other provider-backed tuple
  expressions remain outside this subset.
  Higher-timeframe alignment uses default `gaps_off` and `lookahead_off`: only
  confirmed requested bars are visible, and missing requested bars forward-fill
  the last confirmed value.
  CLI hosts pass these bars with
  `--request-bars SYMBOL:TIMEFRAME=bars.csv`; Python hosts pass
  `request_bars={"SYMBOL:TIMEFRAME": bars}`; WASM hosts pass the same mapping as
  request-bars JSON. Chart identity is explicit through CLI
  `--chart-symbol`/`--chart-timeframe`, Python `chart_symbol`/`chart_timeframe`
  keywords, or the WASM request JSON `$chart` object.
- Pine v1-v4 `security(symbol, resolution, expression, ...)` for the same
  requested-expression subset. The historical binder accepts the v1/v2
  three-or-four-argument form and the v3/v4 three-to-five-argument positional
  or named form. Compile-time bools or matching `barmerge` constants select
  gaps/lookahead; v1/v2 default to historical `lookahead_on` and v3/v4 default
  to `lookahead_off`. `gaps_on` returns values only at the selected requested
  open/confirmation boundary, `gaps_off` carries the latest eligible value,
  and realtime updates use confirmed alignment. A reached lookahead-on call
  reports one non-error repaint warning per callsite. Missing streams retain
  the original complete legacy call span. In Pine v4, a request placed directly
  in a UDF body may depend on scalar parameters and normal immutable scalar
  locals. Series dependency initializers recompute in the isolated requested
  runtime; const/input/simple dependencies are captured. A prior legacy
  request may feed a later request only through the exact three-positional-
  argument form with identical symbol, timeframe, gaps, and lookahead.
  Different selectors, control-flow-local requests, reassignment, persistence,
  recursion, and side effects remain unsupported, and the modern
  `request.security` local-alias boundary is unchanged.

## Explicitly Unsupported in Phase 1

The analyzer should reject these with clear diagnostics:

- strategy order functions and reporting helpers outside the narrow
  `strategy.entry`, `strategy.order`, `strategy.close`, `strategy.close_all`,
  `strategy.cancel`, `strategy.cancel_all`, and `strategy.exit` subsets,
  including series `oca_name` on `strategy.entry`, `strategy.order`, and
  `strategy.exit`,
  same-side or 3+ trigger exits, invalid trailing combinations, partial
  `strategy.close_all()`, pyramiding behavior beyond the fixture-backed
  long-only subset, broker settings beyond the supported declaration subset,
  and sizing modes beyond the fixture-backed fixed, cash, and percent-of-equity
  default entry subset,
  `strategy.*` variables beyond the supported position/profit/equity/count
  state subset, mutable strategy state, and requested-context strategy state,
  `fill_orders_on_standard_ohlc`, and remaining `strategy.risk.*` broker
  directives other than fixture-backed `strategy.risk.allow_entry_in`,
  `strategy.risk.max_position_size`, `strategy.risk.max_drawdown`,
  `strategy.risk.max_intraday_loss`,
  `strategy.risk.max_intraday_filled_orders`, and
  `strategy.risk.max_cons_loss_days`
- `request.*` variants outside the narrow same-context and same-or-higher-timeframe
  provider-backed `request.security` subsets
- legacy lower-timeframe `security`, requested expressions outside the same
  provider-backed subset, and non-empty or dynamic declaration-level
  `study(resolution=...)`; the exact empty-string chart-inherited subset is
  supported, while all execution-timeframe-changing forms remain a precise
  unsupported program-context feature
- `request.security_lower_tf`; lower-timeframe array-returning request APIs need
  typed array return semantics and host output shapes before support is claimed
- unsupported alert frequency values outside the claimed const-string
  frequency subset and alert placeholder interpolation outside the
  supported `alertcondition` message subset
- `library` and root `export` declarations; `import` is partial for
  host-provided exact-key aliases that expose exported const expressions, pure
  exported functions, and the fixture-backed same-imported-identity scalar-tree
  UDT value, history, array, `varip`, and pure-method subsets, while unaliased
  imports, missing host sources, re-exports, broader non-scalar imported values,
  and side-effecting exported functions remain rejected
- unsupported collection families and element types beyond the fixture-backed
  scalar/object/chart-point/scalar-tree-UDT arrays, scalar maps, and
  float/int/bool/string/color matrices; nested collections, non-scalar map
  templates, bare generic collection declarations, and recursive/non-scalar UDT
  arrays remain rejected
- user-defined type forms outside the fixture-backed local and imported
  scalar-tree subsets; construction, typed declarations, selected control-flow
  results, `var`, scalar-tree `varip`, value history, same-identity arrays, and
  root-field replacement are supported where recorded in conformance, while
  recursive, object-backed, nested-collection, and broader mutation shapes
  remain rejected
- user-defined method forms outside the fixture-backed pure local and imported
  scalar-tree receiver/parameter/return subsets; side effects, recursion,
  unknown receivers, mismatched UDT identity, and unsupported parameter families
  remain rejected
- method calls outside the fixture-backed array, map, matrix, drawing-object,
  chart-point, and local/imported UDT method subsets
- drawing-object behavior outside the fixture-backed label, line, linefill,
  box, table, polyline, and chart-point subsets
- general multi-symbol or multi-timeframe data loading outside the documented
  `request.security` provider subset
- broker emulation and order execution outside the fixture-backed long-only
  subset plus market `strategy.short` entries, short closes, and market,
  limit, stop, and stop-limit `strategy.entry` reversals
- `varip` drawing ids, tuple `varip`, and value families outside the
  fixture-backed scalar, chart-point, scalar-array, scalar-map, scalar-matrix,
  and scalar-tree UDT/UDT-array subsets

Longer-term work for these unsupported areas is tracked in
[`LONG_TERM_EXECUTION_PLAN.md`](LONG_TERM_EXECUTION_PLAN.md).

## Compatibility Report

The analyzer returns the public analysis `schemaVersion: 5` contract from CLI
JSON, Python, and WASM. The top-level dialect fields describe validation and
mode classification; compatibility keeps canonical feature evidence separate
from future legacy translations and result-affecting emulations:

```json
{
  "schemaVersion": 5,
  "languageVersion": 5,
  "languageVersionOrigin": "explicit",
  "dialect": "v5",
  "scriptMode": "indicator",
  "executable": true,
  "diagnostics": [],
  "inputs": [],
  "compatibility": {
    "supported": [
      {"feature": "ta.ema", "span": "..."}
    ],
    "unsupported": [],
    "legacyTranslations": [],
    "legacyEmulations": []
  }
}
```

The user experience should be "this part is unsupported" rather than "the
script crashed."

Legacy versions are authoritative on every host: no option is required to
enable a valid v1-v4 indicator, and explicit v5/v6 source is never forced into
a legacy profile. CLI owns representative complete analysis goldens for
implicit v1 and explicit v2-v4 plus a focused v2 failure; Python and WASM
compare their full reports with those same `schemaVersion: 5` values. Input
records also expose compile-time `default`, `min`, `max`, `step`, and `options`
metadata when those values are present in the supported input signature. The
optional `legacyPolicy` rejection switch and source migration preview are not
part of the current API or compatibility claims.
