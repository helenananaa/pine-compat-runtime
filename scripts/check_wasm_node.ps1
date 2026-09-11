[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Invoke-Checked {
    param(
        [Parameter(Mandatory)] [string] $Command,
        [Parameter()] [string[]] $Arguments = @()
    )

    Write-Host "+ $Command $($Arguments -join ' ')"
    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "command failed with exit code ${LASTEXITCODE}: $Command"
    }
}

$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
        throw "Node.js is required for the wasm smoke gate"
    }

    $metadata = cargo metadata --no-deps --format-version 1 | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata failed with exit code $LASTEXITCODE"
    }
    $targetDir = [string] $metadata.target_directory
    if (-not $targetDir) {
        throw "could not determine the Cargo target directory"
    }

    $installedTargets = @(rustup target list --installed)
    if ($installedTargets -notcontains "wasm32-unknown-unknown") {
        throw "missing Rust target wasm32-unknown-unknown; run: rustup target add wasm32-unknown-unknown"
    }

    $outputDir = Join-Path ([System.IO.Path]::GetTempPath()) (
        "pine-wasm-node-" + [guid]::NewGuid().ToString("N")
    )
    [System.IO.Directory]::CreateDirectory($outputDir) | Out-Null
    try {
        Invoke-Checked cargo @("build", "-p", "pine-wasm", "--target", "wasm32-unknown-unknown")

        $wasm = Join-Path $targetDir "wasm32-unknown-unknown\debug\pine_wasm.wasm"
        if (-not (Test-Path -LiteralPath $wasm -PathType Leaf)) {
            throw "expected wasm artifact was not produced: $wasm"
        }

        $hostLine = rustc -vV | Select-String -Pattern "^host: " | Select-Object -First 1
        if (-not $hostLine) {
            throw "could not determine the native Rust host target"
        }
        $hostTarget = $hostLine.Line.Substring(6)
        Invoke-Checked cargo @(
            "run", "--quiet", "-p", "pine-wasm", "--example", "generate_node_bindings",
            "--target", $hostTarget, "--", $wasm, $outputDir
        )

        $bindings = Join-Path $outputDir "pine_wasm.js"
        if (-not (Test-Path -LiteralPath $bindings -PathType Leaf)) {
            throw "expected Node bindings were not produced: $bindings"
        }
        Invoke-Checked node @("scripts/tests/wasm_node_smoke.cjs", $bindings)
    }
    finally {
        $tempRoot = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
        $resolvedOutput = [System.IO.Path]::GetFullPath($outputDir)
        if (-not $resolvedOutput.StartsWith($tempRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "refusing to remove non-temporary wasm output directory: $resolvedOutput"
        }
        if ([System.IO.Directory]::Exists($resolvedOutput)) {
            [System.IO.Directory]::Delete($resolvedOutput, $true)
        }
    }
}
finally {
    Pop-Location
}
