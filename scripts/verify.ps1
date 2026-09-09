[CmdletBinding()]
param(
    [string] $Python = "python"
)

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

function Initialize-MsvcEnvironment {
    $hostLine = rustc -vV | Select-String -Pattern "^host: " | Select-Object -First 1
    if (-not $hostLine -or $hostLine.Line -notlike "*-pc-windows-msvc") {
        return
    }
    if (Get-Command link.exe -ErrorAction SilentlyContinue) {
        return
    }

    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path -LiteralPath $vswhere -PathType Leaf)) {
        throw "Visual Studio locator not found: $vswhere"
    }
    $installation = & $vswhere -latest -products * `
        -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
        -property installationPath
    if ($LASTEXITCODE -ne 0 -or -not $installation) {
        throw "Visual Studio C++ Build Tools were not found"
    }
    $installation = $installation | Select-Object -First 1
    $devShell = Join-Path $installation "Common7\Tools\Microsoft.VisualStudio.DevShell.dll"
    if (-not (Test-Path -LiteralPath $devShell -PathType Leaf)) {
        throw "Visual Studio Developer PowerShell module not found: $devShell"
    }
    Import-Module $devShell
    Enter-VsDevShell -VsInstallPath $installation -SkipAutomaticLocation `
        -DevCmdArguments "-arch=amd64"
}

$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    Initialize-MsvcEnvironment
    Invoke-Checked cargo @("fmt", "--check")
    Invoke-Checked cargo @("clippy", "--workspace", "--all-targets", "--", "-D", "warnings")
    Invoke-Checked cargo @("test", "--workspace")
    Invoke-Checked $Python @("scripts/check_structure.py")
    Invoke-Checked $Python @("-m", "unittest", "discover", "-s", "scripts/tests", "-p", "test_*.py")
    Invoke-Checked $Python @("scripts/check_host_parity.py")
    Invoke-Checked powershell.exe @(
        "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "scripts/check_wasm_node.ps1"
    )

    $gateRoot = Join-Path ([System.IO.Path]::GetTempPath()) (
        "pine-python-release-gate-" + [guid]::NewGuid().ToString("N")
    )
    [System.IO.Directory]::CreateDirectory($gateRoot) | Out-Null
    try {
        $wheelOutputDir = Join-Path $gateRoot "wheels"
        $wheelTestVenv = Join-Path $gateRoot "venv"
        [System.IO.Directory]::CreateDirectory($wheelOutputDir) | Out-Null

        Invoke-Checked maturin @(
            "build", "--manifest-path", "crates/pine-python/Cargo.toml", "--out", $wheelOutputDir
        )
        $wheels = @(Get-ChildItem -LiteralPath $wheelOutputDir -Filter "*.whl" -File)
        if ($wheels.Count -ne 1) {
            throw "expected exactly one freshly built wheel in $wheelOutputDir"
        }

        Invoke-Checked $Python @("-m", "venv", "--system-site-packages", $wheelTestVenv)
        $venvPython = Join-Path $wheelTestVenv "Scripts\python.exe"
        Invoke-Checked $venvPython @(
            "-m", "pip", "install", "--no-deps", "--force-reinstall", $wheels[0].FullName
        )
        Invoke-Checked $venvPython @("-m", "pytest", "python/tests")
    }
    finally {
        $tempRoot = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
        $resolvedGate = [System.IO.Path]::GetFullPath($gateRoot)
        if (-not $resolvedGate.StartsWith($tempRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "refusing to remove non-temporary release gate directory: $resolvedGate"
        }
        if ([System.IO.Directory]::Exists($resolvedGate)) {
            [System.IO.Directory]::Delete($resolvedGate, $true)
        }
    }
}
finally {
    Pop-Location
}
