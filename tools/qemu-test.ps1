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
    [ValidateRange(1, 3600)]
    [int]$TimeoutSec = 30
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$DistIso = Join-Path $Root "dist\rspfdisk-boot.iso"
$SerialLog = Join-Path $Root "dist\qemu-test-serial.log"
$SerialMarker = "RSPFDISK_TUI_READY"

function Test-BootBundle {
    & (Join-Path $Root "tools\make-boot-iso.ps1") -ValidateOnly
    if ($LASTEXITCODE -ne 0) {
        throw "boot bundle validation failed with exit code $LASTEXITCODE"
    }
}

function Assert-TestImagePath {
    param([string]$Path)

    if ($Path -match '(?i)^(/dev/|/sys/|/proc/|/run/)' -or $Path -match '(?i)PhysicalDrive') {
        throw "test image must be a regular image file, not a device: $Path"
    }
    if (Test-Path -LiteralPath $Path) {
        $item = Get-Item -LiteralPath $Path
        if ($item.PSIsContainer) {
            throw "test image must be a regular file: $Path"
        }
    }
}

function Assert-SerialMarker {
    if (-not (Test-Path -LiteralPath $SerialLog -PathType Leaf)) {
        throw "QEMU produced no serial log: $SerialLog"
    }

    $serial = [System.IO.File]::ReadAllText($SerialLog)
    Write-Host "[qemu] serial output: $SerialLog"
    if ($serial.Length -gt 0) {
        Write-Host $serial.TrimEnd()
    }
    if ($serial.IndexOf($SerialMarker, [System.StringComparison]::Ordinal) -lt 0) {
        throw "serial marker $SerialMarker was not observed"
    }
    Write-Host "[qemu] serial marker $SerialMarker observed"
}

function ConvertTo-ProcessArgument {
    param([string]$Value)

    if ($Value -notmatch '[\s"]') {
        return $Value
    }
    return '"' + $Value.Replace('"', '\"') + '"'
}

function ConvertTo-WslPath {
    param([string]$Path)

    if ($Path -match '^(?<Drive>[A-Za-z]):[\\/](?<Rest>.*)$') {
        $drive = $Matches.Drive.ToLowerInvariant()
        $rest = $Matches.Rest -replace '\\', '/'
        return "/mnt/$drive/$rest"
    }
    return $Path -replace '\\', '/'
}

function ConvertTo-BashSingleQuotedArgument {
    param([string]$Value)

    $singleQuote = [string][char]39
    $replacement = $singleQuote + [char]34 + $singleQuote + [char]34 + $singleQuote
    $escaped = $Value.Replace($singleQuote, $replacement)
    return $singleQuote + $escaped + $singleQuote
}

function Invoke-NativeQemu {
    param([string]$QemuPath, [string]$TestImage)

    Assert-TestImagePath -Path $TestImage
    if (-not (Test-Path -LiteralPath $DistIso -PathType Leaf)) {
        throw "ISO not found: $DistIso"
    }

    $imageParent = Split-Path -Parent $TestImage
    if (-not (Test-Path -LiteralPath $TestImage)) {
        New-Item -ItemType Directory -Force -Path $imageParent | Out-Null
        $fs = [System.IO.File]::Create($TestImage)
        try {
            $fs.SetLength(64MB)
        } finally {
            $fs.Close()
        }
    }

    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $SerialLog) | Out-Null
    if (Test-Path -LiteralPath $SerialLog) {
        Remove-Item -LiteralPath $SerialLog -Force
    }

    $qemuArgs = @(
        "-m", "1024",
        "-cdrom", $DistIso,
        "-hda", $TestImage,
        "-display", "none",
        "-serial", "file:$SerialLog",
        "-no-reboot"
    )
    if ($Mode -eq "uefi") {
        $ovmf = $env:OVMF_FD
        if (-not $ovmf) { throw "set OVMF_FD for UEFI test" }
        if (-not (Test-Path -LiteralPath $ovmf -PathType Leaf)) {
            throw "OVMF firmware not found: $ovmf"
        }
        $qemuArgs += @("-bios", $ovmf)
    }

    $argumentLine = ($qemuArgs | ForEach-Object { ConvertTo-ProcessArgument -Value $_ }) -join " "
    Write-Host "[qemu] $QemuPath $argumentLine"
    $process = Start-Process -FilePath $QemuPath -ArgumentList $argumentLine -WorkingDirectory $Root -PassThru
    $completed = $process.WaitForExit($TimeoutSec * 1000)
    $timedOut = -not $completed
    if ($timedOut) {
        try {
            $process.Kill()
        } catch {
            if (-not $process.HasExited) { throw }
        }
        $process.WaitForExit()
    }
    $qemuExit = $process.ExitCode

    Assert-SerialMarker
    if (-not $timedOut -and $qemuExit -ne 0) {
        throw "QEMU exited with $qemuExit after serial marker $SerialMarker"
    }
    if ($timedOut) {
        Write-Host "[qemu] timed out after $TimeoutSec seconds after serial marker $SerialMarker (accepted)"
    }
}

Test-BootBundle

if ($ValidateOnly) {
    Write-Host "[qemu] validate-only passed"
    exit 0
}

$qemu = Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue
$wsl = Get-Command wsl -ErrorAction SilentlyContinue

if ($wsl) {
    $testImgForWsl = if ($Image) { $Image } else { Join-Path $Root "tests\images\qemu-test.img" }
    Assert-TestImagePath -Path $testImgForWsl
    $wslPath = "/mnt/" + ($Root[0].ToString().ToLower()) + "/" + ($Root.Substring(3) -replace '\\', '/')
    $bashArgs = @("tools/qemu-test.sh", "--$Mode")
    if ($Image) { $bashArgs += @("--image", (ConvertTo-WslPath -Path $Image)) }
    $envPrefix = "TIMEOUT_SEC=$TimeoutSec"
    if (-not (Test-Path $DistIso)) {
        Write-Host "[qemu] ISO not found at $DistIso"
        Write-Host "[qemu] Run: bash tools/make-boot-iso.sh (or WSL via make-boot-iso.ps1)"
        exit 2
    }
    $quotedBashArgs = $bashArgs | ForEach-Object { ConvertTo-BashSingleQuotedArgument -Value $_ }
    $cmd = "cd $(ConvertTo-BashSingleQuotedArgument -Value $wslPath) && $envPrefix bash $($quotedBashArgs -join ' ')"
    Write-Host "[qemu] WSL: $cmd"
    wsl bash -lc $cmd
    $wslExit = $LASTEXITCODE
    if ($wslExit -ne 0) {
        throw "WSL QEMU smoke failed with exit code $wslExit"
    }
    Assert-SerialMarker
    Write-Host "[qemu] QEMU smoke test finished ($Mode)"
    exit 0
}

if ($qemu) {
    $testImg = if ($Image) { $Image } else { Join-Path $Root "tests\images\qemu-test.img" }
    Invoke-NativeQemu -QemuPath $qemu.Source -TestImage $testImg
    Write-Host "[qemu] QEMU smoke test finished ($Mode)"
    exit 0
}

Write-Host "[qemu] Neither QEMU nor WSL available. Ran bundle validation only."
Write-Host "[qemu] Install QEMU or WSL to run full smoke test."
exit 2
