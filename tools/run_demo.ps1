# Builds the showcase with one feature group, starts it on a given demo, saves a screenshot
# and prints the tail of the log.
#   pwsh tools/run_demo.ps1 -Features demo_ecs -Demo ChainWeak
param(
    [string]$Features = "",
    [Parameter(Mandatory)][string]$Demo,
    [double]$ShotAt = 4,
    [string]$TargetDir = "target",
    [switch]$AllFeatures
)
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root
$runDir = Join-Path $root "$TargetDir/run"
New-Item -ItemType Directory -Force $runDir | Out-Null
$tag = if ($AllFeatures) { "default" } else { ($Features -replace '[^\w]', '_') }
$exe = Join-Path $runDir "showcase_$tag.exe"
$lock = Join-Path $root "$TargetDir/.run_demo.lock"

# Several builds share one target dir, so build + copy must not interleave.
while ($true) {
    try { New-Item -ItemType Directory $lock -ErrorAction Stop | Out-Null; break }
    catch {
        # A killed run leaves its lock behind; don't wait on it forever.
        $stale = Get-Item $lock -ErrorAction SilentlyContinue
        if ($stale -and $stale.CreationTime -lt (Get-Date).AddMinutes(-25)) {
            Remove-Item $lock -Force -Recurse -ErrorAction SilentlyContinue
        }
        Start-Sleep -Seconds 2
    }
}
try {
    $cargoArgs = @("build", "--target-dir", $TargetDir)
    if (-not $AllFeatures) {
        $cargoArgs += "--no-default-features"
        if ($Features) { $cargoArgs += @("--features", $Features) }
    }
    & cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
    Copy-Item (Join-Path $root "$TargetDir/debug/bevy_showcase.exe") $exe -Force
}
finally { Remove-Item $lock -Force -Recurse -ErrorAction SilentlyContinue }

$shot = Join-Path $runDir "$Demo.png"
$log = Join-Path $runDir "$Demo.log"
Remove-Item $shot, $log -ErrorAction SilentlyContinue
$env:BEVY_ASSET_ROOT = $root
$env:SHOWCASE_DEMO = $Demo
$env:SHOWCASE_SCREENSHOT = $shot
$env:SHOWCASE_SHOT_AT = "$ShotAt"
$proc = Start-Process $exe -WorkingDirectory $root -PassThru -NoNewWindow `
    -RedirectStandardOutput $log -RedirectStandardError "$log.err"
if (-not $proc.WaitForExit([int](($ShotAt + 40) * 1000))) {
    $proc.Kill()
    Write-Output "TIMED OUT (killed)"
}
Write-Output "exit code: $($proc.ExitCode)"
Write-Output "screenshot: $shot (exists: $(Test-Path $shot))"
Write-Output "--- log tail ---"
Get-Content $log, "$log.err" -ErrorAction SilentlyContinue | Select-Object -Last 40
