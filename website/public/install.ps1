# Beejs Windows installer. Downloads bee-vX-x86_64-pc-windows-msvc.zip from GitHub Releases.
param(
    [string]$Version = $env:BEEJS_VERSION,
    [string]$InstallDir = $(if ($env:BEEJS_INSTALL_DIR) { $env:BEEJS_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "beejs\bin" }),
    [string]$Repo = $(if ($env:BEEJS_REPO) { $env:BEEJS_REPO } else { "zh30/beejs" })
)

$ErrorActionPreference = "Stop"
$Target = "x86_64-pc-windows-msvc"

if (-not $Version) {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    $Version = $release.tag_name
}

if ($Version -notmatch '^v') {
    $Version = "v$Version"
}

$asset = "bee-$Version-$Target.zip"
$url = "https://github.com/$Repo/releases/download/$Version/$asset"
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) $asset

Write-Host "Downloading $url"
Invoke-WebRequest -Uri $url -OutFile $tmp

$extract = Join-Path ([System.IO.Path]::GetTempPath()) ("beejs-" + [guid]::NewGuid().ToString("n"))
New-Item -ItemType Directory -Path $extract | Out-Null
Expand-Archive -Path $tmp -DestinationPath $extract -Force

$src = Get-ChildItem -Path $extract -Recurse -Filter "bee.exe" | Select-Object -First 1
if (-not $src) {
    throw "bee.exe not found in archive"
}

New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
Copy-Item $src.FullName (Join-Path $InstallDir "bee.exe") -Force

$envPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($envPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$envPath", "User")
}

Write-Host "Beejs $Version installed to $InstallDir\bee.exe"
Write-Host "Open a new terminal and run: bee --version"
