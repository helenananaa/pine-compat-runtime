# Compatibility, Legal, and Branding Boundaries

This document defines the project's clean-room boundaries and user-facing
compatibility language.

It is an engineering policy, not legal advice.

## Project Positioning

Preferred wording:

```text
Pine-compatible indicator runtime
Pine-style indicator scripting subset
clean-room Pine-compatible runtime
```

Avoid wording such as:

```text
official Pine Script compiler
TradingView runtime
TradingView-compatible compiler
full Pine Script implementation
```

## Non-Affiliation Notice

The README, package metadata, and documentation should include:

```text
This project is not affiliated with, endorsed by, or sponsored by TradingView.
Pine Script is a trademark of its respective owner.
```

## Clean-Room Implementation Rules

Allowed:

- Implement behavior from public documentation.
- Write original parser, analyzer, runtime, tests, examples, and docs.
- Use user-owned scripts or permissively licensed scripts as fixtures.
- Compare numerical behavior against documented formulas.

Not allowed:

- Copy TradingView compiler or runtime code.
- Use private TradingView APIs.
- Scrape or redistribute TradingView market data.
- Copy substantial official documentation text into this repository.
- Copy proprietary or protected third-party scripts.
- Reproduce TradingView UI, icons, branding, or error text.

## Test Fixture Policy

Fixtures should be one of:

- Original scripts written for this project.
- Scripts with explicit compatible licenses.
- Minimal snippets created to test a language feature.

Every non-original fixture should include license and source metadata.

## Compatibility Claims

The project should claim compatibility by feature and version, not by vague
total compatibility.

Good:

```text
Supports a v5/v6-style indicator subset including `indicator`, `input.*`,
common `ta.*` functions, history references, and plot output.
```

Bad:

```text
Runs all Pine scripts exactly like TradingView.
```

## Diagnostics Policy

Unsupported features should be reported explicitly:

```text
`request.security` supports the admitted same-context and host-provided
expression subset over injected bars. Lower and equal timeframes are accepted;
higher fixed timeframes must be integer multiples of the chart timeframe,
while calendar months use calendar boundaries. Forming lower-timeframe
requests require explicitly received current-period intrabars. General
multi-timeframe data loading remains a host responsibility.
```

This is both better UX and a better compatibility boundary.

## Data Policy

The runtime should accept caller-provided OHLCV data. It should not fetch data
from TradingView or any other provider by default.

Host applications may provide their own data adapters, but those adapters are
outside the core runtime.
