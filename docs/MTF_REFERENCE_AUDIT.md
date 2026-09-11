# Matched higher-timeframe reference

Revision: mtf-15m-60m-r1. Code baseline 5920add7e68a51ea307c8dcbc5fc8c42cace22e9;
captured and verified on 2026-09-09 after strategy revalidation. No runtime code
changes were needed for this batch.

## Scope and independent inputs

An original v6 indicator on standard OKX:BTCUSDT 15-minute candles requests a
60-minute tuple with default gaps_off/lookahead_off. It exports current hourly
OHLCV/time, the previous two hourly records, and SMA(close,3), plus chart bar_index.
Hourly inputs are reconstructed solely from those independently exported native
records, never from interpreter output. Repeated records for the same hour must
be identical; conflicts reject the package. The extra two historical records
provide the input needed to seed the first compared requested SMA.

The initial CSV download succeeded despite the browser event wait timing out,
but contained only 309 recent rows beginning at native index 20853. It is retained
as native-chart-raw.csv and is not the accepted capture. Loading chart history
back to February 1 produced native-chart-full-raw.csv with 21,162 rows starting
at index zero. The predeclared cutoff 1788922800 seconds retains 21,133 closed
chart bars and 5,286 hourly records, including three hours before the chart start.
No warmup rows are skipped. Raw captures, source, plan and normalized inputs have
hashes in capture-manifest.json under `.local/delivery-20260909/mtf-reference/`.

## Results and limits

All 20 outputs / 422,660 bar values match: chart index and the three time outputs
compare exactly; numeric values use the fixed absolute/relative 1e-9 tolerances.
The total includes 401,527 identity/raw-data alignment values and 21,133 derived
SMA values; it is not 422,660 independent indicator-formula tests.

The retained installed Windows wheel and actual generated WASM from the code
baseline produce the same complete JSON as the freshly rebuilt main-workspace
CLI. The initial Python verifier incorrectly supplied identity inside $chart;
the existing API rejected that input. The verifier now supplies Python identity
through chart_symbol/chart_timeframe and WASM identity through $chart, preserving
the existing contracts. No implementation or schema was weakened.

Evidence/reproduction files in the same local directory:

- plan.json, mtf-oracle.pine, native-compiled-dom.txt and both original CSVs;
- prepare_capture.py, chart-bars.csv, request-bars.csv, expected.json and
  capture-manifest.json;
- local-runtime.json and comparison.json;
- verify_hosts.py/.cjs, python-result.json, wasm-result.json,
  host-comparison.json and host-verification-v2.log.

This qualifies one historical default-HTF case and its Windows host surfaces.
It does not qualify gaps_on, lookahead_on, lower-timeframe requests, nested
requests, realtime repaint behavior, Magnifier, account semantics, or Linux
release artifacts. Those remain separate work. Existing strategy batches and
the old public r1 reference denominator remain separate.
