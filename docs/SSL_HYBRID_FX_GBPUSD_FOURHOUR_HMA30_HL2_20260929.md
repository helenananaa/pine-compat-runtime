# SSL Hybrid Strategy on GBPUSD four-hour HMA 30 HL2 history

Captured 2026-09-29 from the user's authenticated Chrome TradingView session.
This uses the unchanged public Pine v5 [SSL Hybrid Strategy by
kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/),
source SHA-256 `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
The chart identifies FXCM `FX:GBPUSD`, four hours. Inputs are HMA baseline
length 30 and `(H+L)/2` (`hl2`), retaining the remaining original inputs.
Properties are USD 5,000 initial capital, 10% equity sizing, pyramiding 10,
0.04% commission, zero slippage, default four-tick historical detail, and
one-tick order delay. The strategy's original trading date range starts
2021-08-01 and ends 2030-10-01.

## Frozen data and comparison

The native CSV has 21,345 bars from 2013-01-02 02:00 UTC. The forming
2026-09-29 01:00 UTC bar is excluded; local input contains **21,344 confirmed
bars**, through 2026-09-28 21:00 UTC. No report entry or closed trade occurs
on the excluded bar. Open-entry unrealized PnL is outside the comparison.
The host supplies symbol `FX:GBPUSD`, timeframe `240`, price grid
`1/100000`, and integer quantity precision through the host-neutral CLI.

Compared with the preceding HMA 20 capture, this export contains 37 revised
numeric OHLCV fields: 18 volume, eight high, nine low, one close, and one
open. Three additional differences are floating-point representations.
`bars-receipt.json` retains every difference. Each native comparison uses
the bars exported with that setting.

All **149,234 nonblank positions** in eight exported indicator columns match
at absolute tolerance `1e-8`, including missing positions. The comparator
accounts for the native CSV's false/NA Candle Size values represented as zero.
All **3,721 closed trades** match IDs, directions, UTC+8 report entry/exit
times, displayed prices, integer quantities, entry values, durations,
commission, and net PnL. All **five open entries** match ID, time, price,
and quantity. All **2,193 explicit exit-order fills** match signal, time,
five-decimal price, and quantity. Maximum net-PnL and commission display
deltas are $0.004999080 and $0.004998052, within half-cent display precision.
There are no diagnostics.

Batch, incremental, and realtime-history execution are byte-identical,
SHA-256 `d96e3469b2ff5569aee3b283293fe0b0d82bdc20221f18b891a7d812779e3801`.
On these exact frozen bars, HMA 20 yields **4,731** closed trades versus
HMA 30's **3,721**. Baseline, SSL1, and both baseline channels differ at all
21,311 mutually defined positions. This establishes a meaningful parameter
change; the HMA 20 control on these bars is a local sensitivity check.

## Runtime defect found and repaired

Before repair, native trade 3,223 took profit at 1.34946 on January 13, 2026,
18:00 UTC+8. The runtime missed that touch and exited at breakeven on the
next bar. The native bar high arrived as `1.3494599999999999`, one binary
roundoff step below the canonical five-decimal order price. The missed
profit also changed later equity-based entry quantities.

Fixed exit candidate detection now canonicalizes a chart path endpoint
only when it is within machine-roundoff distance of a configured price
tick. Candidate crossing marks remain inside the original host path.
Genuine sub-tick misses remain misses. The new long/short regression covers
both roundoff touches and genuine misses; it failed before the fix and
passes afterward. This is deterministic broker behavior with no host or
network dependency. All native trade and quantity differences disappear.

Verification on the rebuilt CLI:

- Broker tests: 525 passed.
- Runtime library tests: 1,980 passed.
- CLI tests: 242 passed.
- Prior frozen GBPUSD HMA 20 and EURUSD HMA 20/30 outputs remain byte-identical.
- Current HMA 30 strict native comparison passes in all three historical modes.

The source base is `2f9a83a0c333092167c7253571f56ee82a7ec0d0` plus the local
broker patch. `current-verification.json` records the source diff hash,
artifact/output hashes, controls, regressions, and test commands.
This qualifies the named confirmed-history case. Forming-bar tick parity
and universal Pine compatibility remain unproven.

## Evidence and reproduction

Evidence is retained in the ignored directory
`.local/ssl-hybrid-fx-gbpusd-fourhour-hma30-hl2-20260929/`: unchanged source,
native exports, input/properties snapshots, exact bars, comparator, before
and after outputs, and receipts. The settings screenshot was recaptured
at 10:32 UTC+8 to show the selected HMA 30 and HL2 values after the dropdown
closed; it is a settings record rather than the export-time price snapshot.
Native export files were downloaded into `I:\sys\下载` at 10:19–10:20 UTC+8.

Native chart SHA-256:
`1d9c5799dd11a7ac9e0efbe7883d4e8f6a8bd4a10ad7e0b6b6cd1ca82b0ca7d7`.
Native trades SHA-256:
`a8b927074f270d94136dc16d8761eda0960b95a3e99146636ca992c6a8e868d7`.
Frozen bars SHA-256:
`7e5fa599ad6f8444559e897357229b6521bb8ae68e521559c4c8fbfa1e5e23a4`.
The browser layout is restored to Coinbase BTCUSD daily, HMA 60, close.

Reproduce from the repository root:

```powershell
cargo build -p pine-cli
python .local/ssl-hybrid-fx-gbpusd-fourhour-hma30-hl2-20260929/prepare_bars.py
target/debug/pine-compat.exe run .local/ssl-hybrid-fx-gbpusd-fourhour-hma30-hl2-20260929/ssl-hybrid-original.pine --bars .local/ssl-hybrid-fx-gbpusd-fourhour-hma30-hl2-20260929/bars.csv --chart-symbol FX:GBPUSD --chart-timeframe 240 --chart-price-grid 1/100000 --chart-quantity-precision 0 --input-override 13=30 --input-override 18=hl2 > .local/ssl-hybrid-fx-gbpusd-fourhour-hma30-hl2-20260929/local-batch.json
python .local/ssl-hybrid-fx-gbpusd-fourhour-hma30-hl2-20260929/compare.py
```

Replace `run` with `run-incremental` or `run-realtime-history`, writing to
the corresponding local JSON file, to reproduce the mode checks. Set
override `13=20` on these same bars for the local parameter control.
