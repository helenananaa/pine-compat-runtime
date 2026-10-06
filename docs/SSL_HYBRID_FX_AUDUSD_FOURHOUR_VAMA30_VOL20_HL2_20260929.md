# SSL Hybrid Strategy on AUDUSD VAMA 30 with volatility lookback 20

Captured independently through authenticated Chrome on 2026-09-29. This
extends the [VAMA 30 HL2 lookback 10 case and extrema repair](SSL_HYBRID_FX_AUDUSD_FOURHOUR_VAMA30_HL2_20260929.md)
by changing only the volatility lookback from 10 to **20**. The unchanged
public Pine v5 SSL Hybrid Strategy source, FXCM `FX:AUDUSD`, four-hour
period, HL2, USD account properties, and host-neutral metadata are identical
to that report. CLI overrides are `12=VAMA`, `13=30`, `18=hl2`, `22=20`.
The analyzer receipt confirms call site 22 is the volatility input.

## Frozen data and repair evidence

The native chart contains 21,345 rows starting 2013-01-02 02:00 UTC.
The forming 2026-09-29 01:00 UTC bar is excluded; input contains **21,344
confirmed bars** through 2026-09-28 21:00 UTC. Every native execution
precedes the excluded bar. These confirmed bars are byte-identical to
the lookback 10 and preceding LSMA/TEMA cases. Trade/chart files were
downloaded into `I:\sys\下载` at 12:20 and 12:23 UTC+8 respectively.

Before repair, baseline and SSL1 each lost 19 native valid warmup positions,
both baseline channels lost 18, and one Candle Size signal differed. The
native baseline starts at zero-based bar 29, without waiting for 20 finite
deviations. The same core repair as the lookback 10 case removes every
discrepancy. Both source versions, immutable CLIs, output and comparison
receipts remain in `before-fix/` and the evidence root. The complete strategy
output is unchanged by the warmup repair on this history.

## Results

- **149,248 nonblank positions** across eight named exported columns match
  at absolute tolerance `1e-8`, with missing positions checked. Per-column
  counts equal the lookback 10 report: baseline/SSL1 have 21,315, channels
  21,314 each, MA UP/DOWN 21,323 each, Candle Size 21,344, optional MTF zero.
- **2,964 closed trades** match entry IDs, directions, UTC+8 minute times,
  prices, integer quantities, entry values, durations, commission, and PnL.
- **Five open entries** match IDs, times, prices, and quantities.
- **1,914 explicit exit fills** match signal IDs, times, prices, and quantities.
- Commission/PnL display deltas are below $0.005; diagnostics are empty.

Batch, incremental, and realtime-history outputs are byte-identical,
SHA-256 `3203250b55f1c1453e1b999179fefd2fd2943d9a0f96c8d97cf883ea1ddd0c93`.
The same-bar lookback 10 control exactly reproduces its new native-qualified
output and **3,890** closed trades. Baseline values change at 18,775 of
21,315 mutually defined positions, SSL1 at 18,831/21,315, and each channel
at 18,870/21,314. The parameter change is observable on identical data.
Counts between settings overlap and are not independent sample totals.

## Verification and reproduction

This case uses the same verified rebuilt CLI, source patch, corrected
golden, and **fresh complete release gate** recorded in the lookback 10
report. The shared verifier checks current source and golden hashes,
native artifacts, terminal mode receipts, comparison, failing before-fix
evidence, exact control output, gate log, and browser restoration.

Evidence directory:
`.local/ssl-hybrid-fx-audusd-fourhour-vama30-vol20-hl2-20260929/`.

| Artifact | SHA-256 |
| --- | --- |
| Native chart | `657ae1f1c53a5050ed3b9885f5659318a3f1df823182ffe93deb2c1c12678938` |
| Native trades | `47713930c34063205fde187c0e8059b1576b6d09d4f3bfbaf0b0d7e6032eeab2` |
| Confirmed bars | `40b64a2dd0897ec2c47c206249cda3b9f34ca16bdfd533061285b72d4e34ad58` |

```powershell
python .local/ssl-hybrid-fx-audusd-fourhour-vama30-vol20-hl2-20260929/prepare_bars.py
python .local/ssl-hybrid-fx-audusd-fourhour-vama30-vol20-hl2-20260929/run_modes.py
python .local/ssl-hybrid-fx-audusd-fourhour-vama30-vol20-hl2-20260929/compare.py
python .local/ssl-hybrid-fx-audusd-fourhour-vama30-vol20-hl2-20260929/run_control.py
python .local/verify_vama_cases_20260929.py
```

Chrome was restored to Coinbase BTCUSD daily, HMA 60, close, volatility
lookback 10. This is evidence for this named confirmed-history setting and
selected fields. Forming-tick unrealized PnL, live tick parity, currency
conversion, and arbitrary scripts remain outside the established scope.
