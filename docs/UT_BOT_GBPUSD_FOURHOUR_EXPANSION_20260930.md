# UT Bot GBPUSD four-hour and unconfirmed opening expansion

Verified 2026-09-30 from four fresh authenticated Chrome/TradingView captures.
The complete unchanged 43-line Pine v4 [QuantNomad UT Bot Strategy](https://www.tradingview.com/script/VKMgZdYr-UT-Bot-Strategy/)
has source SHA256 `0e6ea37251e0a35b45de238427c6a9692a64b62abc502198aad6dad7a514d0c1`.
This adds GBPUSD coverage and native evidence for a forming-bar opening
reversal, while keeping full live Tick parity unqualified.

## Native settings and history

Actual feed: FX:GBPUSD, 240; USD, price grid 1/100000, integer quantity,
point value 1. Each native applied Inputs/Properties is captured: initial
capital 1,000,000 USD, fixed quantity 1, pyramiding 1, zero commission and
slippage, infinite leverage, on-bar-close calculation, one-tick order delay.
The earlier visible 100,000 capital is retained separately. Source is not
rewritten to inject properties.

Each raw native chart export has 21,353 rows. All four closed time/OHLCV
histories match exactly: 21,352 bars from 2013-01-02 02:00 UTC through
2026-09-30 05:00 UTC. No initial prefix is removed. The last native row starts
at 09:00 UTC and remains forming at capture. Report times use UTC+8 to the
minute and map uniquely to exported UNIX timestamps.

## Historical and opening comparisons

| Case | Closed before forming bar | Closed including opening reversal | Closed signal cells | New opening entry |
| --- | ---: | ---: | ---: | ---: |
| Default sensitivity 1, ATR10 | 2,496 | 2,497 | 42,704 | 1 |
| Sensitivity 2 | 1,060 | 1,061 | 42,704 | 1 |
| ATR14 | 2,510 | 2,511 | 42,704 | 1 |
| Heikin Ashi | 1,772 | 1,773 | 42,704 | 1 |

The four native cases total **7,842 closed trades and 170,816 closed signal
cells**, over one shared history. All trades are included, including four
boundary closes. Each final open entry matches direction, timestamp,
quantity and price. Fill-price error is at most 2.23e-16 USD. Net PnL agrees
within each native CSV value's own displayed precision; the maximum absolute
error is 0.005 USD for values displayed to two decimals. Values with three,
four or five decimals are checked with their narrower half-unit tolerances.
This does not claim an exact native undisplayed monetary value.

Three historical execution modes produce twelve complete outputs,
byte-identical within each case. Each matches all closed signal bits and
the committed trades/position immediately before 09:00 UTC.

The native report already closes the prior short and opens a long at 17:00
UTC+8 on the forming bar. Omitting that transaction would lose evidence.
The existing `run-realtime-forming` CLI command ultimately confirms its last
bar; it cannot represent this unconfirmed snapshot directly.

An external Rust embedding consumer therefore loads the original source and
identical closed history into the public `RealtimeRuntime`, then supplies
one `BarUpdate::forming` observation with `opening_update=true`. Its time
and opening price come from the native chart export; the replay observation
sets OHLC to that opening price and volume to zero. It infers no later price
path and is never confirmed. All four native reversals match: prior short
closes and a new fixed-quantity long opens at **1.32750**. No extra signal
evaluation occurs on the opening observation. The committed result before
and after it is byte-identical to all three historical mode outputs.

Four additional complete forming snapshots are retained. This qualifies the
observed pending-market-order opening boundary using exported native opening
facts. It is not a recording or qualification of TradingView's live Tick
stream, intrabar extrema, subsequent fills, or forming mark-to-market PnL.

## Host boundary and frozen evidence

The HA case uses the unchanged original security expression and an external
21,352-row provider with key
`{"chart":"heikinashi","symbol":"FX:GBPUSD"}:240`.
HA close is OHLC/4; seed open OC/2; later open is the prior HA open/close
mean; high/low use envelopes. Every row is recomputed and checked. The
opening observation does not execute the strategy's HA expression again.
Concrete data preparation stays outside the interpreter core.

All evidence is retained in `.local/ut-bot-gbpusd-fourhour-20260930/`:
raw native downloads, source, settings/range/export records, screenshots,
closed bars/provider, opening replay CSVs, embedding consumer source and
Cargo lock, executable/build receipt, execution receipts and comparisons.
`capture-provenance.json` records the original filenames in `I:\sys\下载`.
`freeze_verify.py` reruns comparisons and validates all source/artifact pins.

The embedding consumer was rebuilt offline with registry versions/checksums
from the qualified workspace Cargo lock. Its initial independently resolved
build is retained as unqualified setup evidence; only the subsequent pinned
build/executable supports this receipt. The core patch and existing CLI are
unchanged. The preceding successful full fee gate and source pins are reused
and revalidated; no new full-gate run or core/golden update is claimed. That
gate passed 1,988 runtime, 242 CLI and 774 installed-wheel tests, plus WASM/host
checks. Earlier complete-script artifacts remain pinned through the chain.

During sampling the editor changed externally from the starting WMA draft
to a Stoch v5 probe. Both are recorded. The latest Stoch draft was preserved,
with a second clipboard read proving equality. Restoration of the earlier
draft was superseded. This round's UT strategy was removed, BTCUSD daily
restored and the sampling tab closed, with state/PNG evidence. Older stale
debugger-tab cleanup remains unverified.

Native colors/styles/account equity, forming PnL, fractional quantity and
general live Tick parity remain outside this receipt. Non-unit point value
is still explicitly unsupported. Earlier Hull displayed monetary residuals
and broader limit/stop/repeated-partial/reversal fee controls remain open.
The expansion goal stays active; next collect additional native order and
price-condition boundaries while continuing complete-script market coverage.
