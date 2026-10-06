# QQE MOD + SSL Hybrid + Waddah Attar Explosion strategy parity (2026-09-26)

The complete public [v5 strategy by kevinmck100](https://www.tradingview.com/script/YCob5r03-QQE-MOD-SSL-Hybrid-Waddah-Attar-Explosion/) had 5,233 boosts when inspected in Chrome. Its 493-line published source was copied to ignored `.local/continued-popular-20260926/qqe-ssl-wae-strategy-original.pine` (SHA-256 `c1d707c22d2837ba6cdd41121c5060d8b51768f92a1ac877ca6a6e1b4353f522`). It uses no imported libraries.

The unmodified source initially produced six analysis errors: four deprecated `plot(..., transp=...)` arguments and two v5 boolean-to-integer equality checks. TradingView itself runs the published strategy. [TradingView's v5 color documentation](https://www.tradingview.com/pine-script-docs/v5/concepts/colors/) confirms that `transp` remained accepted but deprecated in v5; its [v5 type documentation](https://www.tradingview.com/pine-script-docs/v5/language/type-system/) describes numeric-to-boolean auto-casting. The core now accepts v5 `plot` transparency through the existing legacy output translation and allows v5 numeric-to-boolean equality; v6 continues to reject both forms. The exact public source now analyzes with zero diagnostics or unsupported features.

On `COINBASE:BTCUSD` daily bars, TradingView's downloaded strategy trade list is `I:\sys\下载\QQE_MOD_+_SSL_Hybrid_+_Waddah_Attar_Explosion_COINBASE_BTCUSD_2026-09-26.csv` (SHA-256 `5d5fa72103bc193febb67dcaaedd9cf41a7a49ddcef35201a55e41cbd78be44a`). The immutable comparison copy is ignored `.local/continued-popular-20260926/qqe-ssl-wae-native-trades.csv`. The local input has 4,283 daily bars beginning 2014-12-01 (SHA-256 `0a5f4b9c7b04338bab534088e07c3b13c29bc32a1e992c831122566bcdfe3f13`). The local chart metadata uses quantity precision 6, matching TradingView's trade sizes; leaving it at the generic default 0 changes trade quantities and cents of PnL.

`compare_qqe_ssl_wae.py` checks 60 closed trades against the native CSV: direction, entry/exit dates, prices, six-decimal quantities, and two-decimal net PnL all match. Trade 61 remains open on both sides, with matching entry date, direction, price, and quantity. The receipt is `.local/continued-popular-20260926/qqe-ssl-wae-comparison.json`, with zero failures. The final daily bar was still forming; this comparison claims historical trade-list parity through that snapshot, not realtime tick behavior.

To reproduce from the repository root:

```powershell
$b = '.local/continued-popular-20260926'
target/debug/pine-compat.exe analyze "$b/qqe-ssl-wae-strategy-original.pine" --format json
target/debug/pine-compat.exe run "$b/qqe-ssl-wae-strategy-original.pine" --bars "$b/chart-bars.csv" --chart-symbol COINBASE:BTCUSD --chart-timeframe D --chart-quantity-precision 6 > "$b/qqe-ssl-wae-local-daily.json"
python "$b/compare_qqe_ssl_wae.py"
```
