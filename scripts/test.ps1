$ErrorActionPreference = "Stop"

$ProjectRoot = Split-Path -Parent $PSScriptRoot

$KernelElf      = Join-Path $ProjectRoot "target\x86_64-unknown-none\debug\mentacore-kernel"
$UserspaceElf   = Join-Path $ProjectRoot "userspace\target\x86_64-unknown-none\debug\mentacore-userspace"
$BootloaderEfi  = Join-Path $ProjectRoot "target\x86_64-unknown-uefi\debug\mentacore-bootloader.efi"

$EspRoot        = Join-Path $ProjectRoot "target\esp"
$BootEfi        = Join-Path $EspRoot "EFI\BOOT\BOOTX64.EFI"
$EspKernel      = Join-Path $EspRoot "kernel.elf"
$EspUserspace   = Join-Path $EspRoot "userspace.elf"

$VarsTemplate   = "C:\Program Files\qemu\share\edk2-i386-vars.fd"
$VarsFile       = Join-Path $ProjectRoot "target\edk2-x86_64-vars.fd"

$Qemu           = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$FirmwareCode   = "C:\Program Files\qemu\share\edk2-x86_64-code.fd"

$TestLog        = Join-Path $ProjectRoot "target\mentacore-test.log"
$QemuPidFile    = Join-Path $ProjectRoot "target\mentacore-qemu.pid"

Write-Host ""
Write-Host "========================================"
Write-Host "         MENTACORE TEST RUNNER"
Write-Host "========================================"
Write-Host ""

Set-Location $ProjectRoot

# ------------------------------------------------------------
# 1. Build kernel
# ------------------------------------------------------------

Write-Host "[1/8] Building kernel..." -ForegroundColor Cyan

Push-Location (Join-Path $ProjectRoot "kernel")

cargo build -p mentacore-kernel --features kernel-tests --target x86_64-unknown-none

$KernelBuildExitCode = $LASTEXITCODE

Pop-Location

if ($KernelBuildExitCode -ne 0) {
    Write-Host "[FAIL] Kernel build failed." -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $KernelElf)) {
    Write-Host "[FAIL] Kernel ELF was not produced." -ForegroundColor Red
    exit 1
}

Write-Host "[OK] Kernel built." -ForegroundColor Green
Write-Host ""

# ------------------------------------------------------------
# 2. Build userspace
# ------------------------------------------------------------

Write-Host "[2/8] Building userspace..."

Push-Location (Join-Path $ProjectRoot "userspace")

cargo build
$UserspaceBuildExitCode = $LASTEXITCODE

Pop-Location

if ($UserspaceBuildExitCode -ne 0) {
    Write-Host "ERROR: Userspace build failed."
    exit $UserspaceBuildExitCode
}

if (-not (Test-Path $UserspaceElf)) {
    Write-Host "ERROR: Userspace ELF not found:"
    Write-Host $UserspaceElf
    exit 1
}

# ------------------------------------------------------------
# 3. Build bootloader
# ------------------------------------------------------------

Write-Host "[3/8] Building bootloader..." -ForegroundColor Cyan

cargo build -p mentacore-bootloader --target x86_64-unknown-uefi

if ($LASTEXITCODE -ne 0) {
    Write-Host "[FAIL] Bootloader build failed." -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $BootloaderEfi)) {
    Write-Host "[FAIL] Bootloader EFI was not produced." -ForegroundColor Red
    exit 1
}

Write-Host "[OK] Bootloader built." -ForegroundColor Green
Write-Host ""

# ------------------------------------------------------------
# 4. Prepare EFI boot directory
# ------------------------------------------------------------

Write-Host "[4/8] Preparing EFI boot directory..." -ForegroundColor Cyan

New-Item `
    -ItemType Directory `
    -Force `
    (Split-Path $BootEfi) | Out-Null

Copy-Item `
    $BootloaderEfi `
    $BootEfi `
    -Force

Write-Host "[OK] BOOTX64.EFI updated." -ForegroundColor Green
Write-Host ""

# ------------------------------------------------------------
# 5. Copy kernel
# ------------------------------------------------------------

Write-Host "[5/8] Copying kernel to ESP..." -ForegroundColor Cyan

Copy-Item `
    $KernelElf `
    $EspKernel `
    -Force

Write-Host "[OK] kernel.elf copied." -ForegroundColor Green
Write-Host ""

# ------------------------------------------------------------
# 6. Copy userspace
# ------------------------------------------------------------

Write-Host "[6/8] Copying userspace to ESP..." -ForegroundColor Cyan

Copy-Item `
    $UserspaceElf `
    $EspUserspace `
    -Force

Write-Host "[OK] userspace.elf copied." -ForegroundColor Green
Write-Host ""

# ------------------------------------------------------------
# 7. Prepare test environment
# ------------------------------------------------------------

Write-Host "[7/8] Preparing QEMU test environment..." -ForegroundColor Cyan

if (-not (Test-Path $Qemu)) {
    Write-Host "[FAIL] QEMU not found: $Qemu" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $FirmwareCode)) {
    Write-Host "[FAIL] UEFI firmware not found: $FirmwareCode" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $VarsFile)) {
    if (-not (Test-Path $VarsTemplate)) {
        Write-Host "[FAIL] UEFI VARS template not found: $VarsTemplate" -ForegroundColor Red
        exit 1
    }

    Copy-Item `
        $VarsTemplate `
        $VarsFile `
        -Force
}

if (Test-Path $TestLog) {
    Remove-Item $TestLog -Force
}

if (Test-Path $QemuPidFile) {
    Remove-Item $QemuPidFile -Force
}

New-Item `
    -ItemType File `
    -Path $TestLog `
    -Force | Out-Null

Write-Host "[OK] QEMU test environment ready." -ForegroundColor Green
Write-Host ""

# ------------------------------------------------------------
# 8. Run QEMU test environment
# ------------------------------------------------------------

Write-Host "[8/8] Running kernel tests in QEMU..." -ForegroundColor Cyan
Write-Host ""

$QemuArguments = @(
    "-machine"
    "q35"
    "-m"
    "512M"

    "-drive"
    "if=pflash,format=raw,readonly=on,file=$FirmwareCode"

    "-drive"
    "if=pflash,format=raw,file=$VarsFile"

    "-drive"
    "file=fat:rw:$EspRoot,format=raw"

    "-boot"
    "order=c"

    "-serial"
    "file:$TestLog"

    "-pidfile"
    $QemuPidFile
)

$QemuJob = Start-Job -ScriptBlock {
    param (
        [string]$QemuPath,
        [string[]]$Arguments
    )

    & $QemuPath @Arguments

    exit $LASTEXITCODE

} -ArgumentList $Qemu, $QemuArguments

$TimeoutSeconds = 60
$ElapsedSeconds = 0

$TestPassed = $false
$TestFailed = $false

while ($ElapsedSeconds -lt $TimeoutSeconds) {

    Start-Sleep -Seconds 1
    $ElapsedSeconds++

    if (Test-Path $TestLog) {

        $LogText = Get-Content `
            -Path $TestLog `
            -Raw `
            -ErrorAction SilentlyContinue

        if ($LogText -match "ALL TESTS PASSED") {
            $TestPassed = $true
            break
        }

        if ($LogText -match "TESTS FAILED") {
            $TestFailed = $true
            break
        }
    }

    $JobState = (Get-Job -Id $QemuJob.Id).State

    if ($JobState -eq "Failed" -or $JobState -eq "Stopped") {
        break
    }
}

# ------------------------------------------------------------
# Stop QEMU
# ------------------------------------------------------------

if ($TestPassed -or $TestFailed) {

    if (Test-Path $QemuPidFile) {

        $QemuPid = Get-Content `
            -Path $QemuPidFile `
            -Raw `
            -ErrorAction SilentlyContinue

        $QemuPid = $QemuPid.Trim()

        if ($QemuPid -match '^\d+$') {

            Stop-Process `
                -Id ([int]$QemuPid) `
                -Force `
                -ErrorAction SilentlyContinue
        }
    }

    Stop-Job `
        -Id $QemuJob.Id `
        -ErrorAction SilentlyContinue
}
else {

    Stop-Job `
        -Id $QemuJob.Id `
        -ErrorAction SilentlyContinue

    if (Test-Path $QemuPidFile) {

        $QemuPid = Get-Content `
            -Path $QemuPidFile `
            -Raw `
            -ErrorAction SilentlyContinue

        $QemuPid = $QemuPid.Trim()

        if ($QemuPid -match '^\d+$') {

            Stop-Process `
                -Id ([int]$QemuPid) `
                -Force `
                -ErrorAction SilentlyContinue
        }
    }
}

# ------------------------------------------------------------
# Collect QEMU output
# ------------------------------------------------------------

$JobOutput = Receive-Job `
    -Id $QemuJob.Id `
    -ErrorAction SilentlyContinue

Remove-Job `
    -Id $QemuJob.Id `
    -Force `
    -ErrorAction SilentlyContinue

Start-Sleep -Milliseconds 500

# ------------------------------------------------------------
# Read final test log
# ------------------------------------------------------------

$FinalLog = ""

if (Test-Path $TestLog) {

    $FinalLog = Get-Content `
        -Path $TestLog `
        -Raw `
        -ErrorAction SilentlyContinue
}

# ------------------------------------------------------------
# Cleanup PID file
# ------------------------------------------------------------

if (Test-Path $QemuPidFile) {
    Remove-Item $QemuPidFile -Force -ErrorAction SilentlyContinue
}

# ------------------------------------------------------------
# Result
# ------------------------------------------------------------

Write-Host ""
Write-Host "========================================"
Write-Host "        MENTACORE TEST RESULT"
Write-Host "========================================"

if ($TestPassed) {

    Write-Host "PASS" -ForegroundColor Green
    Write-Host ""

    if ($FinalLog -match "RESULT:\s*(\d+)\/(\d+)\s+TESTS PASSED") {

        $Passed = $Matches[1]
        $Total  = $Matches[2]

        Write-Host "Kernel tests: $Passed/$Total"
    }

    Write-Host "Log: $TestLog"
    Write-Host ""

    exit 0
}

Write-Host "FAIL" -ForegroundColor Red
Write-Host ""

if ($TestFailed) {

    Write-Host "Reason: Kernel reported test failure." `
        -ForegroundColor Red
}
elseif ($ElapsedSeconds -ge $TimeoutSeconds) {

    Write-Host "Reason: Test timeout ($TimeoutSeconds seconds)." `
        -ForegroundColor Red
}
else {

    Write-Host "Reason: QEMU test process failed before tests completed." `
        -ForegroundColor Red
}

Write-Host ""

if ($JobOutput) {

    Write-Host "QEMU process output:"

    $JobOutput | ForEach-Object {
        Write-Host $_
    }

    Write-Host ""
}

if ($FinalLog) {

    Write-Host "Kernel test log:"
    Write-Host ""

    Write-Host $FinalLog
    Write-Host ""
}

Write-Host "Log: $TestLog"
Write-Host ""

exit 1