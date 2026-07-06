#Requires -Version 5.1
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Target = "x86_64-unknown-uefi"
$Out = Join-Path $Root "dist\EFI\BOOT"

Write-Host "[uefi] adding target $Target"
rustup target add $Target

Write-Host "[uefi] building BOOTX64.EFI"
Push-Location $Root
try {
    cargo build -p rspfdisk-uefi --release --target $Target
} finally {
    Pop-Location
}

New-Item -ItemType Directory -Force -Path $Out | Out-Null
$Bin = Join-Path $Root "target\$Target\release\BOOTX64.EFI"
Copy-Item $Bin (Join-Path $Out "BOOTX64.EFI") -Force
Write-Host "[uefi] output: $Out\BOOTX64.EFI"
Get-Item (Join-Path $Out "BOOTX64.EFI") | Format-List Name, Length