$ErrorActionPreference = "Stop"

$ProjectRoot = Split-Path -Parent $PSScriptRoot

$KernelElf      = Join-Path $ProjectRoot "target\x86_64-unknown-none\debug\mentacore-kernel"
$BootloaderEfi  = Join-Path $ProjectRoot "target\x86_64-unknown-uefi\debug\mentacore-bootloader.efi"

$EspRoot        = Join-Path $ProjectRoot "target\esp"
$BootEfi        = Join-Path $EspRoot "EFI\BOOT\BOOTX64.EFI"
$EspKernel      = Join-Path $EspRoot "kernel.elf"

$VarsTemplate   = "C:\Program Files\qemu\share\edk2-i386-vars.fd"
$VarsFile       = Join-Path $ProjectRoot "target\edk2-x86_64-vars.fd"

$Qemu           = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$FirmwareCode   = "C:\Program Files\qemu\share\edk2-x86_64-code.fd"

Write-Host ""
Write-Host "========================================"
Write-Host "        MENTACORE DEVELOPMENT RUNNER"
Write-Host "========================================"
Write-Host ""

Set-Location $ProjectRoot

Write-Host "[1/5] Building kernel..." -ForegroundColor Cyan

cargo build -p mentacore-kernel --target x86_64-unknown-none

if ($LASTEXITCODE -ne 0) {
    throw "Kernel build failed with exit code: $LASTEXITCODE"
}

if (-not (Test-Path $KernelElf)) {
    throw "Kernel ELF was not produced: $KernelElf"
}

Write-Host "[OK] Kernel built." -ForegroundColor Green
Write-Host ""

Write-Host "[2/5] Building bootloader..." -ForegroundColor Cyan

cargo build -p mentacore-bootloader --target x86_64-unknown-uefi

if ($LASTEXITCODE -ne 0) {
    throw "Bootloader build failed with exit code: $LASTEXITCODE"
}

if (-not (Test-Path $BootloaderEfi)) {
    throw "Bootloader EFI was not produced: $BootloaderEfi"
}

Write-Host "[OK] Bootloader built." -ForegroundColor Green
Write-Host ""

Write-Host "[3/5] Preparing EFI boot directory..." -ForegroundColor Cyan

New-Item -ItemType Directory -Force (Split-Path $BootEfi) | Out-Null

Copy-Item `
    $BootloaderEfi `
    $BootEfi `
    -Force

Write-Host "[OK] BOOTX64.EFI updated." -ForegroundColor Green
Write-Host ""

Write-Host "[4/5] Copying kernel to ESP..." -ForegroundColor Cyan

Copy-Item `
    $KernelElf `
    $EspKernel `
    -Force

Write-Host "[OK] kernel.elf copied." -ForegroundColor Green
Write-Host ""

Write-Host "[5/5] Starting QEMU..." -ForegroundColor Cyan
Write-Host ""

if (-not (Test-Path $VarsFile)) {
    if (-not (Test-Path $VarsTemplate)) {
        throw "UEFI VARS template not found: $VarsTemplate"
    }

    Copy-Item `
        $VarsTemplate `
        $VarsFile `
        -Force
}

& $Qemu `
    "-machine" "q35" `
    "-m" "512M" `
    "-drive" "if=pflash,format=raw,readonly=on,file=$FirmwareCode" `
    "-drive" "if=pflash,format=raw,file=$VarsFile" `
    "-drive" "file=fat:rw:$EspRoot,format=raw" `
    "-boot" "order=c" `
    "-serial" "stdio"

$ExitCode = $LASTEXITCODE

Write-Host ""
Write-Host "QEMU exited with code: $ExitCode"

exit $ExitCode