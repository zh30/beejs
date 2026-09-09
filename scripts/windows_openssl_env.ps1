# Locate Chocolatey OpenSSL libs for openssl-sys on GitHub windows-latest.
# Recent OpenSSL 3.x packages put libcrypto.lib under lib\VC\x64\MD, not lib\.
$ErrorActionPreference = "Stop"

$dirs = @(
    "C:\Program Files\OpenSSL-Win64",
    "C:\Program Files\OpenSSL",
    "C:\Program Files (x86)\OpenSSL-Win64"
)
$found = $dirs | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $found) {
    throw "OpenSSL directory not found after choco install"
}

$lib = Get-ChildItem -Path $found -Recurse -Filter "libcrypto.lib" -ErrorAction SilentlyContinue |
    Select-Object -First 1
if (-not $lib) {
    throw "libcrypto.lib not found under $found"
}

$libDir = $lib.Directory.FullName
$inc = Join-Path $found "include"
if (-not (Test-Path $inc)) {
    throw "OpenSSL include dir missing: $inc"
}

Add-Content -Path $env:GITHUB_ENV -Value "OPENSSL_DIR=$found"
Add-Content -Path $env:GITHUB_ENV -Value "OPENSSL_LIB_DIR=$libDir"
Add-Content -Path $env:GITHUB_ENV -Value "OPENSSL_INCLUDE_DIR=$inc"
Write-Host "OpenSSL dir=$found"
Write-Host "OpenSSL lib=$libDir"
Write-Host "OpenSSL include=$inc"
Get-ChildItem $libDir | ForEach-Object { Write-Host ("  " + $_.Name) }
