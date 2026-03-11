Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$projectRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$bundleDir = Join-Path $projectRoot "..\\target\\release\\bundle\\nsis"

if (-not (Test-Path -LiteralPath $bundleDir)) {
    Write-Host "NSIS bundle directory does not exist yet: $bundleDir"
    exit 0
}

$patterns = @("*_x64-setup.exe", "gugaURA_installer.exe")
foreach ($pattern in $patterns) {
    Get-ChildItem -LiteralPath $bundleDir -Filter $pattern -File | Remove-Item -Force
}

Write-Host "Removed stale installer artifacts from $bundleDir"
