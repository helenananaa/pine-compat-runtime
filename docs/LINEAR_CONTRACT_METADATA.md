# Linear contract metadata

Development contract, 2026-10-22. The contract capability is
`CONTRACT_METADATA_SCHEMA_VERSION = 1`; host requirements use schema 4.
Complete local candidate gates passed. Independent reference and release
qualification remain open. This candidate does not implement inverse accounting.

`ChartContext::with_point_value`, CLI `--chart-point-value`, and Python/WASM
`$chart.pointValue` accept a finite positive linear multiplier M. Omission keeps
the synthetic default 1. Requested-symbol metadata can describe another M, but
does not change the chart's native account. Quote prices and contract quantities
retain their original units.

Native monetary valuation uses quantity * price * M * quote-to-account FX.
The raw currency rate remains an FX rate: `strategy.convert_to_account` and
`strategy.convert_to_symbol` do not multiply their argument by M. Percent
commissions include contract value; cash-per-contract and cash-per-order fees
remain account-currency amounts. Cash/equity default sizing divides the account
budget by price * M * FX. Equity, realized/open profit, trade excursions and
percent denominators, margin/liquidation, partial closes, reversals and cash
rebasing use the same valuation convention.

The chart and requested-symbol contracts add decimal-string `quantityStep` and
`minContract`. Rust uses `with_quantity_step`/`with_min_contract`; CLI exposes
`--chart-quantity-step`/`--chart-min-contract`. Minimum quantity and quantity
increment are separate properties. `syminfo.mincontract` uses the supplied
minimum, then the supplied step, then the legacy precision fallback. The host
should supply its actual minimum when it is known, rather than assume it equals
the increment. Exchange maximum quantity and minimum notional remain host policy.

The bounded quantity representation admits at most 18 fractional places and a
u64 coefficient, with a finite positive value preserved by the float's shortest
decimal representation. Decimal/scientific strings such as `.25`, `25e-2` and
`2e-12` are accepted. Division expressions, booleans, zero, nonfinite values and
lossy rules are rejected. Legacy `quantityPrecision` 0..9 remains supported; a
separately supplied step cannot require more places than that precision.

Rounding operates on shortest decimal digits with exact remainder/subtraction,
not floating division followed by multiplication. Explicit quantities truncate
toward zero; default sizing floors to the grid. Quantities below the supplied
minimum become zero. Native risk cuts, close quantities and exit reservations
use the configured rules; a full reduction preserves the remaining position
instead of leaving rounding dust. An unrepresentable off-grid final float fails
closed rather than emitting an apparently aligned quantity.

Controlled evidence lives in `.local/goal-contract-spec-20261021/`. Independent
Python Decimal generation supplies 1,026 grid oracles. Rust contract tests cover
fees/default sizing/risk/margin, shorts/partials/reversal, FX rebasing, extrema
and isolation of requested-symbol metadata. A fresh installed-wheel Python suite
passed 1,916 cases after a reviewed nine-field requirements-shape update; the
initial stale requirements failure is retained separately. Candidate 02 also
passed all 7,960 workspace Rust tests, 172 tooling tests, formatting, strict
all-target Clippy, structure, host parity and generated WASM/Node checks.
The first Node attempt exposed a smoke-script syntax error; the corrected
script passed the generated-binding rerun. Runtime source and fixtures stayed
unchanged across those gates. `candidate-02/receipt.json` records that delta
and the exact installed wheel, SHA-256
`595c72a99cc452fc699525dfa0d9c4bae7096d49c4dd14e2f77b9adbcb13a87a`.

These controlled tests do not establish TradingView broker-emulator equivalence,
an exchange-specific margin model or a released package. Host scheduling,
effective historical rules and data acquisition remain outside this core.
