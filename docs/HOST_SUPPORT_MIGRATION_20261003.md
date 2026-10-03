# Host support migration for the 0.3 prerelease

The 0.3 prerelease moves running-alert configuration and delivery policy into
the optional `pine-host-support` crate. Rust applications using these APIs must
change their dependency and imports. Names, serialized fields, error behavior,
and the existing CLI, Python, and WASM entry points remain the same.

The dependency direction is `pine-host-support -> pine-runtime`. The runtime
does not depend on the host support crate and does not compile the delivery
adapter, retry policy, attempt store, HTTP status classification, webhook
configuration, or secret/transport interfaces. Applications can embed
`pine-runtime` alone.

## Ownership

| Runtime core | Optional host support |
|---|---|
| Pine `alert`/`alertcondition` and broker order-fill event generation | Running-alert identity, immutable script snapshot references, event selection, realtime delivery policy |
| `AlertEvent`, `StrategyOrderFillAlertOutput` | `RunningAlertConfig`, `RunningAlertEvaluationError`, event selection and realtime policy enums |
| `render_strategy_order_fill_alert_template` and its template error contract | `render_strategy_order_fill_running_alert`, delivery candidate/dedupe key/sink |
| Host-neutral runtime input and output contracts | Attempt stores, adapter orchestration, retry decisions/records, diagnostics, webhook payloads/requests, secret resolver and transport traits |

The neutral template renderer still takes a template string and a public event
in the core. Running-alert configuration adds application identity and policy,
so its wrapper lives in host support. The host remains responsible for
concrete network clients, secrets, persistence, scheduling, and alert lifecycle.

## Rust import migration

Add the optional dependency when the application needs these helpers:

```toml
[dependencies]
pine-runtime = { path = "../pine-compat-runtime/crates/pine-runtime" }
pine-host-support = { path = "../pine-compat-runtime/crates/pine-host-support" }
```

Before:

```rust,ignore
use pine_runtime::{
    DeliveryCandidate, RunningAlertConfig, WebhookAdapterConfig,
    strategy_order_fill_delivery_candidate,
};
```

After:

```rust,ignore
use pine_host_support::{
    DeliveryCandidate, RunningAlertConfig, WebhookAdapterConfig,
    strategy_order_fill_delivery_candidate,
};
use pine_runtime::{
    StrategyOrderFillAlertOutput, render_strategy_order_fill_alert_template,
};
```

The original public root-level API names are reexported from
`pine_host_support`; external applications should migrate those imports as
shown above. The new crate also exposes `pine_host_support::delivery` and
`pine_host_support::running_alerts`. The core provides no compatibility
reexport, because that would introduce a dependency back into application policy.

CLI running-alert formatting and the Python/WASM
`render_strategy_order_fill_running_alert` helpers now call host support
explicitly. Their public signatures and configuration JSON are preserved. The
runtime's public result/change/profile schemas and Pine execution semantics
are unchanged by the extraction.

The existing running-alert and delivery tests moved with their implementation,
including configuration serialization, message rendering, dedupe, attempt
lifecycles, bounded retries, redaction, secret resolution, and fake transport
behavior. Verification commands for this boundary are:

```sh
cargo test -p pine-host-support
cargo test -p pine-runtime strategy_order_fill_alert_template
cargo test -p pine-cli running_alert
cargo test -p pine-wasm running_alert
cargo check -p pine-python
```

These tests verify the helper contract and dependency migration. Network
transport and durable delivery remain application capabilities.
