#Requires -Version 5.1
<#
.SYNOPSIS
    Release gate: fmt, clippy, test, checksums.
#>
param(
    [switch]$VerifyOnly,
    [switch]$WithUefi
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Push-Location $Root
try {
    Write-Host "[release] cargo fmt --check"
    cargo fmt --check

    Write-Host "[release] cargo clippy --workspace -- -D warnings"
    cargo clippy --workspace -- -D warnings

    Write-Host "[release] cargo test --workspace (fast)"
    cargo test --workspace
    Write-Host "[release] cargo test --workspace --include-ignored (slow image writes)"
    cargo test --workspace -- --include-ignored

    Write-Host "[release] cargo test -p rspfdisk-uefi"
    cargo test -p rspfdisk-uefi

    if ($WithUefi) {
        & (Join-Path $Root "tools\build-uefi.ps1")
    }

    if ($VerifyOnly) {
        Write-Host "[release] verify-only complete"
        exit 0
    }

    $Checksums = Join-Path $Root "dist\SHA256SUMS"
    New-Item -ItemType Directory -Force -Path (Join-Path $Root "dist") | Out-Null
    Write-Host "[release] writing checksums → $Checksums"

    $files = Get-ChildItem -Path (Join-Path $Root "dist") -Recurse -File |
        Where-Object { $_.Name -ne "SHA256SUMS" } |
        Sort-Object FullName

    $lines = foreach ($f in $files) {
        $hash = (Get-FileHash $f.FullName -Algorithm SHA256).Hash.ToLower()
        $rel = $f.FullName.Substring((Join-Path $Root "dist").Length + 1) -replace '\\', '/'
        "$hash  $rel"
    }
    $lines | Set-Content $Checksums -Encoding utf8
    Get-Content $Checksums
    Write-Host "[release] release gate passed"
} finally {
    Pop-Location
}