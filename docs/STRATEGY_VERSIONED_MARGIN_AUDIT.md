# Source-version margin defaults

Status: scoped Windows development-artifact qualification passed, 2026-09-10.
Base: `906e04592`.

The [official v6 migration rule](https://www.tradingview.com/pine-script-docs/migration-guides/to-pine-version-6/#default-margin-percentage)
changes omitted long and short margins from 0 to 100. Explicit zero restores
the v5 account behavior. Three separately frozen native controls use identical
long/short oversize orders and explicit starting capital, varying only version
or explicit zero margins. Native v5 and v6-zero produce two trades; default v6
rejects both entries. Raw exports are in
`.local/tv-goal-20260910/versioned-margin/`.

Before repair, v5 passes 75051 values and v6-zero passes 75075. Default v6
fails 50030/75045 values, with no dropped bars or relaxed 1e-9 tolerances.

The analyzer resolves omitted margins in strategy HIR. The direct Rust runtime
also resolves them for manually constructed HIR. The public `explicit` flag
continues to describe whether the property appeared in source; it no longer
disables a resolved default. Explicit zero and explicit nonzero settings remain
authoritative. Indicator HIR is not changed.

The new version/override regression failed before the repair. It now covers
both long and short directions, v5/v6 defaults, explicit zero/nonzero overrides,
and direct Rust HIR construction. Rust workspace verification passes 6674
tests with no runtime snapshot changes. The complete Windows gate also passes
715 installed-wheel Python tests, 130 tool tests and actual WASM. A retained
wheel in a fresh environment passes all three native controls, 225171/225171
values. Complete CLI batch/incremental, Python common execution fields, and
generated WASM outputs agree. Linux/release-profile qualification of this slice
remains separate.

This slice does not claim broader order admission, currency conversion, contract
multipliers or private same-price sequencing. It does not resolve the original
live-tick price capture's missing-input uncertainty.
