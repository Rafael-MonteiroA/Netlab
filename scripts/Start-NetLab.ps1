param([int]$Port = 8787, [switch]$LocalOnly, [switch]$NoBrowser)
$ErrorActionPreference = 'Stop'
if ($Port -lt 1 -or $Port -gt 65535) { throw 'Porta deve estar entre 1 e 65535.' }
$projectRoot = Split-Path -Parent $PSScriptRoot
$serverBinary = Join-Path $projectRoot 'bin\netlab-server.exe'
if (-not (Test-Path -LiteralPath $serverBinary)) {
    Push-Location $projectRoot
    try {
        & cargo build --release --locked -p netlab-server
        if ($LASTEXITCODE -ne 0) { throw 'Falha ao compilar. Instale Rust ou use o pacote Windows com executavel.' }
        $serverBinary = Join-Path $projectRoot 'target\release\netlab-server.exe'
    } finally { Pop-Location }
}
$runtimeDir = Join-Path $projectRoot 'work'
New-Item -ItemType Directory -Force -Path $runtimeDir | Out-Null
$stateFile = Join-Path $runtimeDir 'server.json'
if (Test-Path -LiteralPath $stateFile) {
    $saved = Get-Content -LiteralPath $stateFile -Raw | ConvertFrom-Json
    $running = Get-Process -Id $saved.pid -ErrorAction SilentlyContinue
    if ($running -and $running.Path -eq $saved.binary -and $running.StartTime.ToUniversalTime().Ticks -eq ([datetime]$saved.started).ToUniversalTime().Ticks) {
        if (-not $NoBrowser) { Start-Process "http://localhost:$($saved.port)" }
        Write-Host 'NetLab ja estava em execucao. Use Parar-NetLab.cmd para encerrar.'
        exit 0
    }
}
if ([System.Net.NetworkInformation.IPGlobalProperties]::GetIPGlobalProperties().GetActiveTcpListeners() | Where-Object { $_.Port -eq $Port }) {
    throw "A porta $Port ja esta ocupada. Encerre o servidor anterior ou use -Port com outra porta."
}
$bindAddress = if ($LocalOnly) { "127.0.0.1:$Port" } else { "0.0.0.0:$Port" }
$serverProcess = Start-Process -FilePath $serverBinary -ArgumentList @('--bind', $bindAddress) -WorkingDirectory $projectRoot -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $runtimeDir 'server.log') -RedirectStandardError (Join-Path $runtimeDir 'server-error.log')
$ready = $false
for ($attempt = 0; $attempt -lt 30; $attempt++) {
    Start-Sleep -Milliseconds 200
    if ($serverProcess.HasExited) { throw ('Servidor encerrou: ' + (Get-Content (Join-Path $runtimeDir 'server-error.log') -Raw)) }
    try { $health = Invoke-RestMethod "http://127.0.0.1:$Port/api/health" -TimeoutSec 1; if ($health.status -eq 'ok') { $ready = $true; break } } catch { }
}
if (-not $ready) { Stop-Process -Id $serverProcess.Id; throw 'O servidor nao respondeu. Confira os arquivos em work.' }
@{ pid = $serverProcess.Id; port = $Port; binary = $serverBinary; started = $serverProcess.StartTime.ToUniversalTime().ToString('o') } | ConvertTo-Json | Set-Content -LiteralPath $stateFile
if (-not $NoBrowser) { Start-Process "http://localhost:$Port" }
Write-Host 'NetLab iniciado. Use Parar-NetLab.cmd para encerrar.'
