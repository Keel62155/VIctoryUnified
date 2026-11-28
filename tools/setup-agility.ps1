# Puts Microsoft's DirectX 12 Agility SDK runtime (D3D12Core.dll, SDK 1.620.2) into D3D12\ beside
# victory.exe, which loads it from there at start (Windows' own runtime without it). Downloaded once
# from nuget.org (Microsoft.Direct3D.D3D12 1.620.2) and checked against the SHA-512 nuget.org
# publishes; nothing of it is shipped with this program. If anything fails, nothing is written and
# Windows' own Direct3D 12 runtime is used, as before.
# Only on Windows 10 unless VICTORY_AGILITY=1: there Windows' own runtime stops at shader model 6.5,
# which refuses every translated pipeline; Windows 11's has 6.6, and it is what was measured.
#   setup-agility.ps1 -Bin <folder of victory.exe> [-Quiet]
param([Parameter(Mandatory)][string]$Bin, [switch]$Quiet)
$ErrorActionPreference = 'Stop'
if ([Environment]::OSVersion.Version.Build -ge 22000 -and $env:VICTORY_AGILITY -ne '1') { exit 0 }
$version = '1.620.2'
$url = "https://api.nuget.org/v3-flatcontainer/microsoft.direct3d.d3d12/$version/microsoft.direct3d.d3d12.$version.nupkg"
$sha512 = '1oP/f1cmkGw/Q9KZ9s3UscrSL4MpzNJ7NJLQoUE9mQQQK41UNBm+w7NUXIa/SfalAOv0LZXJmaoJaeXWkwMnNQ=='
function Say([string]$s) { if (-not $Quiet) { Write-Host $s } }
$dir = Join-Path $Bin 'D3D12'
$core = Join-Path $dir 'D3D12Core.dll'
if (Test-Path $core) { exit 0 }
$tmp = Join-Path $env:TEMP ('agility-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory $tmp | Out-Null
try {
    $pkg = Join-Path $tmp 'agility.nupkg'
    Say "Downloading Microsoft's DirectX 12 Agility SDK $version (35 MB, once) from nuget.org..."
    & curl.exe -sSfL -o $pkg $url
    if ($LASTEXITCODE -ne 0) { throw "download failed" }
    $bytes = [System.IO.File]::ReadAllBytes($pkg)
    $got = [Convert]::ToBase64String([System.Security.Cryptography.SHA512]::Create().ComputeHash($bytes))
    if ($got -ne $sha512) { throw "hash mismatch" }
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zip = [System.IO.Compression.ZipFile]::OpenRead($pkg)
    try {
        New-Item -ItemType Directory -Force $dir | Out-Null
        foreach ($name in 'D3D12Core.dll', 'd3d12SDKLayers.dll') {
            $entry = $zip.Entries | Where-Object { $_.FullName -eq "build/native/bin/x64/$name" } | Select-Object -First 1
            if (-not $entry) { throw "$name missing from the package" }
            [System.IO.Compression.ZipFileExtensions]::ExtractToFile($entry, (Join-Path $dir $name), $true)
        }
        $license = $zip.Entries | Where-Object { $_.FullName -eq 'LICENSE.txt' } | Select-Object -First 1
        if ($license) { [System.IO.Compression.ZipFileExtensions]::ExtractToFile($license, (Join-Path $dir 'LICENSE-Microsoft-Agility-SDK.txt'), $true) }
    } finally { $zip.Dispose() }
    Say "DirectX 12 Agility SDK $version is ready in $dir."
    exit 0
} catch {
    Remove-Item $dir -Recurse -Force -ErrorAction SilentlyContinue
    Say "Agility SDK not set up ($($_.Exception.Message)): Windows' own Direct3D 12 is used."
    exit 1
} finally {
    Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
