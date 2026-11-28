# Prepares dlss\ for START-DLSS.bat: DLSS upscaling through OptiScaler in place of the game's FSR.
# Nothing to do by hand:
#  - OptiScaler (GPL-3.0, https://github.com/optiscaler/OptiScaler) is downloaded once from its
#    official GitHub release and checked against its SHA-256 before anything of it is used.
#  - NVIDIA's DLSS library (nvngx_dlss.dll) is not downloaded: the newest copy found in a game
#    you have installed (Steam, Epic, Xbox app) is used, unmodified. None found: exit code 3 and
#    START-DLSS.bat falls back to FSR.
#  - The game runtime is copied into dlss\ (OptiScaler loads as dlss\dxgi.dll beside it), and
#    copied again whenever bin\victory.exe is newer.
# Frame generation is set up but off; the End key turns it on and off in game.
param([switch]$Quiet, [ValidateSet('dlss', 'fsr31')][string]$Upscaler = 'dlss', [switch]$FrameGen)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$dlss = Join-Path $root 'dlss'
$version = '0.9.5'
$asset = 'Optiscaler_0.9.5-final.20261116.7z'
$url = "https://github.com/optiscaler/OptiScaler/releases/download/v$version/$asset"
$sha256 = '77e3ed76bed04e1fb32bacb5a72799ff172f3da22ed5231c0cf3e84fe6761662'
function Say([string]$s) { if (-not $Quiet) { Write-Host $s } }
New-Item -ItemType Directory -Force $dlss | Out-Null

# --- OptiScaler, once ---
$marker = Join-Path $dlss "optiscaler-$version.txt"
if (-not (Test-Path $marker)) {
    $tmp = Join-Path $env:TEMP ("optiscaler-" + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory $tmp | Out-Null
    try {
        $archive = Join-Path $tmp $asset
        Say "Downloading OptiScaler $version (70 MB) from its GitHub release..."
        & curl.exe -sSfL -o $archive $url
        if ($LASTEXITCODE -ne 0) { throw "download failed ($url)" }
        $got = (Get-FileHash $archive -Algorithm SHA256).Hash.ToLower()
        if ($got -ne $sha256) { throw "OptiScaler archive hash mismatch: $got" }
        $x = Join-Path $tmp 'x'
        New-Item -ItemType Directory $x | Out-Null
        & tar.exe -xf $archive -C $x
        if ($LASTEXITCODE -ne 0) { throw 'could not unpack the OptiScaler archive' }
        Copy-Item (Join-Path $x 'OptiScaler.dll') (Join-Path $dlss 'dxgi.dll') -Force
        foreach ($f in 'amd_fidelityfx_dx12.dll', 'amd_fidelityfx_upscaler_dx12.dll', 'amd_fidelityfx_framegeneration_dx12.dll') {
            Copy-Item (Join-Path $x $f) $dlss -Force
        }
        New-Item -ItemType Directory -Force (Join-Path $dlss 'Licenses') | Out-Null
        Copy-Item (Join-Path $x 'Licenses\*') (Join-Path $dlss 'Licenses') -Force
        # Our settings over the release's INI.
        $want = @{
            'Upscalers|Dx12Upscaler' = 'dlss'
            'FrameGen|Enabled' = 'false'; 'FrameGen|FGInput' = 'upscaler'; 'FrameGen|FGOutput' = 'fsrfg'
            'Spoofing|Dxgi' = 'false'; 'Hotfix|CheckForUpdate' = 'false'; 'Menu|DisableSplash' = 'true'
            'Inputs|EnableFfxInputs' = 'true'; 'Inputs|UseFfxInputs' = 'true'
            'Log|LogToFile' = 'true'; 'Log|LogLevel' = '2'
        }
        $section = ''
        $ini = foreach ($line in Get-Content (Join-Path $x 'OptiScaler.ini')) {
            if ($line -match '^\[(.+)\]\s*$') { $section = $Matches[1] }
            if ($line -match '^\s*([A-Za-z0-9_]+)\s*=' -and $want.ContainsKey("$section|$($Matches[1])")) { "$($Matches[1])=$($want["$section|$($Matches[1])"])" } else { $line }
        }
        $ini | Set-Content (Join-Path $dlss 'OptiScaler.ini') -Encoding ascii
        "OptiScaler $version from $url (sha256 $sha256)" | Set-Content $marker
        Say 'OptiScaler is ready.'
    } finally {
        Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# --- nvngx_dlss.dll from a game you have installed (newest version) ---
$have = Join-Path $dlss 'nvngx_dlss.dll'
if (-not (Test-Path $have)) {
    $places = New-Object System.Collections.Generic.List[string]
    $steam = @("${env:ProgramFiles(x86)}\Steam", "$env:ProgramFiles\Steam") | Where-Object { Test-Path $_ }
    foreach ($s in $steam) {
        $places.Add("$s\steamapps\common")
        $vdf = "$s\steamapps\libraryfolders.vdf"
        if (Test-Path $vdf) {
            foreach ($m in [regex]::Matches((Get-Content $vdf -Raw), '"path"\s+"([^"]+)"')) {
                $places.Add(($m.Groups[1].Value -replace '\\\\', '\') + '\steamapps\common')
            }
        }
    }
    foreach ($p in "$env:ProgramFiles\Epic Games", 'C:\XboxGames', 'D:\XboxGames') { $places.Add($p) }
    $found = foreach ($p in ($places | Select-Object -Unique | Where-Object { Test-Path $_ })) {
        Get-ChildItem $p -Recurse -Filter 'nvngx_dlss.dll' -ErrorAction SilentlyContinue -Depth 6
    }
    $best = $found | Where-Object { $_.VersionInfo.FileVersion } |
        Sort-Object { [version](($_.VersionInfo.FileVersion -replace ',', '.' -replace '[^0-9.]', '').Trim('.')) } -Descending |
        Select-Object -First 1
    if (-not $best) {
        Say 'No DLSS library (nvngx_dlss.dll) found in your installed games: FSR is used instead.'
        exit 3
    }
    Copy-Item $best.FullName $have -Force
    Say "DLSS library: $($best.FullName) ($($best.VersionInfo.FileVersion))"
}

# --- this start's choices: upscaler, and whether frame generation is on at start ---
$iniPath = Join-Path $dlss 'OptiScaler.ini'
$section = ''
$set = @{ 'Upscalers|Dx12Upscaler' = $Upscaler; 'FrameGen|Enabled' = $(if ($FrameGen) { 'true' } else { 'false' }) }
$ini = foreach ($line in Get-Content $iniPath) {
    if ($line -match '^\[(.+)\]\s*$') { $section = $Matches[1] }
    if ($line -match '^\s*([A-Za-z0-9_]+)\s*=' -and $set.ContainsKey("$section|$($Matches[1])")) { "$($Matches[1])=$($set["$section|$($Matches[1])"])" } else { $line }
}
$ini | Set-Content $iniPath -Encoding ascii

# --- the runtime beside OptiScaler ---
$bin = Join-Path $root 'bin\victory.exe'
$there = Join-Path $dlss 'victory.exe'
if (-not (Test-Path $there) -or (Get-Item $bin).LastWriteTime -gt (Get-Item $there).LastWriteTime) {
    Copy-Item $bin $there -Force
}
exit 0
