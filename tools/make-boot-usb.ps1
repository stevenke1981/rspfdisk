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
        "boot\grub\grub.cfg",
        "boot\rspfdisk-media",
        "tools\make-boot-usb.sh",
        "tools\lib\boot-common.sh"
    )
    foreach ($rel in $required) {
        if (-not (Test-Path (Join-Path $Root $rel) -PathType Leaf)) {
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
$buildExit = $LASTEXITCODE
if ($buildExit -ne 0) {
    Write-Error "USB image build failed in WSL with exit code $buildExit. No image was reported as ready."
    exit $buildExit
}

if (Test-Path $DistUsb -PathType Leaf) {
    $image = Get-Item -LiteralPath $DistUsb
    if ($image.Length -le 0) {
        Write-Error "USB builder returned success but produced an empty image: $DistUsb"
        exit 1
    }
    Write-Host "[boot] USB image ready: $DistUsb"
    exit 0
}
Write-Error "USB builder returned success but did not produce: $DistUsb"
exit 1
