# Deterministic execution limits

Every chart-bar execution has an `ExecutionLimits` allowance. Defaults are
10,000,000 evaluation steps and 1,000,000 total loop iterations. Expressions,
statements and explicit loop preparation consume steps. Iterations are shared
across nested for, while and foreach loops and across requested evaluations.
Requested history does not receive a fresh allowance for each requested bar;
restored request checkpoints also inherit the current chart execution budget.

Each ordinary for and while loop retains an additional 100,000-iteration cap.
Pine v6 dynamic for boundaries are still recalculated after each iteration.
Exhaustion returns a deterministic `RuntimeError`, rather than waiting for a
wall-clock timeout. Built-in kernels have collection/input limits; evaluation
steps do not represent their CPU instruction counts. A host can separately
manage wall-clock deadlines or process lifecycle outside the interpreter.

Rust hosts can use `HistoricalRuntime::with_execution_limits` or
`RealtimeRuntime::with_execution_limits` and inspect `execution_limits()`.
Zero is a valid allowance and prevents the corresponding work. Configuration
is retained by realtime clones, seed and replay. The allowance resets once per
chart update, not per strategy fill callback. Standard bindings use defaults.

An execution error invalidates a low-level HistoricalRuntime for further
execution; see the failure lifecycle in EXECUTION_SEMANTICS.md. Configuration
changes do not revive an invalidated runtime. RealtimeRuntime evaluates a
candidate and keeps its earlier valid state when the candidate fails.

Loop control is typed execution state generated only by reached break/continue
nodes. Public RuntimeError text is never interpreted as a control signal;
runtime.error preserves every user-supplied string, including former internal
sentinel names. Public `RuntimeError { message }` construction remains valid.

## Logical valuewhen event limits

`ValueWhenLimits { max_retained_values: Some(limit) }` sets an optional aggregate
limit on `ta.valuewhen` events. The default is `None`, which keeps the existing
per-call-site retention rules and permits more than one million events across
multiple call sites. One retained Pine value counts as one event, including
`na`; the value's payload size does not change that count.

The limit covers local histories, retained requested-context checkpoints, and
requested evaluators while they execute. A replacement checkpoint releases the
old checkpoint's allowance before evaluating its replacement. Temporary request
evaluators spend the allowance while running and disappear from the retained
count when discarded. Confirmed and forming states, and independent runtime
clones, each have their own allowance. This contract does not enable nested
Pine request expressions that semantic analysis currently rejects.

Rust exposes `set_valuewhen_limits`, `with_valuewhen_limits`,
`valuewhen_limits`, and `valuewhen_retained_values` on historical and realtime
runtimes. Realtime also exposes `confirmed_valuewhen_retained_values`.
Python realtime sessions expose `set_valuewhen_limit(limit=None)` and the
`valuewhen_limit`, `valuewhen_retained_values`, and
`confirmed_valuewhen_retained_values` properties. WASM realtime sessions expose
`setValueWhenLimit(limit)`, `valueWhenLimit()`, `valueWhenRetainedValues()`, and
`confirmedValueWhenRetainedValues()`; null/undefined removes the limit.
Bindings reject booleans, fractional values, negative values, and values that
do not fit the platform. WASM additionally requires a safe JavaScript integer.

Setting a limit below current usage returns `E_VALUEWHEN_BUDGET` and leaves the
configuration and Pine state unchanged. An execution that exceeds the limit
returns the same error family. Realtime rejects the candidate without advancing
its result, delta cache, or revision; the historical failure lifecycle above
still applies. Seed/replay retain the configuration and recompute the count.

This is an event-count limit. It does not bound payload bytes, shared physical
leaves, rollback copies, request output caches, other collections, retained
output, or process RSS. Hosts still own external resource and process policy.

## Collection and Matrix Resource Limits

`ResourceLimits` adds independently optional per-chart-bar logical collection
allocation/copy bytes and matrix work units. Defaults are 64 MiB and 100,000,000
units respectively; Rust historical and realtime callers use
`with_resource_limits`, and `None` disables an allowance. Preparation of
intrabar persistence, requested evaluators and strategy fill passes share the
same execution allowance. Exhaustion returns `E_RESOURCE_BUDGET` without
publishing a failed realtime candidate. This is not an allocator, process RSS
or total retained-heap limit. See [Execution semantics](EXECUTION_SEMANTICS.md#deterministic-resource-allowances)
for covered numerical kernels and state lifetimes.
