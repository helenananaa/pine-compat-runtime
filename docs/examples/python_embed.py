"""Minimal Python embedding path for the local 0.3.0-rc.1 candidate."""
from __future__ import annotations

import json

import pine_compat

SOURCE = """//@version=6
indicator("python embed")
factor = input.float(2.0, "Scale")
plot(close * factor, "scaled")
plot(timenow, "clock")
"""

BARS = [
    {"time": 0, "open": 10.0, "high": 10.0, "low": 10.0, "close": 10.0, "volume": 1.0},
    {"time": 60_000, "open": 11.0, "high": 11.0, "low": 11.0, "close": 11.0, "volume": 1.0},
]


def main() -> None:
    print("version", pine_compat.__version__)
    program = pine_compat.compile_script(SOURCE)
    requirements = program.host_requirements()
    clocks = [1_000, 2_000]
    historical = program.run(BARS, execution_times=clocks)
    session = program.realtime_session()
    session.seed(BARS, execution_times=clocks)
    forming = {"time": 120_000, "open": 12.0, "high": 12.0, "low": 12.0, "close": 12.0, "volume": 1.0}
    try:
        session.update_forming(forming)
        raise SystemExit("missing clock must fail")
    except ValueError as error:
        missing_clock = str(error)
    preview = session.update_forming(forming, execution_time=3_000)
    confirmed = session.update_confirmed(
        {"time": 120_000, "open": 12.0, "high": 13.0, "low": 12.0, "close": 13.0, "volume": 1.0},
        execution_time=4_000,
    )
    print(
        json.dumps(
            {
                "version": pine_compat.__version__,
                "clock": requirements["execution"]["clock"],
                "historicalScaled": historical["plots"][0]["values"],
                "missingClock": missing_clock,
                "previewScaled": preview["plots"][0]["values"],
                "confirmedScaled": confirmed["plots"][0]["values"],
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
