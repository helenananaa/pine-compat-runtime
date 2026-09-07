# Agent Instructions

## Project Scope

`pine-compat-runtime` is a pure Pine interpreter/runtime core. It exists in
part to power CandleScope, but it must remain independently usable and must not
depend on CandleScope or any other concrete host application.

The project owns Pine language and runtime semantics, including:

- parsing, semantic analysis, and execution;
- series values, history, state, incremental updates, and realtime behavior;
- deterministic strategy and broker-emulator semantics;
- order lifecycles, fills, positions, margin, and risk rules; and
- stable, host-neutral input/output contracts and extension interfaces needed
  to embed the runtime.

The project does not own host integration or application infrastructure,
including:

- market-data acquisition, subscriptions, or persistence;
- users, tenants, authentication, authorization, or billing;
- schedulers, workers, actors, distributed execution, or service orchestration;
- databases, caches, network APIs, monitoring, or deployment concerns; and
- CandleScope-specific adapters, domain objects, services, or configuration.

## Architectural Boundary

Implement functionality here when it defines Pine language behavior or
deterministic runtime semantics. Implement it in the host or an external
adapter when it supplies external data, resources, lifecycle management, or
application policy.

The core may define host-neutral traits, callbacks, data structures, and error
or capability contracts. Concrete integrations belong outside this repository.
Missing host capabilities must be represented explicitly through those
contracts or as unsupported behavior; do not make the core reach directly into
host services.

Use CandleScope requirements to inform the runtime's capabilities and ABI, but
do not introduce CandleScope dependencies into the interpreter core. New core
behavior should remain deterministic and testable without starting CandleScope,
connecting to a network, or provisioning external infrastructure.

When ownership is unclear, apply this rule:

> Pine language and execution semantics belong here; providing external
> capabilities required by those semantics belongs to the host adapter.
