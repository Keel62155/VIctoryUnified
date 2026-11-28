# The game's own graphics settings (data\<user>\storage\settings\settings.xml), changed for one
# start and put back afterwards.
#   graphics-preset.ps1 -Data <data dir> -Preset <file>   copy each settings.xml aside, then apply
#   graphics-preset.ps1 -Data <data dir> -Restore          put the copies back
# A preset file holds "name=value" lines (# comments), names as in settings.xml's elements.
# The copies are settings.xml.before-max-performance beside each file. If a start ended without
# putting them back (the window closed, a crash), the next -Preset puts them back first, so the
# copy is always your own settings and never the preset's.
param([Parameter(Mandatory)][string]$Data, [string]$Preset, [switch]$Restore)
$ErrorActionPreference = 'Stop'
$suffix = '.before-max-performance'
$files = @(Get-ChildItem $Data -Recurse -Filter settings.xml -ErrorAction SilentlyContinue | Where-Object { $_.Directory.Name -eq 'settings' })

function Put-Back {
    foreach ($f in $files) {
        $copy = $f.FullName + $suffix
        if (Test-Path $copy) { Move-Item $copy $f.FullName -Force; "settings.xml put back: $($f.FullName)" }
    }
}

if ($Restore) { Put-Back; exit 0 }
if (-not $Preset) { throw 'Give -Preset <file> or -Restore.' }
Put-Back
if (-not $files.Count) { "No settings.xml yet (the game writes it on its first start): the preset applies from the next start."; exit 0 }
$pairs = [ordered]@{}
foreach ($line in Get-Content $Preset) {
    $l = $line.Trim()
    if (-not $l -or $l.StartsWith('#')) { continue }
    $k, $v = $l -split '=', 2
    $pairs[$k.Trim()] = $v.Trim()
}
foreach ($f in $files) {
    $text = [IO.File]::ReadAllText($f.FullName)
    $changed = 0
    foreach ($k in $pairs.Keys) {
        $pattern = '(<' + [regex]::Escape($k) + ' value=")[^"]*(")'
        if ($text -match $pattern) { $text = [regex]::Replace($text, $pattern, ('${1}' + $pairs[$k] + '${2}')); $changed++ }
    }
    Copy-Item $f.FullName ($f.FullName + $suffix) -Force
    [IO.File]::WriteAllText($f.FullName, $text)
    "max-performance graphics settings: $changed of $($pairs.Count) applied to $($f.FullName) (your own kept aside)"
}
