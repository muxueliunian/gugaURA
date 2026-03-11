Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$projectRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$bundleDir = Join-Path $projectRoot "..\\target\\release\\bundle\\nsis"
$targetPath = Join-Path $bundleDir "gugaURA_installer.exe"

if (-not (Test-Path -LiteralPath $bundleDir)) {
    throw "NSIS bundle directory not found: $bundleDir"
}

$matches = @(Get-ChildItem -LiteralPath $bundleDir -Filter "*_x64-setup.exe" -File)
if ($matches.Count -ne 1) {
    throw "Expected exactly one NSIS installer matching '*_x64-setup.exe' in '$bundleDir', found $($matches.Count)."
}

if (Test-Path -LiteralPath $targetPath) {
    Remove-Item -LiteralPath $targetPath -Force
}

Move-Item -LiteralPath $matches[0].FullName -Destination $targetPath -Force
Write-Host "Installer renamed to $targetPath"
