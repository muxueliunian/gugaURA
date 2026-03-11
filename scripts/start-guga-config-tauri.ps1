param(
    [switch]$Watch,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$PassThruArgs
)

$ErrorActionPreference = 'Stop'

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location (Join-Path $repoRoot 'guga_ura_config_tauri')

$pnpmArgs = @('tauri', 'dev')
if (-not $Watch) {
    $pnpmArgs += '--no-watch'
}
if ($PassThruArgs) {
    $pnpmArgs += $PassThruArgs
}

& pnpm @pnpmArgs
