# Pine v5/v6 `strategy.default_entry_qty()` native check

On September 25, 2026, the independent TradingView layout `Pine quantity oracle 20260925` at `https://www.tradingview.com/chart/K1m3gqVM/` ran this probe on COINBASE:BTCUSD 1D:

```pine
//@version=6
strategy("Default entry qty native probe", initial_capital=10000, default_qty_type=strategy.percent_of_equity, default_qty_value=25, commission_type=strategy.commission.percent, commission_value=0.05, precision=8)
plot(strategy.default_entry_qty(100), title="Default Qty")
```

The chart legend showed `Default Qty: 24.98750600`. Raw equity sizing would be 25 units. Reserving the 0.05% commission gives `25 / 1.0005 = 24.987506246...` units; flooring to the symbol's six-decimal quantity grid gives `24.987506`. The runtime now applies the same percent-of-equity sizing path as actual default-quantity orders to this builtin when the host supplies the symbol quantity precision. The focused test reproduces this native result with an explicit six-decimal chart quantity contract.

The same declaration and plot with `//@version=5` also compiled on the same chart and showed `Default Qty: 24.98750600`. The focused percent-of-equity regression executes both v5 and v6. The repository's language gate exposes this builtin from v5 onward.

Cash and fixed probes in both v5 and v6 on the same chart supplied `commission_value=0.05` and `precision=9`:

| Default quantity | `fill_price` | Native legend in v5 and v6 | Local expectation |
| --- | ---: | ---: | ---: |
| `strategy.cash`, value 100 | 9.87 | `10.131712000` | floor(`100 / 9.87`, 6 decimals) |
| `strategy.fixed`, value 7.123456789 | 10 | `7.123456000` | floor(7.123456789, 6 decimals) |

These outputs show that the builtin applies the symbol quantity grid to cash and fixed quantities too. The cash result does not reserve commission. The runtime and focused regression now cover all three default quantity types in v5 and v6 under this chart contract.

Two further v6 probes changed the percent-of-equity strategy's commission type while keeping initial capital 10,000, default quantity 25%, fill price 100, commission value 1, and the COINBASE:BTCUSD six-decimal quantity grid:

| Commission type | Native `Default Qty` | Budget calculation |
| --- | ---: | ---: |
| `strategy.commission.cash_per_contract` | `24.75247500` | floor(`2500 / (100 + 1)`, 6 decimals) |
| `strategy.commission.cash_per_order` | `24.99000000` | floor(`(2500 - 1) / 100`, 6 decimals) |

The runtime uses these formulas for `strategy.default_entry_qty()` and default-quantity entry/order placement. A focused local test checks both the plot and the resulting entry quantity in v5 and v6.

For independent placement evidence, two more v6 strategies set `process_orders_on_close=true` and placed a single `strategy.entry("L", strategy.long)` on `bar_index == 0`. Each stored `strategy.default_entry_qty(close)` on that bar and plotted it alongside `strategy.position_size`. The first historical COINBASE:BTCUSD daily bar closed at 370 USD. The last chart legend showed:

| Commission type, value 1 | Stored expected quantity | Filled position size |
| --- | ---: | ---: |
| Cash per contract | `6.73854400` | `6.73854400` |
| Cash per order | `6.75405400` | `6.75405400` |

The local regression also checks these two filled quantities with a 370 USD bar. The v5 fixed-fee cases have local regression coverage but no independent native output. The checks do not establish behavior for other symbols. The chart's Pine Editor and applied strategy were restored to their prior short-market probe after the checks.
