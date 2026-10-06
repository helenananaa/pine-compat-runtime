# Optional alertcondition arguments

2026-09-26 local source evidence, continuing the public TASC indicator audit.

The original [TASC Low-Risk ETF indicator](https://www.tradingview.com/script/2S4BqQzQ-TASC-2026-10-A-Low-Risk-ETF-Trading-Strategy/)
uses two-argument `alertcondition` calls. The runtime incorrectly required both
title and message. Both are now optional, while condition remains required and
provided strings retain their constant-string constraints.

The [official alerts documentation](https://www.tradingview.com/pine-script-docs/concepts/alerts/)
specifies an omitted title of `Alert`. A native v6 probe compiled all four forms:
condition alone, condition/title, named condition/message, and explicit empty
message. TradingView's unsent alert draft exposes the expected `Alert`,
`Named only`, `Alert`, and `Explicit empty` conditions.

The native UI uses `Alert Fired!` for both omitted and explicitly empty messages.
This is not evidence for replacing script-level empty strings with that UI
fallback. Runtime events preserve an empty message when omitted, an explicit
empty title/message when supplied, and placeholder substitution for supplied
messages. No remote alert was created and no notification delivery was tested.

The original TASC source now has three remaining diagnostics, down from five:
two missing `plot(linestyle=...)` parameters and display-mask subtraction. It is
not yet numerically qualified and also requires independent benchmark data.

Validation: 6,090 builtins/sema/runtime/CLI tests passed, including optional
argument binding, false-condition suppression, empty strings and placeholders.
The native probe also executes on 4,283 confirmed daily bars with zero local
diagnostics. This is admission/default-contract evidence, not native historical
alert-delivery parity. Fixture: `tests/fixtures/runtime/alertcondition_optional_arguments.pine`.

Ignored local evidence in `.local/continued-popular-20260926`:
`optional-alert-probe-v6.pine`, `optional-alert-default-message-dom.txt`,
`optional-alert-explicit-empty-dom.txt`, `optional-alert-local.json`, and
`test-optional-alert.log`. The draft was cancelled and temporary probe removed.
