# Imports a shared save (a folder or a .zip) into this install's saves.
# Story slots (SARC*) are added next to yours; a slot of the same name is replaced. Your player
# profile and settings are kept unless -All is given. Everything there before is backed up first
# to save-backups\before-import-<date-time>. Called by IMPORT-SAVE.bat.
param([Parameter(Mandatory)][string]$Source, [switch]$All)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$data = if ($env:VICTORY_DATA_DIR) { $env:VICTORY_DATA_DIR } else { Join-Path $root 'data' }
$target = Join-Path $data '7952987A\saves\0009000000000001\00000000-0000-0000-0000-00007952987a'

$Source = $Source.Trim().Trim('"')
if (-not (Test-Path -LiteralPath $Source)) { throw "Not found: $Source" }

# The game must be closed, or it may write over the import.
$running = Get-CimInstance Win32_Process -Filter "Name='victory.exe'" |
    Where-Object { $_.CommandLine -match '\brun\b' -and $_.ExecutablePath -like "$root*" }
if ($running) { throw 'The game is running. Close it first, then import again.' }

$work = $null
if ((Get-Item -LiteralPath $Source).PSIsContainer) {
    $from = $Source
} elseif ($Source -match '\.zip$') {
    $work = Join-Path $env:TEMP ('save-import-' + [guid]::NewGuid().ToString('N'))
    Expand-Archive -LiteralPath $Source -DestinationPath $work
    $from = $work
} else {
    throw 'Give a save folder or a .zip file.'
}

try {
    $pattern = if ($All) { '^(SARC\d+|Player|SettingsProfileSettings)~[0-9a-f]+$' } else { '^SARC\d+~[0-9a-f]+$' }
    $slots = @(Get-ChildItem -LiteralPath $from -Recurse -Directory | Where-Object { $_.Name -match $pattern })
    if (-not $slots) { throw "No save slots (folders named SARC0000~...) found in $Source" }

    if (Test-Path -LiteralPath (Join-Path $data '7952987A\saves')) {
        $backup = Join-Path $root ('save-backups\before-import-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
        New-Item -ItemType Directory -Force $backup | Out-Null
        Copy-Item -LiteralPath (Join-Path $data '7952987A\saves') -Destination $backup -Recurse
        "Backed up your saves to $backup"
    }
    New-Item -ItemType Directory -Force $target | Out-Null
    foreach ($slot in $slots) {
        $dest = Join-Path $target $slot.Name
        $replaced = Test-Path -LiteralPath $dest
        if ($replaced) { Remove-Item -LiteralPath $dest -Recurse -Force }
        Copy-Item -LiteralPath $slot.FullName -Destination $target -Recurse
        '{0} {1}' -f ($(if ($replaced) { 'Replaced' } else { 'Added   ' })), $slot.Name
    }
    "Done: $($slots.Count) imported into $target"
} finally {
    if ($work) { Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue }
}
