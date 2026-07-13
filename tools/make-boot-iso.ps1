#Requires -Version 5.1
<#
.SYNOPSIS
    Build dist/rspfdisk-boot.iso (delegates to WSL/Linux bash script).
#>
param(
    [string]$Kernel,
    [switch]$SkipBuild,
    [switch]$ValidateOnly
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$DistIso = Join-Path $Root "dist\rspfdisk-boot.iso"

function Test-BootBundle {
    $required = @(
        "boot\initramfs\init",
        "boot\grub\grub.cfg",
        "boot\rspfdisk-media",
        "tools\make-boot-iso.sh",
        "tools\make-boot-usb.sh",
        "tools\qemu-test.sh",
        "tools\lib\boot-common.sh"
    )
    foreach ($rel in $required) {
        $path = Join-Path $Root $rel
        if (-not (Test-Path $path)) {
            throw "missing boot file: $rel"
        }
    }
    Write-Host "[boot] bundle validation passed"
}

function Invoke-WslBuild {
    param([string[]]$Args)
    $bashScript = (Join-Path $Root "tools\make-boot-iso.sh") -replace '\\', '/'
    $wslPath = "/mnt/" + ($Root[0].ToString().ToLower()) + "/" + ($Root.Substring(3) -replace '\\', '/')
    $cmd = "cd '$wslPath' && bash tools/make-boot-iso.sh $($Args -join ' ')"
    Write-Host "[boot] WSL: $cmd"
    wsl bash -lc $cmd
}

Test-BootBundle

if ($ValidateOnly) {
    exit 0
}

$wsl = Get-Command wsl -ErrorAction SilentlyContinue
if (-not $wsl) {
    Write-Host "[boot] WSL not found. Boot ISO must be built on Linux:"
    Write-Host "  bash tools/make-boot-iso.sh"
    Write-Host "Bundle structure is valid. Place output at: $DistIso"
    exit 2
}

$argsList = @()
if ($Kernel) { $argsList += @("--kernel", $Kernel) }
if ($SkipBuild) { $argsList += "--skip-build" }

Invoke-WslBuild -Args $argsList

if (Test-Path $DistIso) {
    Write-Host "[boot] ISO ready: $DistIso"
    exit 0
}

Write-Host "[boot] ISO build did not produce $DistIso (check WSL build logs)"
exit 1
