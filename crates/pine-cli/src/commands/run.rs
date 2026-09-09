use std::{collections::HashMap, fs, sync::Arc};

use pine_runtime::{
    BarUpdate, ChartContext, HistoricalRuntime, InMemoryRequestDataProvider, MagnifierInput,
    RealtimeRuntime, RequestEnvironment, RequestKey, RequestTimeframe, RunningAlertConfig,
    RuntimeProfile, RuntimeResult, input_calls, magnifier_input_from_json,
    public_runtime_profiled_result_json, public_runtime_result_json,
    session_window_input_from_json,
};
use pine_sema::analyze_input;

use crate::bars_csv::parse_bars_csv;
use crate::library_sources::{
    LibrarySourceSpec, analysis_input_from_paths, parse_library_source_spec,
};
use crate::usage;

mod input_overrides;

use input_overrides::{InputOverrideSpec, input_overrides_from_specs, parse_input_override_spec};

pub(crate) fn run(args: Vec<String>) -> Result<(), String> {
    let options = parse_options(&args)?;
    run_with_options(&options)
}

pub(crate) fn run_incremental(args: Vec<String>) -> Result<(), String> {
    let options = parse_options(&args)?;
    run_with_options_in_mode(&options, ExecutionMode::Incremental)
}

pub(crate) fn run_realtime_history(args: Vec<String>) -> Result<(), String> {
    let options = parse_options(&args)?;
    run_with_options_in_mode(&options, ExecutionMode::RealtimeHistory)
}

pub(crate) fn run_realtime_forming(args: Vec<String>) -> Result<(), String> {
    let options = parse_options(&args)?;
    run_with_options_in_mode(&options, ExecutionMode::RealtimeForming)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecutionMode {
    Batch,
    Incremental,
    RealtimeHistory,
    RealtimeForming,
}

#[derive(Debug)]
struct RunOptions {
    path: String,
    bars_path: String,
    magnifier_bars_path: Option<String>,
    session_windows_path: Option<String>,
    execution_times_path: Option<String>,
    chart_context: ChartContext,
    profile: bool,
    request_bars: Vec<RequestBarsSpec>,
    library_sources: Vec<LibrarySourceSpec>,
    input_overrides: Vec<InputOverrideSpec>,
    strategy_alert_template: Option<StrategyAlertTemplateOptions>,
    strategy_running_alert: Option<StrategyRunningAlertOptions>,
}

#[derive(Debug)]
struct StrategyAlertTemplateOptions {
    template: String,
    index: usize,
}

#[derive(Debug)]
struct StrategyRunningAlertOptions {
    config: RunningAlertConfig,
    index: usize,
}

#[derive(Debug)]
struct RequestBarsSpec {
    key: RequestKey,
    path: String,
}

fn run_with_options(options: &RunOptions) -> Result<(), String> {
    run_with_options_in_mode(options, ExecutionMode::Batch)
}

fn run_with_options_in_mode(
    options: &RunOptions,
    execution_mode: ExecutionMode,
) -> Result<(), String> {
    println!(
        "{}",
        run_output_with_options_in_mode(options, execution_mode)?
    );
    Ok(())
}

#[cfg(test)]
fn run_output_with_options(options: &RunOptions) -> Result<String, String> {
    run_output_with_options_in_mode(options, ExecutionMode::Batch)
}

fn run_output_with_options_in_mode(
    options: &RunOptions,
    execution_mode: ExecutionMode,
) -> Result<String, String> {
    if options.profile
        && (options.strategy_alert_template.is_some() || options.strategy_running_alert.is_some())
    {
        return Err("strategy alert rendering cannot be combined with --profile".to_owned());
    }
    if let Some(template) = &options.strategy_alert_template {
        let result = run_result_with_options_in_mode(options, execution_mode)?;
        return render_strategy_alert_template(&result, template);
    }
    if let Some(running_alert) = &options.strategy_running_alert {
        let result = run_result_with_options_in_mode(options, execution_mode)?;
        return render_strategy_running_alert(&result, running_alert);
    }
    run_json_with_options_in_mode(options, execution_mode)
}

#[cfg(test)]
fn run_json_with_options(options: &RunOptions) -> Result<String, String> {
    run_json_with_options_in_mode(options, ExecutionMode::Batch)
}

fn run_json_with_options_in_mode(
    options: &RunOptions,
    execution_mode: ExecutionMode,
) -> Result<String, String> {
    if options.profile {
        return run_profiled_json_with_options_in_mode(options, execution_mode);
    }
    let result = run_result_with_options_in_mode(options, execution_mode)?;
    Ok(public_runtime_result_json(&result))
}

fn run_profiled_json_with_options_in_mode(
    options: &RunOptions,
    execution_mode: ExecutionMode,
) -> Result<String, String> {
    if execution_mode != ExecutionMode::Batch {
        let (result, profile) = run_non_batch_with_options(options, execution_mode)?;
        return Ok(public_runtime_profiled_result_json(&result, &profile));
    }
    let input = analysis_input_from_paths(&options.path, &options.library_sources)?;
    let source = input.root().clone();
    let analysis = analyze_input(&input);
    if !analysis.diagnostics.is_empty() {
        for diagnostic in analysis.diagnostics {
            let line_col = source.line_col(diagnostic.span.start);
            eprintln!(
                "{}:{:?}:{}:{}: {}",
                diagnostic.code,
                diagnostic.severity,
                line_col.line,
                line_col.column,
                diagnostic.message
            );
        }
        return Err("analysis failed".to_owned());
    }
    let Some(hir) = analysis.hir else {
        return Err("analysis did not produce executable HIR".to_owned());
    };

    let bars_text = fs::read_to_string(&options.bars_path)
        .map_err(|err| format!("failed to read {}: {err}", options.bars_path))?;
    let bars = parse_bars_csv(&bars_text)?;
    let request_environment =
        request_environment_from_specs(&options.request_bars, options.chart_context.clone())?;
    let input_calls = input_calls(&hir)
        .into_iter()
        .map(|input| (input.call_site_id, input))
        .collect::<HashMap<_, _>>();
    let input_overrides = input_overrides_from_specs(&options.input_overrides, &input_calls)?;
    let execution_times = execution_times_from_path(options.execution_times_path.as_deref())?;
    let magnifier = magnifier_input_from_path(options.magnifier_bars_path.as_deref())?;
    let session_windows = session_windows_from_path(options.session_windows_path.as_deref())?;
    let mut runtime = HistoricalRuntime::with_request_environment_and_input_overrides(
        &hir,
        request_environment,
        input_overrides,
    );
    if let Some(magnifier) = magnifier {
        runtime = runtime.with_magnifier_input(magnifier);
    }
    if let Some(session_windows) = session_windows {
        runtime = runtime
            .with_session_windows(session_windows)
            .map_err(|err| err.message)?;
    }
    match execution_times.as_deref() {
        Some(execution_times) => runtime.append_bars_with_execution_times(&bars, execution_times),
        None => runtime.append_bars(&bars),
    }
    .map_err(|err| format!("runtime failed: {}", err.message))?;
    Ok(public_runtime_profiled_result_json(
        &runtime.result(),
        &runtime.profile(),
    ))
}

fn run_result_with_options_in_mode(
    options: &RunOptions,
    execution_mode: ExecutionMode,
) -> Result<RuntimeResult, String> {
    if execution_mode != ExecutionMode::Batch {
        return run_non_batch_with_options(options, execution_mode).map(|(result, _)| result);
    }
    let input = analysis_input_from_paths(&options.path, &options.library_sources)?;
    let source = input.root().clone();
    let analysis = analyze_input(&input);
    if !analysis.diagnostics.is_empty() {
        for diagnostic in analysis.diagnostics {
            let line_col = source.line_col(diagnostic.span.start);
            eprintln!(
                "{}:{:?}:{}:{}: {}",
                diagnostic.code,
                diagnostic.severity,
                line_col.line,
                line_col.column,
                diagnostic.message
            );
        }
        return Err("analysis failed".to_owned());
    }
    let Some(hir) = analysis.hir else {
        return Err("analysis did not produce executable HIR".to_owned());
    };

    let bars_text = fs::read_to_string(&options.bars_path)
        .map_err(|err| format!("failed to read {}: {err}", options.bars_path))?;
    let bars = parse_bars_csv(&bars_text)?;
    let request_environment =
        request_environment_from_specs(&options.request_bars, options.chart_context.clone())?;
    let input_calls = input_calls(&hir)
        .into_iter()
        .map(|input| (input.call_site_id, input))
        .collect::<HashMap<_, _>>();
    let input_overrides = input_overrides_from_specs(&options.input_overrides, &input_calls)?;
    let execution_times = execution_times_from_path(options.execution_times_path.as_deref())?;
    let magnifier = magnifier_input_from_path(options.magnifier_bars_path.as_deref())?;
    let session_windows = session_windows_from_path(options.session_windows_path.as_deref())?;
    let mut runtime = HistoricalRuntime::with_request_environment_and_input_overrides(
        &hir,
        request_environment,
        input_overrides,
    );
    if let Some(magnifier) = magnifier {
        runtime = runtime.with_magnifier_input(magnifier);
    }
    if let Some(session_windows) = session_windows {
        runtime = runtime
            .with_session_windows(session_windows)
            .map_err(|err| err.message)?;
    }
    match execution_times.as_deref() {
        Some(execution_times) => runtime.append_bars_with_execution_times(&bars, execution_times),
        None => runtime.append_bars(&bars),
    }
    .map_err(|err| format!("runtime failed: {}", err.message))?;
    Ok(runtime.result())
}

fn run_non_batch_with_options(
    options: &RunOptions,
    execution_mode: ExecutionMode,
) -> Result<(RuntimeResult, RuntimeProfile), String> {
    debug_assert_ne!(execution_mode, ExecutionMode::Batch);
    let input = analysis_input_from_paths(&options.path, &options.library_sources)?;
    let source = input.root().clone();
    let analysis = analyze_input(&input);
    if !analysis.diagnostics.is_empty() {
        for diagnostic in analysis.diagnostics {
            let line_col = source.line_col(diagnostic.span.start);
            eprintln!(
                "{}:{:?}:{}:{}: {}",
                diagnostic.code,
                diagnostic.severity,
                line_col.line,
                line_col.column,
                diagnostic.message
            );
        }
        return Err("analysis failed".to_owned());
    }
    let Some(hir) = analysis.hir else {
        return Err("analysis did not produce executable HIR".to_owned());
    };

    let bars_text = fs::read_to_string(&options.bars_path)
        .map_err(|err| format!("failed to read {}: {err}", options.bars_path))?;
    let bars = parse_bars_csv(&bars_text)?;
    let request_environment =
        request_environment_from_specs(&options.request_bars, options.chart_context.clone())?;
    let input_calls = input_calls(&hir)
        .into_iter()
        .map(|input| (input.call_site_id, input))
        .collect::<HashMap<_, _>>();
    let input_overrides = input_overrides_from_specs(&options.input_overrides, &input_calls)?;
    let execution_times = execution_times_from_path(options.execution_times_path.as_deref())?;
    let magnifier = magnifier_input_from_path(options.magnifier_bars_path.as_deref())?;
    let session_windows = session_windows_from_path(options.session_windows_path.as_deref())?;

    if let Some(execution_times) = &execution_times
        && execution_times.len() != bars.len()
    {
        return Err(format!(
            "runtime failed: execution timestamp count {} does not match bar count {}",
            execution_times.len(),
            bars.len()
        ));
    }

    match execution_mode {
        ExecutionMode::Batch => unreachable!("batch execution uses the batch runtime path"),
        ExecutionMode::Incremental => {
            let mut runtime = HistoricalRuntime::with_request_environment_and_input_overrides(
                &hir,
                request_environment,
                input_overrides,
            );
            if let Some(magnifier) = magnifier {
                runtime = runtime.with_magnifier_input(magnifier);
            }
            if let Some(session_windows) = session_windows.clone() {
                runtime = runtime
                    .with_session_windows(session_windows)
                    .map_err(|err| err.message)?;
            }
            match execution_times.as_deref() {
                Some(execution_times) => {
                    runtime.append_bars_with_execution_times(&bars, execution_times)
                }
                None => runtime.append_bars(&bars),
            }
            .map_err(|err| format!("runtime failed: {}", err.message))?;
            Ok((runtime.result(), runtime.profile()))
        }
        ExecutionMode::RealtimeHistory => {
            let mut runtime = RealtimeRuntime::with_request_environment_and_input_overrides(
                &hir,
                request_environment,
                input_overrides,
            );
            if let Some(magnifier) = magnifier.clone() {
                runtime = runtime.with_magnifier_input(magnifier);
            }
            if let Some(session_windows) = session_windows.clone() {
                runtime = runtime
                    .with_session_windows(session_windows)
                    .map_err(|err| err.message)?;
            }
            runtime
                .prepare_magnifier_chart_bar_count(bars.len())
                .map_err(|err| format!("runtime failed: {}", err.message))?;
            for (index, bar) in bars.iter().copied().enumerate() {
                match execution_times.as_ref().map(|values| values[index]) {
                    Some(execution_time) => runtime
                        .update_with_execution_time(BarUpdate::historical(bar), execution_time),
                    None => runtime.update(BarUpdate::historical(bar)),
                }
                .map_err(|err| format!("runtime failed: {}", err.message))?;
            }
            Ok((runtime.confirmed_result(), runtime.confirmed_profile()))
        }
        ExecutionMode::RealtimeForming => {
            let (last, history) = bars.split_last().ok_or_else(|| {
                "runtime failed: realtime forming requires at least one bar".to_owned()
            })?;
            let mut runtime = RealtimeRuntime::with_request_environment_and_input_overrides(
                &hir,
                request_environment,
                input_overrides,
            );
            if let Some(magnifier) = magnifier {
                runtime = runtime.with_magnifier_input(magnifier);
            }
            if let Some(session_windows) = session_windows.clone() {
                runtime = runtime
                    .with_session_windows(session_windows)
                    .map_err(|err| err.message)?;
            }
            runtime
                .prepare_magnifier_chart_bar_count(history.len())
                .map_err(|err| format!("runtime failed: {}", err.message))?;
            for (index, bar) in history.iter().copied().enumerate() {
                match execution_times.as_ref().map(|values| values[index]) {
                    Some(execution_time) => runtime
                        .update_with_execution_time(BarUpdate::historical(bar), execution_time),
                    None => runtime.update(BarUpdate::historical(bar)),
                }
                .map_err(|err| format!("runtime failed: {}", err.message))?;
            }
            let confirmed_execution_time = execution_times
                .as_ref()
                .and_then(|values| values.last().copied());
            let mutated = pine_runtime::Bar {
                time: last.time,
                open: last.open + 3.0,
                high: last.high + 9.0,
                low: last.low - 7.0,
                close: last.close + 5.0,
                volume: last.volume + 11.0,
            };
            for (update, execution_time) in [
                (
                    BarUpdate::forming(mutated),
                    confirmed_execution_time.map(|value| value.saturating_sub(2)),
                ),
                (
                    BarUpdate::forming(*last),
                    confirmed_execution_time.map(|value| value.saturating_sub(1)),
                ),
                (BarUpdate::confirmed(*last), confirmed_execution_time),
            ] {
                match execution_time {
                    Some(execution_time) => {
                        runtime.update_with_execution_time(update, execution_time)
                    }
                    None => runtime.update(update),
                }
                .map_err(|err| format!("runtime failed: {}", err.message))?;
            }
            Ok((runtime.confirmed_result(), runtime.confirmed_profile()))
        }
    }
}

fn render_strategy_alert_template(
    result: &RuntimeResult,
    template: &StrategyAlertTemplateOptions,
) -> Result<String, String> {
    let strategy = result
        .strategy
        .as_ref()
        .ok_or_else(|| "runtime result does not contain strategy alerts".to_owned())?;
    let alert = strategy.alerts.get(template.index).ok_or_else(|| {
        format!(
            "strategy alert index {} is out of range for {} alert(s)",
            template.index,
            strategy.alerts.len()
        )
    })?;
    pine_runtime::render_strategy_order_fill_alert_template(&template.template, alert)
        .map_err(|err| err.to_string())
}

fn render_strategy_running_alert(
    result: &RuntimeResult,
    running_alert: &StrategyRunningAlertOptions,
) -> Result<String, String> {
    let strategy = result
        .strategy
        .as_ref()
        .ok_or_else(|| "runtime result does not contain strategy alerts".to_owned())?;
    let alert = strategy.alerts.get(running_alert.index).ok_or_else(|| {
        format!(
            "strategy alert index {} is out of range for {} alert(s)",
            running_alert.index,
            strategy.alerts.len()
        )
    })?;
    pine_runtime::render_strategy_order_fill_running_alert(&running_alert.config, alert)
        .map_err(|err| err.to_string())
}

fn parse_options(args: &[String]) -> Result<RunOptions, String> {
    let Some(path) = args.first() else {
        return Err(usage());
    };
    let mut options = RunOptions {
        path: path.clone(),
        bars_path: String::new(),
        magnifier_bars_path: None,
        session_windows_path: None,
        execution_times_path: None,
        chart_context: ChartContext::default(),
        profile: false,
        request_bars: Vec::new(),
        library_sources: Vec::new(),
        input_overrides: Vec::new(),
        strategy_alert_template: None,
        strategy_running_alert: None,
    };
    let mut strategy_alert_template = None;
    let mut strategy_running_alert_template = None;
    let mut running_alert_script_snapshot_id = None;
    let mut running_alert_symbol = None;
    let mut running_alert_timeframe = None;
    let mut strategy_alert_index = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--bars" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                options.bars_path = value.clone();
            }
            "--magnifier-bars" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                if value.trim().is_empty() {
                    return Err("magnifier bars path must not be empty".to_owned());
                }
                options.magnifier_bars_path = Some(value.clone());
            }
            "--session-windows" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                if value.trim().is_empty() {
                    return Err("session windows path must not be empty".to_owned());
                }
                options.session_windows_path = Some(value.clone());
            }
            "--execution-times" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                if value.trim().is_empty() {
                    return Err("execution times path must not be empty".to_owned());
                }
                options.execution_times_path = Some(value.clone());
            }
            "--profile" => {
                options.profile = true;
            }
            "--chart-symbol" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                if value.trim().is_empty() {
                    return Err("chart symbol must not be empty".to_owned());
                }
                options.chart_context = options.chart_context.clone().with_symbol(value.trim());
            }
            "--chart-timeframe" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                let timeframe =
                    RequestTimeframe::parse(value).map_err(|error| error.to_string())?;
                options.chart_context = options.chart_context.clone().with_timeframe(timeframe);
            }
            "--chart-quantity-precision" => {
                index += 1;
                let value = args.get(index).ok_or_else(usage)?;
                let precision = value.parse::<u32>().map_err(|_| {
                    "chart quantity precision must be an integer between 0 and 9".to_owned()
                })?;
                options.chart_context = options
                    .chart_context
                    .with_quantity_precision(precision)
                    .map_err(str::to_owned)?;
            }
            "--chart-price-grid" => {
                index += 1;
                let value = args.get(index).ok_or_else(usage)?;
                let (min_move, price_scale) = value
                    .split_once('/')
                    .ok_or("chart price grid must be MIN_MOVE/PRICE_SCALE")?;
                options.chart_context = options
                    .chart_context
                    .clone()
                    .with_price_grid(
                        min_move
                            .parse()
                            .map_err(|_| "chart minMove must be a positive integer")?,
                        price_scale
                            .parse()
                            .map_err(|_| "chart priceScale must be a positive integer")?,
                    )
                    .map_err(str::to_owned)?;
            }
            "--render-strategy-order-alert-template" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                strategy_alert_template = Some(value.clone());
            }
            "--render-strategy-running-alert" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                strategy_running_alert_template = Some(value.clone());
            }
            "--running-alert-script-snapshot-id" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                running_alert_script_snapshot_id = Some(value.clone());
            }
            "--running-alert-symbol" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                running_alert_symbol = Some(value.clone());
            }
            "--running-alert-timeframe" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                running_alert_timeframe = Some(value.clone());
            }
            "--strategy-alert-index" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                strategy_alert_index = Some(parse_strategy_alert_index(value)?);
            }
            "--request-bars" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                options.request_bars.push(parse_request_bars_spec(value)?);
            }
            "--library-source" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                options
                    .library_sources
                    .push(parse_library_source_spec(value)?);
            }
            "--input-override" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(usage());
                };
                options
                    .input_overrides
                    .push(parse_input_override_spec(value)?);
            }
            _ => return Err(usage()),
        }
        index += 1;
    }
    if options.bars_path.is_empty() {
        return Err(usage());
    }
    if strategy_alert_template.is_some() && strategy_running_alert_template.is_some() {
        return Err(usage());
    }
    options.strategy_alert_template = match (strategy_alert_template, &strategy_alert_index) {
        (None, _) => None,
        (Some(template), Some(index)) => Some(StrategyAlertTemplateOptions {
            template,
            index: *index,
        }),
        (Some(_), None) => return Err(usage()),
    };
    options.strategy_running_alert = if options.strategy_alert_template.is_some() {
        if strategy_running_alert_template.is_some()
            || running_alert_script_snapshot_id.is_some()
            || running_alert_symbol.is_some()
            || running_alert_timeframe.is_some()
        {
            return Err(usage());
        }
        None
    } else {
        match (
            strategy_running_alert_template,
            running_alert_script_snapshot_id,
            running_alert_symbol,
            running_alert_timeframe,
            strategy_alert_index,
        ) {
            (None, None, None, None, None) => None,
            (
                Some(template),
                Some(script_snapshot_id),
                Some(symbol),
                Some(timeframe),
                Some(index),
            ) => Some(StrategyRunningAlertOptions {
                config: RunningAlertConfig::new_strategy_order_fills(
                    script_snapshot_id,
                    symbol,
                    timeframe,
                    template,
                ),
                index,
            }),
            _ => return Err(usage()),
        }
    };
    Ok(options)
}

fn parse_strategy_alert_index(value: &str) -> Result<usize, String> {
    value
        .parse()
        .map_err(|_| "strategy alert index must be a non-negative integer".to_owned())
}

fn execution_times_from_path(path: Option<&str>) -> Result<Option<Vec<i64>>, String> {
    let Some(path) = path else {
        return Ok(None);
    };
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read execution times {path}: {err}"))?;
    parse_execution_times(&text).map(Some)
}

fn parse_execution_times(text: &str) -> Result<Vec<i64>, String> {
    let mut execution_times = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let value = line.trim();
        if value.is_empty()
            || (index == 0 && matches!(value, "execution_time" | "execution_timestamp" | "timenow"))
        {
            continue;
        }
        let execution_time = value.parse::<i64>().map_err(|_| {
            format!(
                "invalid execution timestamp `{value}` on line {}: expected one integer millisecond timestamp per line",
                index + 1
            )
        })?;
        execution_times.push(execution_time);
    }
    Ok(execution_times)
}

fn parse_request_bars_spec(spec: &str) -> Result<RequestBarsSpec, String> {
    let Some((key, path)) = spec.split_once('=') else {
        return Err("request bars must use SYMBOL:TIMEFRAME=path.csv".to_owned());
    };
    if path.trim().is_empty() {
        return Err("request bars path must not be empty".to_owned());
    }
    let Some((symbol, timeframe)) = key.rsplit_once(':') else {
        return Err("request bars key must use SYMBOL:TIMEFRAME".to_owned());
    };
    if symbol.trim().is_empty() {
        return Err("request bars symbol must not be empty".to_owned());
    }
    let timeframe = RequestTimeframe::parse(timeframe).map_err(|err| err.to_string())?;
    Ok(RequestBarsSpec {
        key: RequestKey::new(symbol.trim(), timeframe),
        path: path.to_owned(),
    })
}

fn request_environment_from_specs(
    specs: &[RequestBarsSpec],
    chart_context: ChartContext,
) -> Result<RequestEnvironment, String> {
    if specs.is_empty() {
        return Ok(RequestEnvironment::default().for_chart(chart_context));
    }

    let mut streams = Vec::with_capacity(specs.len());
    for spec in specs {
        let text = fs::read_to_string(&spec.path)
            .map_err(|err| format!("failed to read {}: {err}", spec.path))?;
        streams.push((spec.key.clone(), parse_bars_csv(&text)?));
    }
    let provider =
        InMemoryRequestDataProvider::from_streams(streams).map_err(|err| err.to_string())?;
    Ok(RequestEnvironment::new(chart_context, Arc::new(provider)))
}

fn magnifier_input_from_path(path: Option<&str>) -> Result<Option<MagnifierInput>, String> {
    let Some(path) = path else {
        return Ok(None);
    };
    let text = fs::read_to_string(path).map_err(|err| format!("failed to read {path}: {err}"))?;
    magnifier_input_from_json(&text).map(Some)
}

fn session_windows_from_path(
    path: Option<&str>,
) -> Result<Option<pine_runtime::SessionWindowInput>, String> {
    let Some(path) = path else {
        return Ok(None);
    };
    let text = fs::read_to_string(path).map_err(|err| format!("failed to read {path}: {err}"))?;
    session_window_input_from_json(&text).map(Some)
}

#[cfg(test)]
mod tests;
