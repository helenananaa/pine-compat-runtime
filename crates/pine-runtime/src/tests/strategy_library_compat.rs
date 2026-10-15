use pine_syntax::SourceFile;

use super::*;

fn compile(text: &str) -> pine_ir::HirProgram {
    let analysis = analyze_source(&SourceFile::new("library-compat.pine", text));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis.compatibility.unsupported.is_empty(),
        "{:?}",
        analysis.compatibility
    );
    analysis.hir.expect("executable compatibility program")
}

#[test]
fn legacy_function_aliases_keep_global_parameter_and_local_value_namespaces() {
    for version in [3, 4] {
        let text = format!(
            "//@version={version}\nstudy(\"namespaces\")\nmacd=7.0\natr=11.0\nsma=19.0\nf(sma)=>\n    ema=13.0\n    sma(close,2)+ema(close,2)+sma+ema\n[m,s,h]=macd(close,2,3,2)\nplot(m+s+h)\nplot(atr(2))\nplot(f(17))\nplot(macd+atr+sma)\n"
        );
        let canonical = text
            .replacen(&format!("version={version}"), "version=5", 1)
            .replace("study(", "indicator(")
            .replace("sma(close,", "ta.sma(close,")
            .replace("ema(close,", "ta.ema(close,")
            .replace("macd(close,", "ta.macd(close,")
            .replace("atr(2)", "ta.atr(2)");
        let bars = [bar(10.0), bar(20.0), bar(30.0), bar(40.0)];
        let actual = run_historical(&compile(&text), &bars).unwrap();
        let expected = run_historical(&compile(&canonical), &bars).unwrap();
        assert_eq!(actual.plots, expected.plots, "v{version}");
        assert_values_close(&actual.plots[3].values, &[37.0; 4]);
    }
    let custom = compile(
        "//@version=4\nstudy(\"custom\")\nsma=7\nsma(src,len)=>src*2\nplot(sma(close,2)+sma)\n",
    );
    assert_values_close(
        &run_historical(&custom, &[bar(10.0)]).unwrap().plots[0].values,
        &[27.0],
    );
}

#[test]
fn price_volume_trend_alias_is_v4_only_and_preserves_value_shadowing() {
    let text = "//@version=4\nstudy(\"pvt\")\nplot(pvt)\nplot(pvt[1])\n";
    let canonical = text
        .replace("version=4", "version=5")
        .replace("study(", "indicator(")
        .replace("pvt", "ta.pvt");
    let bars = [bar(10.0), bar(20.0), bar(30.0)];
    assert_eq!(
        run_historical(&compile(text), &bars).unwrap().plots,
        run_historical(&compile(&canonical), &bars).unwrap().plots
    );
    let shadow = compile("//@version=4\nstudy(\"shadow\")\npvt=5\nplot(pvt)\n");
    assert_values_close(
        &run_historical(&shadow, &bars).unwrap().plots[0].values,
        &[5.0; 3],
    );
    for version in [1, 2, 3, 5, 6] {
        let declaration = if version < 5 { "study" } else { "indicator" };
        let analysis = analyze_source(&SourceFile::new(
            "version.pine",
            format!("//@version={version}\n{declaration}(\"version\")\nplot(pvt)\n"),
        ));
        assert!(analysis.hir.is_none(), "v{version}");
    }
}

#[test]
fn legacy_order_directions_preserve_net_orders_and_default_sizing() {
    for version in [2, 3, 4] {
        for (long, short) in [("true", "false"), ("1", "0"), ("long=true", "long=false")] {
            let program = compile(&format!(
                "//@version={version}\nstrategy(\"legacy order\",default_qty_value=2)\nif bar_index==0\n    strategy.order(\"L\",{long})\nif bar_index==2\n    strategy.order(\"S\",{short},qty=3)\nplot(strategy.position_size)\n"
            ));
            let result =
                run_historical(&program, &[bar(10.0); 4]).expect("legacy order directions");
            assert_values_close(&result.plots[0].values, &[0.0, 2.0, 2.0, -1.0]);
            assert_eq!(result.strategy.expect("orders").orders.len(), 2);
        }
    }
    for version in [5, 6] {
        for direction in ["true", "false", "1", "0", "long=true"] {
            let analysis = analyze_source(&SourceFile::new(
                "modern.pine",
                format!(
                    "//@version={version}\nstrategy(\"strict direction\")\nstrategy.order(\"L\",{direction})\n"
                ),
            ));
            assert!(analysis.hir.is_none(), "v{version} {direction}");
        }
    }
}

#[test]
fn table_clear_defaults_each_omitted_end_coordinate_to_its_start() {
    for version in [4, 5, 6] {
        let declaration = if version == 4 { "study" } else { "indicator" };
        let program = compile(&format!(
            "//@version={version}\n{declaration}(\"clear defaults\")\nvar t=table.new(position.top_right,3,3)\nif bar_index==0\n    for c=0 to 2\n        for r=0 to 2\n            table.cell(t,c,r,\"keep\")\n    table.clear(t,0,0)\n    table.clear(t,1,1,2)\n    table.clear(t,0,1,end_row=2)\ntable.clear(na,0,0)\nplot(close)\n"
        ));
        let result = run_historical(&program, &[bar(10.0); 2]).expect("table defaults");
        let coordinates = result.tables[0]
            .snapshots
            .last()
            .expect("table")
            .cells
            .iter()
            .map(|cell| (cell.column, cell.row))
            .collect::<Vec<_>>();
        assert_eq!(coordinates, [(1, 0), (2, 0), (1, 2), (2, 2)]);
        for call in ["table.clear(t,0)", "table.merge_cells(t,0,0)"] {
            let analysis = analyze_source(&SourceFile::new(
                "required.pine",
                format!(
                    "//@version={version}\n{declaration}(\"required coordinates\")\nt=table.new(position.top_right,3,3)\n{call}\n"
                ),
            ));
            assert!(analysis.hir.is_none(), "{call}");
        }
        let invalid = compile(&format!(
            "//@version={version}\n{declaration}(\"invalid coordinates\")\nt=table.new(position.top_right,3,3)\ntable.clear(t,1,1,end_row=0)\n"
        ));
        assert!(run_historical(&invalid, &[bar(10.0)]).is_err());
    }
}

#[test]
fn udf_concat_preserves_target_identity_and_snapshots_self_aliases() {
    for version in [4, 5, 6] {
        let declaration = if version == 4 { "study" } else { "indicator" };
        let concat = if version == 4 {
            "array.concat(target, source)"
        } else {
            "target.concat(source)"
        };
        let program = compile(&format!(
            "//@version={version}\n{declaration}(\"UDF concat\")\njoin(target, source) =>\n    {concat}\nleft = array.new_float(1, 10)\nright = array.new_float(2, 2)\njoined = join(left, right)\narray.push(joined, 7)\nplot(array.size(left))\nplot(array.sum(left))\nplot(array.size(right))\nselfJoined = join(right, right)\nplot(array.size(selfJoined))\nplot(array.sum(right))\n"
        ));
        let result = run_historical(&program, &[bar(1.0), bar(2.0), bar(3.0)])
            .expect("UDF concat execution");
        assert_eq!(result.plots.len(), 5);
        for (plot, expected) in result.plots.iter().zip([4.0, 21.0, 2.0, 4.0, 8.0]) {
            assert_values_close(&plot.values, &[expected; 3]);
        }
    }
}

#[test]
fn v4_inline_branches_and_local_array_builders_execute() {
    let program = compile(
        r#"//@version=4
study("legacy value paths")
twice(x) => data = x + 1, data * 2
build(x) =>
    values = array.new_float(0)
    for i = 0 to 2
        array.push(values, x + i)
    values
v = if bar_index == 0
    twice(2)
else if bar_index == 1
    array.get(build(3), 2)
else
    8
plot(v)
plot(cum(1))
plot(true == 1 ? 1 : 0)
plot(false == 0 ? 1 : 0)
plot(true != 1 ? 1 : 0)
"#,
    );
    let result =
        run_historical(&program, &[bar(1.0), bar(2.0), bar(3.0)]).expect("legacy execution");
    assert_values_close(&result.plots[0].values, &[6.0, 5.0, 8.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 2.0, 3.0]);
    assert_values_close(&result.plots[2].values, &[0.0, 0.0, 0.0]);
    assert_values_close(&result.plots[3].values, &[0.0, 0.0, 0.0]);
    assert_values_close(&result.plots[4].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn v4_conditional_orders_preserve_pending_orders_and_dynamic_identity() {
    let program = compile(
        r#"//@version=4
strategy("conditional dynamic identity")
if bar_index == 0
    strategy.entry(tostring(bar_index), long=true, qty=1, limit=10)
if bar_index == 1
    strategy.cancel(tostring(bar_index - 1), when=false)
    strategy.cancel_all(false)
if bar_index == 2
    strategy.exit("disabled", tostring(bar_index - 2), stop=10, when=false)
    strategy.order("disabled order", strategy.long, qty=2, when=false)
if bar_index == 3
    strategy.close(tostring(bar_index - 3), when=true)
plot(strategy.position_size)
"#,
    );
    let bars = (0..5)
        .map(|index| Bar {
            time: index * 60_000,
            open: if index == 2 { 10.0 } else { 12.0 },
            high: if index == 2 { 11.0 } else { 13.0 },
            low: if index == 2 { 9.0 } else { 11.0 },
            close: if index == 2 { 10.0 } else { 12.0 },
            volume: 100.0,
        })
        .collect::<Vec<_>>();
    let result = run_historical(&program, &bars).expect("conditional orders");
    assert_values_close(&result.plots[0].values, &[0.0, 0.0, 1.0, 1.0, 0.0]);
    assert_eq!(result.strategy.expect("broker output").trades.len(), 1);
}

#[test]
fn v6_keeps_removed_when_parameters_and_legacy_direction_fail_closed() {
    for call in [
        "strategy.order(\"E\", strategy.long, when=false)",
        "strategy.exit(\"X\", \"E\", stop=1, when=false)",
        "strategy.cancel(\"E\", when=false)",
        "strategy.cancel_all(when=false)",
        "strategy.entry(\"E\", long=true)",
    ] {
        let analysis = analyze_source(&SourceFile::new(
            "v6.pine",
            format!("//@version=6\nstrategy(\"version boundary\")\n{call}\n"),
        ));
        assert!(analysis.hir.is_none(), "{call}");
        assert!(
            !analysis.diagnostics.is_empty() || !analysis.compatibility.unsupported.is_empty(),
            "{call}"
        );
    }
}

#[test]
fn v1_scalar_aliases_are_available_without_admitting_v4_only_cmo() {
    let program = compile(
        "//@version=1\nstudy(\"aliases\")\nplot(cum(1))\nplot(roc(close,2))\nplot(sar(.02,.02,.2))\nplot(sin(1))\nplot(dev(close,2))\nplot(correlation(close,open,2))\nplot(cog(close,2))\nplot(percentrank(close,2))\nplot(lowestbars(low,2))\nplot(highestbars(high,2))\n",
    );
    let result = run_historical(&program, &[bar(1.0), bar(2.0), bar(3.0)]).expect("v1 aliases");
    assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
    let analysis = analyze_source(&SourceFile::new(
        "v3.pine",
        "//@version=3\nstudy(\"version gate\")\nplot(cmo(close,2))\n",
    ));
    assert!(analysis.hir.is_none());
}

#[test]
fn local_matrix_chains_survive_scope_teardown() {
    let program = compile(
        r#"//@version=6
indicator("local matrix chains", behind_chart=false)
var matrix<float> result = na
if barstate.isfirst
    left = matrix.new<float>(1, 1, 2.0)
    right = left.sum(1.0)
    result := right.transpose().mult(left)
plot(result.get(0, 0))
"#,
    );
    let result = run_historical(&program, &[bar(1.0), bar(2.0)]).expect("local matrix chain");
    assert_values_close(&result.plots[0].values, &[6.0, 6.0]);
}

#[test]
fn combined_exits_fill_one_leg_and_share_the_reserved_quantity() {
    for (direction, stop, limit, activation, high, low, expected, comment) in [
        ("long", 95.0, 120.0, 105.0, 104.0, 94.0, 95.0, "loss"),
        ("long", 95.0, 108.0, 105.0, 112.0, 100.0, 108.0, "profit"),
        ("long", 95.0, 120.0, 105.0, 112.0, 100.0, 110.0, "trail"),
        ("short", 105.0, 80.0, 95.0, 106.0, 96.0, 105.0, "loss"),
        ("short", 105.0, 92.0, 95.0, 100.0, 88.0, 92.0, "profit"),
        ("short", 105.0, 80.0, 95.0, 100.0, 88.0, 90.0, "trail"),
    ] {
        let program = compile(&format!(
            r#"//@version=6
strategy("combined exit", margin_long=0, margin_short=0)
if bar_index == 0
    strategy.entry("E", strategy.{direction}, qty=2)
    strategy.exit("X", "E", stop={stop}, limit={limit}, trail_price={activation}, trail_offset=200, qty=1, comment_loss="loss", comment_profit="profit", comment_trailing="trail")
plot(strategy.position_size)
plot(strategy.closedtrades.exit_comment(0) == "{comment}" ? 1 : 0)
"#
        ));
        let bars = [
            bar(100.0),
            bar(100.0),
            bar_ohlc(100.0, high, low, 100.0),
            bar(100.0),
        ];
        let result = run_historical(&program, &bars).expect("combined fills");
        let strategy = result.strategy.expect("broker output");
        assert_eq!(strategy.trades.len(), 1, "{direction} {comment}");
        assert_eq!(
            strategy.trades[0].exit_price, expected,
            "{direction} {comment}"
        );
        assert_eq!(strategy.trades[0].qty.abs(), 1.0);
        assert_eq!(result.plots[1].values[3].as_f64(), Some(1.0));
        assert_eq!(
            result.plots[0].values[3].as_f64(),
            Some(if direction == "long" { 1.0 } else { -1.0 })
        );
    }
}

#[test]
fn paired_exit_parameters_obey_v5_and_v6_precedence() {
    for (version, expected) in [(5, 95.0), (6, 99.0)] {
        let program = compile(&format!(
            r#"//@version={version}
strategy("paired exit", margin_long=0)
if bar_index == 0
    strategy.entry("E", strategy.long, qty=1)
    strategy.exit("X", "E", stop=95, loss=100, limit=120, profit=1000)
plot(strategy.position_size)
"#
        ));
        let result = run_historical(
            &program,
            &[bar(100.0), bar(100.0), bar_ohlc(100.0, 102.0, 94.0, 100.0)],
        )
        .expect("paired exits");
        assert_eq!(
            result.strategy.expect("broker output").trades[0].exit_price,
            expected
        );
    }
}

#[test]
fn exit_runtime_na_quantity_uses_percentage_or_full_default() {
    for version in [4, 5, 6] {
        for combined in [false, true] {
            for (quantity, expected) in [
                ("qty=unspecified,qty_percent=50", 1.0),
                ("qty_percent=unspecified", 2.0),
                ("qty=unspecified,qty_percent=unspecified", 2.0),
            ] {
                let trailing = if combined {
                    ",trail_price=110,trail_offset=200"
                } else {
                    ""
                };
                let program = compile(&format!(
                    "//@version={version}\nstrategy(\"runtime NA exit quantity\",margin_long=0)\nfloat unspecified=bar_index>=0?na:1\nif bar_index==0\n    strategy.entry(\"E\",strategy.long,qty=2)\nif bar_index==1\n    strategy.exit(\"X\",\"E\",stop=95,{quantity}{trailing})\nplot(strategy.position_size)\n"
                ));
                let result = run_historical(
                    &program,
                    &[bar(100.0), bar(100.0), bar_ohlc(100.0, 101.0, 94.0, 100.0)],
                )
                .expect("runtime NA exit quantity");
                let output = result.strategy.expect("broker output");
                assert_eq!(output.trades.len(), 1, "v{version} {combined} {quantity}");
                assert_eq!(output.trades[0].qty, expected);
                assert_eq!(output.trades[0].exit_price, 95.0);
                assert_eq!(result.plots[0].values[2].as_f64(), Some(2.0 - expected));
            }
        }
    }
}

#[test]
fn zero_capital_and_triangle_labels_are_legal() {
    let program = compile(
        r#"//@version=4
strategy("zero capital", initial_capital=0)
label.new(bar_index, close, style=label.style_triangleup)
label.new(bar_index, low, style=label.style_triangledown)
plot(strategy.equity)
"#,
    );
    let result = run_historical(&program, &[bar(100.0)]).expect("zero capital and labels");
    assert_values_close(&result.plots[0].values, &[0.0]);
}

#[test]
fn source_exit_positions_reserve_qty_and_preserve_versioned_comments() {
    for version in [4, 5, 6] {
        let tail = if version == 4 {
            ",true"
        } else {
            ",\"take profit\""
        };
        let program = compile(&format!(
            "//@version={version}\nstrategy(\"source positions\")\nif bar_index == 0\n    strategy.entry(\"E\",strategy.long,qty=2)\n    strategy.exit(\"X\",\"E\",1,na,100,na,na,na,na,na,na,\"\",\"common\"{tail})\nplot(strategy.position_size)\n"
        ));
        let result = run_historical(
            &program,
            &[bar(100.0), bar(100.0), bar_ohlc(100.0, 102.0, 100.0, 101.0)],
        )
        .expect("positional exit");
        let output = result.strategy.expect("broker");
        assert_eq!(output.trades.len(), 1);
        assert_eq!(output.trades[0].qty, 1.0);
        assert_eq!(output.trades[0].exit_price, 101.0);
        assert_eq!(result.plots[0].values[2].as_f64(), Some(1.0));
    }
}

#[test]
fn external_when_false_never_emits_order_intents() {
    let program = compile(
        "//@version=4\nstrategy(\"external gates\")\nstrategy.order(\"E\",strategy.long,qty=1,when=false)\nstrategy.exit(\"X\",\"E\",stop=1,when=false)\nstrategy.cancel(\"E\",when=false)\nstrategy.cancel_all(when=false)\nplot(strategy.position_size)\n",
    );
    let account = ExternalAccountFrame {
        time: 0,
        position_size: 0.0,
        position_avg_price: None,
        equity: 1000.0,
        initial_capital: 1000.0,
        netprofit: 0.0,
        openprofit: 0.0,
    };
    let mut runtime = HistoricalRuntime::new(&program)
        .with_external_accounts(vec![account])
        .expect("account");
    runtime.append_bar(bar(1.0)).expect("disabled calls");
    assert!(
        runtime
            .external_intents()
            .expect("external mode")
            .is_empty()
    );
}

#[test]
fn combined_exit_cancellation_and_replacement_preserve_one_reservation() {
    let program = compile(
        r#"//@version=6
strategy("cancel and replace",margin_long=0)
if bar_index == 0
    strategy.entry("E",strategy.long,qty=2)
    strategy.exit("X","E",stop=90,limit=120,trail_price=110,trail_offset=200,qty=1)
if bar_index == 1
    strategy.cancel("X")
if bar_index == 2
    strategy.exit("X","E",stop=95,limit=120,trail_price=110,trail_offset=200,qty=1)
if bar_index == 3
    strategy.exit("X","E",stop=97,limit=120,trail_price=110,trail_offset=200,qty=1)
plot(strategy.position_size)
"#,
    );
    let bars = [
        bar(100.0),
        bar(100.0),
        bar_ohlc(100.0, 101.0, 89.0, 100.0),
        bar(100.0),
        bar_ohlc(100.0, 101.0, 96.0, 100.0),
    ];
    let bars = std::array::from_fn::<_, 5, _>(|index| Bar {
        time: index as i64 * 60_000,
        ..bars[index]
    });
    let result = run_historical(&program, &bars).expect("replacement");
    let output = result.strategy.as_ref().expect("broker");
    assert_eq!(output.trades.len(), 1);
    assert_eq!(output.trades[0].exit_price, 97.0);
    assert_eq!(output.trades[0].qty, 1.0);
    assert_eq!(result.plots[0].values[4].as_f64(), Some(1.0));
    let mut realtime = RealtimeRuntime::new(&program);
    for input in bars.iter().take(4) {
        realtime
            .update(BarUpdate::historical(*input))
            .expect("seed");
    }
    let forming = realtime
        .update(BarUpdate::forming(bars[4]))
        .expect("forming fill");
    // Close-only strategies keep the reservation until confirmation.
    assert!(forming.strategy.as_ref().expect("broker").trades.is_empty());
    let rollback = realtime
        .update(BarUpdate::forming(Bar {
            low: 99.0,
            ..bars[4]
        }))
        .expect("rollback fill");
    assert!(
        rollback
            .strategy
            .as_ref()
            .expect("broker")
            .trades
            .is_empty()
    );
    let confirmed = realtime
        .update(BarUpdate::confirmed(bars[4]))
        .expect("confirm");
    // Realtime observes ticks, not the unseen historical high/low path.
    assert!(
        confirmed
            .strategy
            .as_ref()
            .expect("broker")
            .trades
            .is_empty()
    );
    let next = realtime
        .update(BarUpdate::confirmed(Bar {
            time: 300_000,
            ..bar(96.0)
        }))
        .expect("observed gap fill");
    let output = next.strategy.expect("broker");
    assert_eq!(output.trades.len(), 1);
    assert_eq!(output.trades[0].exit_price, 96.0);
    assert_eq!(output.trades[0].qty, 1.0);
}

#[test]
fn combined_exit_without_entry_id_covers_future_pyramided_entries() {
    let program = compile(
        r#"//@version=6
strategy("all entries",pyramiding=3,margin_long=0,close_entries_rule="ANY")
if bar_index == 0 or bar_index == 2
    strategy.entry("E",strategy.long,qty=1)
if bar_index == 0
    strategy.exit("X",stop=95,limit=120,trail_price=110,trail_offset=200)
plot(strategy.position_size)
"#,
    );
    let result = run_historical(
        &program,
        &[
            bar(100.0),
            bar(100.0),
            bar(101.0),
            bar(102.0),
            bar_ohlc(100.0, 101.0, 94.0, 100.0),
        ],
    )
    .expect("all entries");
    let output = result.strategy.expect("broker");
    assert_eq!(output.trades.len(), 2);
    assert!(
        output
            .trades
            .iter()
            .all(|trade| trade.exit_price == 95.0 && trade.qty == 1.0)
    );
    assert_eq!(result.plots[0].values[4].as_f64(), Some(0.0));
}

#[test]
fn combined_exit_named_entry_resolves_relative_prices_per_pyramided_trade() {
    for version in [5, 6] {
        let program = compile(&format!(
            "//@version={version}\nstrategy(\"named pyramided exits\",pyramiding=3,margin_long=0,close_entries_rule=\"ANY\")\nif bar_index==0 or bar_index==2\n    strategy.entry(\"E\",strategy.long,qty=1)\nif bar_index==3\n    strategy.exit(\"X\",\"E\",loss=100,trail_points=3000,trail_offset=200)\nplot(strategy.position_size)\n"
        ));
        let result = run_historical(
            &program,
            &[
                bar(100.0),
                bar(100.0),
                bar(100.0),
                bar(110.0),
                bar_ohlc(110.0, 111.0, 108.0, 110.0),
                bar_ohlc(100.0, 101.0, 98.0, 100.0),
            ],
        )
        .expect("per-trade relative exits");
        let output = result.strategy.expect("broker output");
        assert_eq!(output.trades.len(), 2, "v{version}");
        assert_eq!(output.trades[0].entry_price, 110.0);
        assert_eq!(output.trades[0].exit_price, 109.0);
        assert_eq!(output.trades[1].entry_price, 100.0);
        assert_eq!(output.trades[1].exit_price, 99.0);
        assert_values_close(&result.plots[0].values, &[0.0, 1.0, 1.0, 2.0, 1.0, 0.0]);
    }
}

#[test]
fn combined_exit_gap_uses_the_active_protective_leg_comment() {
    let program = compile(
        r#"//@version=6
strategy("gap metadata",margin_long=0)
if bar_index == 0
    strategy.entry("E",strategy.long,qty=1)
    strategy.exit("X","E",stop=95,limit=120,trail_price=105,trail_offset=200,comment_loss="fixed",comment_trailing="trail")
plot(strategy.closedtrades.exit_comment(0)=="trail"?1:0)
"#,
    );
    let result = run_historical(
        &program,
        &[
            bar(100.0),
            bar(100.0),
            bar_ohlc(104.0, 110.0, 104.0, 110.0),
            bar(90.0),
        ],
    )
    .expect("gap exit");
    let output = result.strategy.expect("broker");
    assert_eq!(output.trades.len(), 1);
    assert_eq!(output.trades[0].exit_price, 90.0);
    assert_eq!(result.plots[0].values[3].as_f64(), Some(1.0));
}

#[test]
fn v4_numeric_entry_direction_uses_legacy_boolean_conversion() {
    for long in ["1", "-1", "0.25", "bar_index + 0.25"] {
        let program = compile(&format!(
            "//@version=4\nstrategy(\"numeric directions\")\nif bar_index==0\n    strategy.entry(\"L\",{long},qty=2)\nif bar_index==2\n    strategy.entry(\"S\",0.0,qty=1)\nplot(strategy.position_size)\n"
        ));
        let result = run_historical(&program, &[bar(10.0), bar(11.0), bar(12.0), bar(13.0)])
            .expect("legacy numeric direction");
        assert_values_close(&result.plots[0].values, &[0.0, 2.0, 2.0, -1.0]);
        let trades = result.strategy.expect("broker output").trades;
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].entry_price, 11.0);
        assert_eq!(trades[0].exit_price, 13.0);
    }
    for version in [5, 6] {
        let analysis = analyze_source(&SourceFile::new(
            "modern-numeric-direction.pine",
            format!("//@version={version}\nstrategy(\"boundary\")\nstrategy.entry(\"E\",1)\n"),
        ));
        assert!(analysis.hir.is_none());
    }
}

#[test]
fn inline_input_udf_argument_is_versioned_and_preserves_value() {
    for version in [4, 5, 6] {
        let declaration = if version == 4 { "study" } else { "indicator" };
        let program = compile(&format!(
            "//@version={version}\n{declaration}(\"inline input\")\nf(x)=>x+1\nplot(f(input(2,\"Wrapped input\")))\n"
        ));
        let result = run_historical(&program, &[bar(10.0), bar(11.0)]).expect("inline input");
        assert_values_close(&result.plots[0].values, &[3.0, 3.0]);
    }
    let analysis = analyze_source(&SourceFile::new(
        "v3-inline-input.pine",
        "//@version=3\nstudy(\"boundary\")\nf(x)=>x+1\nplot(f(input(2)))\n",
    ));
    assert!(analysis.hir.is_none());
}

#[test]
fn strategy_close_source_positions_preserve_partial_fills_and_timing() {
    for version in [4, 5, 6] {
        for percentage in [false, true] {
            for immediate in [false, true] {
                if version == 4 && immediate {
                    continue;
                }
                let quantity = if percentage { "na,50" } else { "1,na" };
                let prefix = if version <= 5 { "true," } else { "" };
                let tail = if version >= 5 {
                    format!(",{immediate},false")
                } else {
                    String::new()
                };
                let program = compile(&format!(
                    "//@version={version}\nstrategy(\"close positions\")\nif bar_index==0\n    strategy.entry(\"E\",strategy.long,qty=2)\nif bar_index==2\n    strategy.close(\"E\",{prefix}\"half\",{quantity},\"half alert\"{tail})\nplot(strategy.position_size)\n"
                ));
                let result = run_historical(
                    &program,
                    &[bar(10.0), bar(11.0), bar(12.0), bar(13.0), bar(14.0)],
                )
                .expect("partial close positions");
                // Without calc_on_order_fills, plots precede the immediate fill.
                let expected = [0.0, 2.0, 2.0, 1.0, 1.0];
                assert_values_close(&result.plots[0].values, &expected);
                let trades = result.strategy.expect("broker output").trades;
                assert_eq!(trades.len(), 1);
                assert_eq!(trades[0].qty, 1.0);
                assert_eq!(trades[0].exit_price, if immediate { 12.0 } else { 13.0 });
            }
        }
    }
}

#[test]
fn v5_close_second_source_position_is_when_and_keeps_false_order_disabled() {
    let program = compile(
        "//@version=5\nstrategy(\"positional when\")\nif bar_index==0\n    strategy.entry(\"E\",strategy.long,qty=2)\nif bar_index==1\n    strategy.close(\"E\",false)\nif bar_index==2\n    strategy.close(\"E\",true)\nplot(strategy.position_size)\n",
    );
    let result = run_historical(&program, &[bar(10.0), bar(11.0), bar(12.0), bar(13.0)])
        .expect("versioned close when");
    assert_values_close(&result.plots[0].values, &[0.0, 2.0, 2.0, 0.0]);
}

#[test]
fn legacy_aliases_match_modern_values_and_keep_version_boundaries() {
    let aliases = [
        ("swma(close)", "ta.swma(close)", 3),
        ("tsi(close,2,3)", "ta.tsi(close,2,3)", 3),
        ("tan(0.5)", "math.tan(0.5)", 3),
        (
            "percentile_nearest_rank(close,3,25)",
            "ta.percentile_nearest_rank(close,3,25)",
            3,
        ),
        (
            "percentile_linear_interpolation(close,3,25)",
            "ta.percentile_linear_interpolation(close,3,25)",
            3,
        ),
        ("wpr(3)", "ta.wpr(3)", 4),
        ("tonumber(\"2.5\")", "str.tonumber(\"2.5\")", 4),
    ];
    let bars = [
        bar(10.0),
        bar(12.0),
        bar(11.0),
        bar(15.0),
        bar(13.0),
        bar(16.0),
    ];
    for (old, new, minimum) in aliases {
        let reference = run_historical(
            &compile(&format!(
                "//@version=6\nindicator(\"reference\")\nplot({new})\n"
            )),
            &bars,
        )
        .expect("reference");
        for version in minimum..=4 {
            let result = run_historical(
                &compile(&format!(
                    "//@version={version}\nstudy(\"legacy alias\")\nplot({old})\n"
                )),
                &bars,
            )
            .expect("alias");
            assert_eq!(
                result.plots[0].values, reference.plots[0].values,
                "{old} v{version}"
            );
        }
    }
    for call in ["wpr(3)", "supertrend(3,10)", "tonumber(\"2.5\")"] {
        let analysis = analyze_source(&SourceFile::new(
            "v3-newer-alias.pine",
            format!("//@version=3\nstudy(\"boundary\")\nx={call}\n"),
        ));
        assert!(analysis.hir.is_none(), "{call}");
    }
    let program = compile(
        "//@version=4\nstudy(\"supertrend alias\")\n[s,d]=supertrend(3,2)\nplot(s)\nplot(d)\n",
    );
    let result = run_historical(&program, &bars).expect("legacy supertrend");
    assert_eq!(result.plots.len(), 2);
}

#[test]
fn nested_inline_input_keeps_one_metadata_identity_and_override() {
    for version in [4, 5, 6] {
        let declaration = if version == 4 { "study" } else { "indicator" };
        let program = compile(&format!(
            "//@version={version}\n{declaration}(\"nested inline input\")\nf(x)=>x+1\ng()=>f(input(2,\"Nested input\")+1)\nplot(g())\nplot(g()+g())\n"
        ));
        let inputs = input_calls(&program);
        assert_eq!(inputs.len(), 1);
        let result = run_historical_with_input_overrides(
            &program,
            &[bar(10.0), bar(11.0)],
            InputOverrides::new().with_value(inputs[0].call_site_id, PineValue::Int(7)),
        )
        .expect("nested input override");
        assert_values_close(&result.plots[0].values, &[9.0, 9.0]);
        assert_values_close(&result.plots[1].values, &[18.0, 18.0]);
    }
}

#[test]
fn legacy_duplicate_inputs_use_positional_priority_and_last_named_metadata() {
    for version in [1, 2, 3, 4] {
        let program = compile(&format!(
            "//@version={version}\nstudy(\"duplicate input\")\na=input(2,defval=1,title=\"A\")\nb=input(2,defval=3,title=\"B\")\nc=input(1,title=\"First title\",title=\"Second title\")\nplot(a)\nplot(b)\nplot(c)\nplot(cos(0.5))\n"
        ));
        let inputs = input_calls(&program);
        assert_eq!(inputs.len(), 3);
        assert_eq!(inputs[0].default_value, Some(PineValue::Int(2)));
        assert_eq!(inputs[1].default_value, Some(PineValue::Int(2)));
        assert_eq!(inputs[2].title.as_deref(), Some("Second title"));
        let result = run_historical(&program, &[bar(1.0), bar(2.0)]).expect("legacy duplicates");
        for (plot, value) in result.plots.iter().zip([2.0, 2.0, 1.0, 0.5_f64.cos()]) {
            assert_values_close(&plot.values, &[value, value]);
        }
    }
}

#[test]
fn discarded_legacy_input_defaults_still_require_the_selected_type() {
    for version in [1, 2, 3, 4] {
        let analysis = analyze_source(&SourceFile::new(
            "bad-repeated-input.pine",
            format!(
                "//@version={version}\nstudy(\"type boundary\")\nx=input(2,defval=\"ignored\")\nplot(x)\n"
            ),
        ));
        assert!(analysis.hir.is_none());
        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|d| d.code == "E_CALL_ARG_TYPE")
        );
    }
}

#[test]
fn v4_named_repeated_defaults_choose_last_value_and_group_binding() {
    let program = compile(
        "//@version=4\nstudy(\"named duplicate input\")\nx=input(defval=2,defval=3,title=\"Named default\",group=\"First group\",group=\"Second group\")\nplot(x)\n",
    );
    let inputs = input_calls(&program);
    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0].default_value, Some(PineValue::Int(3)));
    let result = run_historical(&program, &[bar(1.0), bar(2.0)]).expect("named default");
    assert_values_close(&result.plots[0].values, &[3.0, 3.0]);
}

#[test]
fn ema_integer_arithmetic_aliases_truncate_only_the_final_length() {
    let bars = (0..32)
        .map(|i| bar(10.0 + (i as f64 / 3.0).sin()))
        .collect::<Vec<_>>();
    for version in [4, 5, 6] {
        let decl = if version == 4 { "study" } else { "indicator" };
        let ema = if version == 4 { "ema" } else { "ta.ema" };
        let program = compile(&format!(
            "//@version={version}\n{decl}(\"represented length\")\nn=input(19)\na=n/2\nb=a*2\nc=timeframe.isintraday?n/timeframe.multiplier*7:7\nplot(a)\nplot(b)\nplot({ema}(close,a))\nplot({ema}(close,int(a)))\nplot({ema}(close,b))\nplot({ema}(close,19))\nplot({ema}(close,c))\nplot({ema}(close,int(c)))\n"
        ));
        let result = run_historical(&program, &bars).expect("represented length aliases");
        assert_values_close(&result.plots[0].values, &[9.5; 32]);
        assert_values_close(&result.plots[1].values, &[19.0; 32]);
        for (left, right) in [(2, 3), (4, 5), (6, 7)] {
            assert_eq!(result.plots[left].values, result.plots[right].values);
        }
        assert!(
            result.plots[4]
                .values
                .last()
                .unwrap()
                .as_f64()
                .is_some_and(f64::is_finite)
        );
    }
}

#[test]
fn ema_length_rejects_explicit_float_and_series_arithmetic() {
    for version in [4, 5, 6] {
        let decl = if version == 4 { "study" } else { "indicator" };
        let ema = if version == 4 { "ema" } else { "ta.ema" };
        for body in [
            "a=input(2.8)",
            "n=input(19)\na=n/2.0",
            "a=bar_index/2",
            "n=input(19)\na=n/2\na:=2.8",
        ] {
            let analysis = analyze_source(&SourceFile::new(
                "bad-length.pine",
                format!(
                    "//@version={version}\n{decl}(\"bad length\")\n{body}\nplot({ema}(close,a))\n"
                ),
            ));
            assert!(analysis.hir.is_none(), "{version}: {body}");
            assert!(
                analysis
                    .diagnostics
                    .iter()
                    .any(|d| d.code == "E_CALL_ARG_TYPE"),
                "{:?}",
                analysis.diagnostics
            );
        }
    }
}

#[test]
fn v4_resolution_named_options_preserve_metadata_and_selector_value() {
    let program = compile(
        "//@version=4\nstudy(\"resolution options\")\ntf=input(\"60\",title=\"Timeframe\",type=input.resolution,options=[\"1\",\"60\",\"D\"],tooltip=\"Tip\")\nplot(tf==\"60\"?1:0)\n",
    );
    let inputs = input_calls(&program);
    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0].name, "input.timeframe");
    assert_eq!(
        inputs[0].options,
        vec![
            PineValue::String("1".into()),
            PineValue::String("60".into()),
            PineValue::String("D".into())
        ]
    );
    let result = run_historical(&program, &[bar(1.0), bar(2.0)]).expect("resolution input");
    assert_values_close(&result.plots[0].values, &[1.0, 1.0]);
}

#[test]
fn expression_blocks_return_terminal_assignments_once() {
    for version in [4, 5, 6] {
        let decl = if version == 4 { "study" } else { "indicator" };
        let program = compile(&format!(
            "//@version={version}\n{decl}(\"assignment returns\")\nf(n)=>\n    float s=0\n    if n>0\n        for i=1 to n\n            s:=s+i\n    else\n        s:=0\nx=0\ny=if true\n    x:=7\nelse\n    x:=99\nq=for i=0 to 2\n    x+=i\nw=while x<12\n    x+=1\nd=if true\n    int local=23\nelse\n    int local=29\nplot(f(3))\nplot(f(0))\nplot(y)\nplot(q)\nplot(w)\nplot(x)\nplot(d)\n"
        ));
        let result = run_historical(&program, &[bar(1.0); 2]).unwrap();
        for (plot, expected) in result
            .plots
            .iter()
            .zip([6.0, 0.0, 7.0, 10.0, 12.0, 12.0, 23.0])
        {
            assert_values_close(&plot.values, &[expected; 2]);
        }
    }
}

#[test]
fn loop_assignment_returns_preserve_break_continue_and_skipped_iteration() {
    let program = compile(
        "//@version=5\nindicator(\"assignment control flow\")\nx=0\na=for i=0 to 4\n    if i==1\n        continue\n    if i==3\n        break\n    x+=i\nb=while false\n    x+=100\nitems=array.from(2,3,5)\nc=for item in items\n    x+=item\nplot(a)\nplot(b)\nplot(c)\nplot(x)\n",
    );
    let result = run_historical(&program, &[bar(1.0); 2]).unwrap();
    assert_values_close(&result.plots[0].values, &[2.0; 2]);
    assert!(result.plots[1].values.iter().all(PineValue::is_na));
    assert_values_close(&result.plots[2].values, &[12.0; 2]);
    assert_values_close(&result.plots[3].values, &[12.0; 2]);
}

#[test]
fn missing_else_preserves_side_effects_and_versioned_bool_defaults() {
    for version in [4, 5, 6] {
        let decl = if version == 4 { "study" } else { "indicator" };
        let bool_probe = if version >= 6 {
            "b==false?1:0"
        } else {
            "na(b)?1:0"
        };
        let function_probe = if version >= 6 {
            "g(false)==false?1:0"
        } else {
            "na(g(false))?1:0"
        };
        let program = compile(&format!(
            "//@version={version}\n{decl}(\"missing else\")\nx=0\ny=if bar_index%2==0\n    x+=7\nz=if bar_index<0\n    x+=99\nb=if bar_index<0\n    true\ng(c)=>\n    if c\n        true\nplot(y)\nplot(z)\nplot(x)\nplot({bool_probe})\nplot({function_probe})\n"
        ));
        let result = run_historical(&program, &[bar(1.0); 4]).unwrap();
        assert_eq!(
            result.plots[0].values,
            vec![
                PineValue::Int(7),
                PineValue::Na,
                PineValue::Int(7),
                PineValue::Na
            ]
        );
        assert!(result.plots[1].values.iter().all(PineValue::is_na));
        assert_values_close(&result.plots[2].values, &[7.0, 0.0, 7.0, 0.0]);
        assert_values_close(&result.plots[3].values, &[1.0; 4]);
        assert_values_close(&result.plots[4].values, &[1.0; 4]);
    }
}

#[test]
fn zero_default_quantity_skips_orders_and_preserves_explicit_quantity() {
    for version in [4, 5, 6] {
        for qty_type in ["fixed", "cash", "percent_of_equity"] {
            let program = compile(&format!(
                "//@version={version}\nstrategy(\"zero default\",default_qty_type=strategy.{qty_type},default_qty_value=0)\nif bar_index==0\n    strategy.entry(\"Zero\",strategy.long)\n    strategy.order(\"Zero order\",strategy.short)\nif bar_index==2\n    strategy.entry(\"Explicit\",strategy.long,qty=1)\nplot(strategy.position_size)\n"
            ));
            let result = run_historical(&program, &[bar(10.0); 4]).unwrap();
            assert_values_close(&result.plots[0].values, &[0.0, 0.0, 0.0, 1.0]);
            let orders = result.strategy.unwrap().orders;
            assert_eq!(orders.len(), 1);
            assert_eq!(orders[0].id, "Explicit");
        }
        let analysis = analyze_source(&SourceFile::new(
            "negative-default.pine",
            format!("//@version={version}\nstrategy(\"negative default\",default_qty_value=-1)\n"),
        ));
        assert!(analysis.hir.is_none());
        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|d| d.code == "E_CALL_ARG_VALUE")
        );
    }
}

#[test]
fn switch_blocks_return_assignments_and_declarations_from_selected_arm_once() {
    for version in [5, 6] {
        let program = compile(&format!(
            "//@version={version}\nindicator(\"switch assignment result\")\nx=0.0\na=switch\n    bar_index%2==0=>\n        x+=2.5\n    =>\n        float local=9.5\nb=switch bar_index%2\n    0=>\n        x+=4\n    =>\n        x+=7\nplot(a)\nplot(b)\nplot(x)\n"
        ));
        let result = run_historical(&program, &[bar(1.0); 4]).unwrap();
        assert_values_close(&result.plots[0].values, &[2.5, 9.5, 2.5, 9.5]);
        assert_values_close(&result.plots[1].values, &[6.5, 7.0, 6.5, 7.0]);
        assert_eq!(result.plots[1].values, result.plots[2].values);
    }
}

#[test]
fn numeric_for_loops_preserve_fractional_bounds_and_counter_types() {
    for version in [4, 5, 6] {
        let declaration = if version == 4 { "study" } else { "indicator" };
        let program = compile(&format!(
            r#"//@version={version}
{declaration}("numeric loops")
a=0.0
b=0.0
c=0
d=0
for i=0.8 to 3.8 by 0.5
    a+=i
for j=-0.8 to -3.8
    b+=j
for k=-3 to -0.8
    c+=1
for m=3 to 0.8
    d+=1
indices=array.from(7,8,9,10)
e=0
for n=0 to 3.8
    e+=array.get(indices,n)
plot(a)
plot(b)
plot(c)
plot(d)
plot(e)
"#
        ));
        let result = run_historical(&program, &[bar(10.0); 3]).unwrap();
        for (plot, expected) in result.plots.iter().zip([16.1, -9.2, 3.0, 3.0, 34.0]) {
            assert_values_close(&plot.values, &[expected; 3]);
        }
    }
}

#[test]
fn fractional_for_expression_returns_and_loop_control_keep_float_values() {
    for version in [5, 6] {
        let program = compile(&format!(
            r#"//@version={version}
indicator("float return")
f(float start)=>
    for i=start to 3.8
        i
x=for j=0.8 to 3.8
    if j>2
        break
    j
y=for k=0.8 to 3.8
    if k<2
        continue
    k
plot(f(0.8))
plot(x)
plot(y)
"#
        ));
        let result = run_historical(&program, &[bar(1.0)]).unwrap();
        for (plot, expected) in result.plots.iter().zip([3.8, 1.8, 3.8]) {
            assert_values_close(&plot.values, &[expected]);
        }
    }
}

#[test]
fn fractional_loop_boundary_is_frozen_before_v6_and_dynamic_in_v6() {
    for (version, expected) in [(5, 4.0), (6, 2.0)] {
        let program = compile(&format!(
            "//@version={version}\nindicator(\"dynamic end\")\nlimit=3.8\ncount=0\nfor i=0.8 to limit\n    count+=1\n    limit:=1.8\nplot(count)\n"
        ));
        let result = run_historical(&program, &[bar(1.0)]).unwrap();
        assert_values_close(&result.plots[0].values, &[expected]);
    }
}

#[test]
fn numeric_for_steps_reject_int_float_mismatch_and_all_zero_steps() {
    for header in [
        "0 to 3 by 0.5",
        "0.8 to 3.8 by 0.0",
        "0.8 to 3.8 by 0",
        "\"x\" to 3",
        "0 to true",
    ] {
        let analysis = analyze_source(&SourceFile::new(
            "invalid-loop.pine",
            format!("//@version=5\nindicator(\"invalid\")\nfor i={header}\n    value=i\n"),
        ));
        assert!(
            analysis.hir.is_none(),
            "{header}: {:?}",
            analysis.diagnostics
        );
        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|d| d.code == "E_LOOP_RANGE_TYPE" || d.code == "E_LOOP_STEP"),
            "{header}: {:?}",
            analysis.diagnostics
        );
    }
}

#[test]
fn integer_for_counter_remains_exact_above_float_precision_and_at_overflow() {
    let program = compile(
        "//@version=6\nindicator(\"exact ints\")\ncount=0\nfor i=9223372036854775805 to 9223372036854775807\n    count+=1\nplot(count)\n",
    );
    let result = run_historical(&program, &[bar(1.0)]).unwrap();
    assert_values_close(&result.plots[0].values, &[3.0]);
}

#[test]
fn strategy_series_defaults_read_current_state_at_each_call() {
    for version in [5, 6] {
        let program = compile(&format!(
            r#"//@version={version}
strategy("state defaults",default_qty_value=2)
f(a=strategy.position_avg_price,p=strategy.position_size)=>
    nz(a)+p
if bar_index==0
    strategy.entry("L",strategy.long)
plot(f())
plot(f(p=7))
plot(f(100))
plot(f(100,7))
"#
        ));
        let result = run_historical(&program, &[bar(10.0), bar(11.0), bar(12.0)]).unwrap();
        for (plot, expected) in result.plots.iter().zip([
            [0.0, 13.0, 13.0],
            [7.0, 18.0, 18.0],
            [100.0, 102.0, 102.0],
            [107.0, 107.0, 107.0],
        ]) {
            assert_values_close(&plot.values, &expected);
        }
    }
}

#[test]
fn strategy_series_defaults_keep_simple_and_reference_type_guards() {
    for default in [
        "simple float a=strategy.position_avg_price",
        "a=label.all",
        "a=strategy.position_size+1",
        "a=ta.sma(close,2)",
    ] {
        let analysis = analyze_source(&SourceFile::new(
            "invalid-default.pine",
            format!("//@version=5\nstrategy(\"invalid\")\nf({default})=>1\nplot(f())\n"),
        ));
        assert!(
            analysis.hir.is_none(),
            "{default}: {:?}",
            analysis.diagnostics
        );
        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|d| d.code.starts_with("E_FUNCTION_DEFAULT")),
            "{default}: {:?}",
            analysis.diagnostics
        );
    }
}

#[test]
fn negative_integer_and_float_steps_fail_instead_of_taking_absolute_value() {
    for version in [4, 5, 6] {
        let declaration = if version == 4 { "study" } else { "indicator" };
        for header in [
            "0 to 3 by -1",
            "3 to 0 by -1",
            "0.8 to 3.8 by -0.5",
            "3.8 to 0.8 by -0.5",
        ] {
            let program = compile(&format!(
                "//@version={version}\n{declaration}(\"negative step\")\nfor i={header}\n    value=i\nplot(close)\n"
            ));
            let error = run_historical(&program, &[bar(1.0)]).unwrap_err();
            assert!(
                error.message.contains("step must be greater than zero"),
                "{version} {header}: {:?}",
                error
            );
        }
    }
}

#[test]
fn float_loop_endpoint_comparisons_use_native_boundary_without_rounding_values() {
    for version in [4, 5, 6] {
        let declaration = if version == 4 { "study" } else { "indicator" };
        let program = compile(&format!(
            "//@version={version}\n{declaration}(\"loop precision\")\na=0\nb=0\nc=0\nfor i=0.1 to 0.3 by 0.1\n    a+=1\nfor j=0.3 to 0.1 by 0.1\n    b+=1\nfor k=0.0 to 0.3 by 0.1\n    c+=1\nplot(a)\nplot(b)\nplot(c)\n"
        ));
        let result = run_historical(&program, &[bar(1.0)]).unwrap();
        for (plot, expected) in result.plots.iter().zip([3.0, 3.0, 4.0]) {
            assert_values_close(&plot.values, &[expected]);
        }
    }
}
