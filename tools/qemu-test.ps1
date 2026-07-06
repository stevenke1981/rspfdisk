#Requires -Version 5.1
<#
.SYNOPSIS
    QEMU smoke test for rspfdisk-boot.iso.
#>
param(
    [ValidateSet("bios", "uefi")]
    [string]$Mode = "bios",
    [switch]$ValidateOnly,
    [string]$Image,
    [int]$TimeoutSec = 30
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$DistIso = Join-Path $Root "dist\rspfdisk-boot.iso"

function Test-BootBundle {
    & (Join-Path $Root "tools\make-boot-iso.ps1") -ValidateOnly
}

Test-BootBundle

if ($ValidateOnly) {
    Write-Host "[qemu] validate-only passed"
    exit 0
}

$qemu = Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue
$wsl = Get-Command wsl -ErrorAction SilentlyContinue

if ($wsl) {
    $wslPath = "/mnt/" + ($Root[0].ToString().ToLower()) + "/" + ($Root.Substring(3) -replace '\\', '/')
    $args = @("tools/qemu-test.sh", "--$Mode")
    if ($Image) { $args += @("--image", ($Image -replace '\\', '/')) }
    $envPrefix = "TIMEOUT_SEC=$TimeoutSec"
    if (-not (Test-Path $DistIso)) {
        Write-Host "[qemu] ISO not found at $DistIso"
        Write-Host "[qemu] Run: bash tools/make-boot-iso.sh (or WSL via make-boot-iso.ps1)"
        exit 2
    }
    $cmd = "cd '$wslPath' && $envPrefix bash $($args -join ' ')"
    Write-Host "[qemu] WSL: $cmd"
    wsl bash -lc $cmd
    exit $LASTEXITCODE
}

if ($qemu) {
    if (-not (Test-Path $DistIso)) {
        Write-Host "[qemu] ISO not found: $DistIso"
        exit 2
    }
    $testImg = if ($Image) { $Image } else { Join-Path $Root "tests\images\qemu-test.img" }
    if (-not (Test-Path $testImg)) {
        New-Item -ItemType Directory -Force -Path (Split-Path $testImg) | Out-Null
        $fs = [System.IO.File]::Create($testImg)
        $fs.SetLength(64MB)
        $fs.Close()
    }
    $qemuArgs = @(
        "-m", "1024",
        "-cdrom", $DistIso,
        "-hda", $testImg,
        "-display", "none",
        "-serial", "stdio",
        "-no-reboot"
    )
    if ($Mode -eq "uefi") {
        $ovmf = $env:OVMF_FD
        if (-not $ovmf) { throw "set OVMF_FD for UEFI test" }
        $qemuArgs += @("-bios", $ovmf)
    }
    Write-Host "[qemu] $($qemuArgs -join ' ')"
    & qemu-system-x86_64 @qemuArgs
    exit $LASTEXITCODE
}

Write-Host "[qemu] Neither QEMU nor WSL available. Ran bundle validation only."
Write-Host "[qemu] Install QEMU or WSL to run full smoke test."
exit 2