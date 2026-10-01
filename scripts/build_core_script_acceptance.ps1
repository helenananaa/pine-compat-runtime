[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $OutputDirectory,
    [string] $Python = 'python'
)
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$root = [System.IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $root) { throw 'Choose a new directory; retained evidence must not be overwritten.' }
function Checked([string]$command, [string[]]$arguments) {
    Write-Host "+ $command $arguments"
    & $command @arguments
    if ($LASTEXITCODE -ne 0) { throw "failed ($LASTEXITCODE): $command" }
}
Push-Location $repo
try {
    $commit = (git rev-parse HEAD).Trim()
    if (git diff HEAD -- crates Cargo.toml Cargo.lock) { throw 'Commit the runtime changes before qualification.' }
    $names = @(git ls-files crates Cargo.toml Cargo.lock)
    $coreHashes = @{}
    foreach ($name in $names) { $coreHashes[$name] = (Get-FileHash -LiteralPath $name).Hash.ToLowerInvariant() }
    New-Item -ItemType Directory -Path $root | Out-Null
    $vs = & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    Import-Module (Join-Path $vs 'Common7\Tools\Microsoft.VisualStudio.DevShell.dll')
    Enter-VsDevShell -VsInstallPath $vs -SkipAutomaticLocation -DevCmdArguments '-arch=amd64'
    Checked powershell.exe @('-NoProfile','-ExecutionPolicy','Bypass','-File','scripts/verify.ps1','-Python',$Python)
    Checked cargo @('build','--locked','-p','pine-cli')
    Copy-Item -LiteralPath 'target/debug/pine-compat.exe' -Destination (Join-Path $root 'pine-compat.exe')
    New-Item -ItemType Directory -Path (Join-Path $root 'wasm'),(Join-Path $root 'wheels'),(Join-Path $root 'rust-probe') | Out-Null
    Checked cargo @('run','--locked','--quiet','-p','pine-wasm','--example','generate_node_bindings','--target','x86_64-pc-windows-msvc','--',(Join-Path $repo 'target/wasm32-unknown-unknown/debug/pine_wasm.wasm'),(Join-Path $root 'wasm'))
    Checked maturin @('build','--locked','--manifest-path','crates/pine-python/Cargo.toml','--out',(Join-Path $root 'wheels'))
    Checked $Python @('-m','venv','--system-site-packages',(Join-Path $root 'venv'))
    $py = Join-Path $root 'venv/Scripts/python.exe'
    $wheels = @(Get-ChildItem -LiteralPath (Join-Path $root 'wheels') -Filter '*.whl')
    if ($wheels.Count -ne 1) { throw 'Expected one fresh wheel.' }
    Checked $py @('-m','pip','install','--no-deps','--force-reinstall',$wheels[0].FullName)
    Checked $py @('-m','pytest','python/tests')
    $probeRoot = Join-Path $root 'rust-probe'
    Copy-Item -LiteralPath 'scripts/core_script_probe.rs' -Destination (Join-Path $probeRoot 'main.rs')
    Copy-Item -LiteralPath 'Cargo.lock' -Destination (Join-Path $probeRoot 'Cargo.lock')
    $dependencies = @('pine-runtime','pine-sema','pine-syntax') | ForEach-Object {
        $cratePath = (Join-Path $repo "crates/$_").Replace('\','/')
        "$_ = { path = `"$cratePath`" }"
    }
    $manifest = @('[package]','name="core-script-probe"','version="0.0.0"','edition="2024"','[workspace]','[[bin]]','name="core-script-probe"','path="main.rs"','[dependencies]') + $dependencies + @('serde_json="1"')
    $manifest | Set-Content -LiteralPath (Join-Path $probeRoot 'Cargo.toml') -Encoding UTF8
    Checked cargo @('build','--offline','--manifest-path',(Join-Path $probeRoot 'Cargo.toml'),'--target-dir',(Join-Path $repo 'target'))
    Copy-Item -LiteralPath 'target/debug/core-script-probe.exe' -Destination (Join-Path $root 'core-script-probe.exe')
    foreach ($name in $names) {
        if ((Get-FileHash -LiteralPath $name).Hash.ToLowerInvariant() -ne $coreHashes[$name]) { throw "Source changed: $name" }
    }
    if ((git rev-parse HEAD).Trim() -ne $commit) { throw 'HEAD changed during build.' }
    @{ sourceCommit=$commit; coreFiles=$coreHashes; profile='debug'; platform='Windows x86_64'; gateExitCode=0; installedWheelTests='passed'; probeSourceSha256=(Get-FileHash scripts/core_script_probe.rs).Hash.ToLowerInvariant() } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $root 'artifact-build-receipt.json') -Encoding UTF8
    Checked $Python @('scripts/requalify_core_scripts.py','--artifacts',$root)
}
finally { Pop-Location }
