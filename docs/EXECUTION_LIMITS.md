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
