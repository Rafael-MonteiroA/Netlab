$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$stateFile = Join-Path $projectRoot 'work\server.json'
if (-not (Test-Path -LiteralPath $stateFile)) { Write-Host 'Nenhum servidor iniciado por este atalho.'; exit 0 }
$saved = Get-Content -LiteralPath $stateFile -Raw | ConvertFrom-Json
$running = Get-Process -Id $saved.pid -ErrorAction SilentlyContinue
if ($running -and $running.Path -eq $saved.binary -and $running.StartTime.ToUniversalTime().Ticks -eq ([datetime]$saved.started).ToUniversalTime().Ticks) {
    Stop-Process -Id $running.Id
    Write-Host 'NetLab encerrado. As salas em memoria foram removidas.'
} else { Write-Host 'O processo original ja foi encerrado.' }
Remove-Item -LiteralPath $stateFile
