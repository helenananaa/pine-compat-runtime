//! Offline sustained-tail probe. Process measurements stay outside the core.
use std::{collections::BTreeMap, io, time::Instant};

#[path = "benchmark_support/memory.rs"]
mod memory;

use pine_runtime::{
    Bar, BarUpdate, HistoricalRuntime, RealtimeRuntime, public_runtime_result_json,
};
use pine_sema::{AnalysisInput, analyze_input};
use pine_syntax::{Severity, SourceFile};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Input {
    source: String,
    #[serde(default)]
    libraries: BTreeMap<String, String>,
    bars: Vec<InputBar>,
    history_bars: usize,
    repetitions: usize,
    replacements_per_bar: usize,
    magnifier: Option<Value>,
}

#[derive(Deserialize)]
struct InputBar {
    time: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

type Timings = BTreeMap<&'static str, Vec<f64>>;

fn timed<T>(times: &mut Timings, phase: &'static str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = f();
    times
        .entry(phase)
        .or_default()
        .push(start.elapsed().as_secs_f64() * 1000.0);
    result
}

fn memory_checkpoint(phase: &str, repetition: usize) -> Value {
    let reading = memory::read();
    json!({"phase":phase,"repetition":repetition,"source":memory::SOURCE,
        "peakRssKiB":reading.peak_resident_kib,"peakCommitKiB":reading.peak_commit_kib})
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input: Input = serde_json::from_reader(io::stdin())?;
    if input.history_bars == 0
        || input.history_bars >= input.bars.len()
        || input.repetitions < 2
        || input.replacements_per_bar == 0
    {
        return Err(
            "need nonempty history and tail, repetitions >= 2 and replacementsPerBar >= 1".into(),
        );
    }
    let bars: Vec<Bar> = input
        .bars
        .iter()
        .map(|b| Bar {
            time: b.time,
            open: b.open,
            high: b.high,
            low: b.low,
            close: b.close,
            volume: b.volume,
        })
        .collect();
    if bars.windows(2).any(|pair| pair[0].time >= pair[1].time) {
        return Err("sustained input timestamps must be strictly increasing".into());
    }
    let analysis_input = AnalysisInput::with_library_sources(
        SourceFile::new("long-session.pine", input.source),
        input
            .libraries
            .into_iter()
            .map(|(key, text)| {
                let source = SourceFile::new(key.clone(), text);
                (key, source)
            })
            .collect(),
    )?;
    let mut times = Timings::new();
    let analysis = timed(&mut times, "compile", || analyze_input(&analysis_input));
    if analysis
        .diagnostics
        .iter()
        .any(|d| d.severity == Severity::Error)
    {
        return Err(format!("analysis failed: {:?}", analysis.diagnostics).into());
    }
    let hir = analysis.hir.ok_or("no executable HIR")?;
    let magnifier = input
        .magnifier
        .as_ref()
        .map(|v| pine_runtime::magnifier_input_from_json(&v.to_string()))
        .transpose()?;
    if magnifier.is_some() != hir.strategy_settings.use_bar_magnifier {
        return Err("probe requires Magnifier declaration and intrabar input to agree; fallback is not a magnifier measurement".into());
    }
    let make_historical = || {
        let mut runtime = HistoricalRuntime::new(&hir);
        if let Some(magnifier) = &magnifier {
            runtime = runtime.with_magnifier_input(magnifier.clone());
            runtime.prepare_magnifier_chart_bar_count(bars.len())?;
        }
        Ok::<_, pine_runtime::RuntimeError>(runtime)
    };
    let (history, tail) = bars.split_at(input.history_bars);
    let mut checkpoints = Vec::new();
    let mut expected_historical: Option<String> = None;
    let mut expected_live: Option<String> = None;
    for repetition in 0..input.repetitions {
        {
            let mut runtime = make_historical().map_err(|error| error.message)?;
            timed(&mut times, "historySeed", || runtime.append_bars(history))
                .map_err(|error| error.message)?;
            checkpoints.push(memory_checkpoint("afterHistorySeed", repetition));
            for bar in tail {
                timed(&mut times, "tailAppend", || runtime.append_bar(*bar))
                    .map_err(|error| error.message)?;
            }
            checkpoints.push(memory_checkpoint("afterTailAppend", repetition));
            if runtime.profile().bars != bars.len() {
                return Err(
                    "historical runtime did not process the complete history and tail".into(),
                );
            }
            let snapshot = timed(&mut times, "resultSnapshot", || runtime.result());
            let rendered = timed(&mut times, "outputSerialization", || {
                public_runtime_result_json(&snapshot)
            });
            let mut control = make_historical().map_err(|error| error.message)?;
            control.append_bars(&bars).map_err(|error| error.message)?;
            if rendered != public_runtime_result_json(&control.result()) {
                return Err("seed plus tail differs from full historical batch".into());
            }
            if expected_historical
                .as_ref()
                .is_some_and(|value| value != &rendered)
            {
                return Err("historical tail result changed between repetitions".into());
            }
            expected_historical = Some(rendered);
        }
        if magnifier.is_none() {
            let mut runtime = RealtimeRuntime::new(&hir);
            timed(&mut times, "liveSeed", || runtime.seed_historical(history))
                .map_err(|error| error.message)?;
            checkpoints.push(memory_checkpoint("afterLiveSeed", repetition));
            for bar in tail {
                let mut initial = *bar;
                initial.high = initial.open;
                initial.low = initial.open;
                initial.close = initial.open;
                initial.volume = 0.0;
                std::hint::black_box(
                    timed(&mut times, "formingInitial", || {
                        runtime.update(BarUpdate::forming(initial))
                    })
                    .map_err(|error| error.message)?,
                );
                for replacement in 0..input.replacements_per_bar {
                    let mut changed = *bar;
                    changed.close = if replacement % 2 == 0 {
                        bar.high
                    } else {
                        bar.low
                    };
                    std::hint::black_box(
                        timed(&mut times, "formingReplace", || {
                            runtime.update(BarUpdate::forming(changed))
                        })
                        .map_err(|error| error.message)?,
                    );
                }
                std::hint::black_box(
                    timed(&mut times, "formingConfirm", || {
                        runtime.update(BarUpdate::confirmed(*bar))
                    })
                    .map_err(|error| error.message)?,
                );
            }
            checkpoints.push(memory_checkpoint("afterLiveTail", repetition));
            if runtime.confirmed_profile().bars != bars.len() {
                return Err("live runtime did not confirm the complete tail".into());
            }
            let snapshot = timed(&mut times, "liveResultSnapshot", || {
                runtime.confirmed_result()
            });
            let rendered = timed(&mut times, "liveOutputSerialization", || {
                public_runtime_result_json(&snapshot)
            });
            if expected_live
                .as_ref()
                .is_some_and(|value| value != &rendered)
            {
                return Err("live tail sequence changed between repetitions".into());
            }
            expected_live = Some(rendered);
        }
    }
    checkpoints.push(memory_checkpoint(
        "afterVerification",
        input.repetitions - 1,
    ));
    println!(
        "{}",
        json!({
            "schemaVersion":1,"historyBars":history.len(),"tailBars":tail.len(),
            "repetitions":input.repetitions,"replacementsPerBar":input.replacements_per_bar,
            "timingsMs":times,"memoryCheckpoints":checkpoints,
            "memoryScope":"process cumulative peaks including input, all phases and verification; before report rendering",
            "formingTimingScope":"update includes returned full snapshot; snapshot destruction excluded",
            "formingExecutesScript":if magnifier.is_none(){Some(hir.script_mode != pine_ir::ScriptMode::Strategy || hir.strategy_settings.calc_on_every_tick)}else{None},
            "correctness":{"batchEqualsTail":true,"repeatedHistoricalStable":true,
                "repeatedLiveStable":if magnifier.is_none(){Some(true)}else{None}},
            "historicalResult":expected_historical,"liveResult":expected_live,
            "realtimeExclusion":if magnifier.is_some(){Some("historical-only magnifier input")}else{None}
        })
    );
    Ok(())
}
