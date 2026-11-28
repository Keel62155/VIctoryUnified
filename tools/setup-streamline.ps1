# DLSS Frame Generation (NVIDIA Streamline) for this install, on first use: NVIDIA's Streamline SDK
# 2.14.16 release is downloaded from github.com/NVIDIA-RTX/Streamline (276 MB, once), its SHA-256
# checked, and the libraries frame generation needs are kept in streamline\ beside bin\ (about
# 70 MB); each must carry NVIDIA's valid signature. NVIDIA's files are not part of VIctoryUnified's
# release: their licence is NVIDIA's (streamline\license.txt).
# Exit codes: 0 ready, 1 failed (the game then starts without frame generation).
param([switch]$Quiet)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$dir = Join-Path $root 'streamline'
$version = '2.14.16'
$url = "https://github.com/NVIDIA-RTX/Streamline/releases/download/v$version/streamline-sdk-v$version.zip"
$sha256 = '32041e51db0cab709a271b72a352281b690c2abbd669a827809401a5dd23956b'
$needed = 'sl.interposer.dll', 'sl.common.dll', 'sl.dlss.dll', 'sl.dlss_g.dll', 'sl.reflex.dll', 'sl.pcl.dll', 'nvngx_dlss.dll', 'nvngx_dlssg.dll'

function Test-Signed([string]$path) {
    $s = Get-AuthenticodeSignature -LiteralPath $path
    return $s.Status -eq 'Valid' -and $s.SignerCertificate.Subject -match 'O=NVIDIA Corporation'
}

$have = $needed | Where-Object { -not (Test-Path (Join-Path $dir $_)) }
if (-not $have) {
    $bad = $needed | Where-Object { -not (Test-Signed (Join-Path $dir $_)) }
    if (-not $bad) { if (-not $Quiet) { "Streamline $version is ready in $dir" }; exit 0 }
    "Not signed by NVIDIA: $($bad -join ', '); downloading again."
}

try {
    New-Item -ItemType Directory -Force $dir | Out-Null
    $zip = Join-Path $env:TEMP "streamline-sdk-v$version.zip"
    $ok = (Test-Path $zip) -and ((Get-FileHash $zip -Algorithm SHA256).Hash -eq $sha256)
    if (-not $ok) {
        "Downloading NVIDIA Streamline SDK $version (276 MB, once) from GitHub..."
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        $ProgressPreference = 'SilentlyContinue'
        Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing
        $hash = (Get-FileHash $zip -Algorithm SHA256).Hash
        if ($hash -ne $sha256) { Remove-Item $zip -Force; throw "the download's SHA-256 is $hash, not NVIDIA's release ($sha256)" }
    }
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = [IO.Compression.ZipFile]::OpenRead($zip)
    try {
        foreach ($name in $needed + 'license.txt') {
            $entry = $archive.Entries | Where-Object { $_.FullName -eq "bin/x64/$name" -or ($name -eq 'license.txt' -and $_.FullName -eq 'license.txt') } | Select-Object -First 1
            if (-not $entry) { throw "$name is not in the SDK" }
            [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, (Join-Path $dir $name), $true)
        }
    } finally { $archive.Dispose() }
    $bad = $needed | Where-Object { -not (Test-Signed (Join-Path $dir $_)) }
    if ($bad) {
        $bad | ForEach-Object { Remove-Item (Join-Path $dir $_) -Force }
        throw "not signed by NVIDIA: $($bad -join ', ')"
    }
    Remove-Item $zip -Force -ErrorAction SilentlyContinue
    "Streamline $version is ready in $dir (each library signed by NVIDIA)"
    exit 0
} catch {
    "Streamline could not be set up: $($_.Exception.Message)"
    exit 1
}
