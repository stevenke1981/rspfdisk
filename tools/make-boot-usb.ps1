#Requires -Version 5.1
<#
.SYNOPSIS
    Build dist/rspfdisk-boot-usb.img (delegates to WSL/Linux bash script).
#>
param(
    [int]$SizeMb = 512,
    [switch]$ValidateOnly
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$DistUsb = Join-Path $Root "dist\rspfdisk-boot-usb.img"

function Test-BootBundle {
    $required = @(
        "boot\initramfs\init",
        "tools\make-boot-usb.sh"
    )
    foreach ($rel in $required) {
        if (-not (Test-Path (Join-Path $Root $rel))) {
            throw "missing boot file: $rel"
        }
    }
    Write-Host "[boot] USB bundle validation passed"
}

Test-BootBundle
if ($ValidateOnly) { exit 0 }

$wsl = Get-Command wsl -ErrorAction SilentlyContinue
if (-not $wsl) {
    Write-Host "[boot] WSL not found. Build on Linux:"
    Write-Host "  bash tools/make-boot-usb.sh --size-mb $SizeMb"
    exit 2
}

$wslPath = "/mnt/" + ($Root[0].ToString().ToLower()) + "/" + ($Root.Substring(3) -replace '\\', '/')
$cmd = "cd '$wslPath' && bash tools/make-boot-usb.sh --size-mb $SizeMb"
Write-Host "[boot] WSL: $cmd"
wsl bash -lc $cmd

if (Test-Path $DistUsb) {
    Write-Host "[boot] USB image ready: $DistUsb"
    exit 0
}
exit 1