param(
    [Parameter(Mandatory=$true)]
    [string]$Package,

    [ValidateSet('debug', 'release')]
    [string]$Profile = 'release'
)

$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot

$pkgDir = Join-Path $repoRoot $Package
if (-not (Test-Path -Path $pkgDir -PathType Container)) {
    throw "Package directory not found: $pkgDir"
}

$cargoTomlPath = Join-Path $pkgDir 'Cargo.toml'
$cargoToml = Get-Content -Raw -Path $cargoTomlPath

$nameMatch = [Regex]::Match($cargoToml, '(?m)^\s*name\s*=\s*"([^"]+)"\s*$')
if (-not $nameMatch.Success) {
    throw "Could not find package name in $cargoTomlPath"
}
$crateName = $nameMatch.Groups[1].Value

$targetMatch = [Regex]::Match($cargoToml, '(?m)^\s*target\s*=\s*"([^"]+)"\s*$')
if (-not $targetMatch.Success) {
    throw "Could not find build.target in $cargoTomlPath"
}
$targetTriple = $targetMatch.Groups[1].Value

Push-Location $pkgDir
try {
    if ($Profile -eq 'release') {
        & cargo build --release
    } else {
        & cargo build
    }
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed"
    }
} finally {
    Pop-Location
}

$profileDir = if ($Profile -eq 'release') { 'release' } else { 'debug' }
$elfPath = Join-Path $pkgDir ("target\{0}\{1}\{2}" -f $targetTriple, $profileDir, $crateName)
if (-not (Test-Path -Path $elfPath -PathType Leaf)) {
    throw "ELF not found: $elfPath"
}

$outBin = Join-Path $pkgDir 'target\firmware.bin'
$outUf2 = Join-Path $pkgDir 'target\firmware.uf2'

$objcopyCandidates = @('rust-objcopy', 'llvm-objcopy', 'arm-none-eabi-objcopy')
$objcopy = $null
foreach ($c in $objcopyCandidates) {
    if (Get-Command $c -ErrorAction SilentlyContinue) {
        $objcopy = $c
        break
    }
}
if (-not $objcopy) {
    $sysroot = & rustc --print sysroot
    if ($LASTEXITCODE -eq 0 -and $sysroot) {
        $sysroot = $sysroot.Trim()
        $llvmObjcopy = Get-ChildItem -Path (Join-Path $sysroot 'lib\rustlib') -Recurse -Filter 'llvm-objcopy.exe' -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($llvmObjcopy) {
            $objcopy = $llvmObjcopy.FullName
        }
    }
}
if (-not $objcopy) {
    throw "objcopy not found. Install one of: rust-objcopy (cargo-binutils), llvm-objcopy, arm-none-eabi-objcopy, or add llvm-objcopy.exe from rustup toolchain to PATH."
}

& $objcopy -O binary $elfPath $outBin

$memoryXPath = Join-Path $pkgDir 'memory.x'
if (-not (Test-Path -Path $memoryXPath -PathType Leaf)) {
    throw "memory.x not found: $memoryXPath"
}
$memoryX = Get-Content -Raw -Path $memoryXPath
$flashMatch = [Regex]::Match($memoryX, '(?m)^\s*FLASH_ORIGIN\s*=\s*(0x[0-9A-Fa-f]+|\d+)\s*;')
if (-not $flashMatch.Success) {
    throw "Could not extract FLASH_ORIGIN from $memoryXPath"
}
$flashOrigin = $flashMatch.Groups[1].Value

$bin2uf2 = Join-Path $repoRoot 'tools\bin2uf2.py'
if (Get-Command py -ErrorAction SilentlyContinue) {
    & py -3 $bin2uf2 --input $outBin --output $outUf2 --base $flashOrigin
} elseif (Get-Command python -ErrorAction SilentlyContinue) {
    & python $bin2uf2 --input $outBin --output $outUf2 --base $flashOrigin
} else {
    throw "Python (py or python) not found for BIN->UF2 conversion"
}

Write-Host ("Package: {0}" -f $crateName)
Write-Host ("Target:  {0}" -f $targetTriple)
Write-Host ("BIN:     {0}" -f $outBin)
Write-Host ("UF2:     {0}" -f $outUf2)
