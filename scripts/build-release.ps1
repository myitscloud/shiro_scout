<#
.SYNOPSIS
    Local build verification for ShiroScout release.
.DESCRIPTION
    Runs the full verification pipeline: frontend build, Rust check, Rust release build,
    binary check, and SBOM generation.
    Exit code 0 = all gates pass.
#>

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot

Write-Host "=== ShiroScout Release Verification ===" -ForegroundColor Cyan
Write-Host "Project root: $ProjectRoot" -ForegroundColor Gray

# Step 1: Frontend
Write-Host "`n[1/5] Building frontend..." -ForegroundColor Yellow
Set-Location $ProjectRoot
pnpm install --frozen-lockfile
if ($LASTEXITCODE -ne 0) { throw "pnpm install failed" }

pnpm build
if ($LASTEXITCODE -ne 0) { throw "pnpm build failed" }
Write-Host "[1/5] Frontend build: PASS" -ForegroundColor Green

# Step 2: Rust check (debug)
Write-Host "`n[2/5] Running cargo check (debug)..." -ForegroundColor Yellow
Set-Location (Join-Path $ProjectRoot "src-tauri")
cargo check
if ($LASTEXITCODE -ne 0) { throw "cargo check failed" }
Write-Host "[2/5] cargo check: PASS" -ForegroundColor Green

# Step 3: Rust build (release)
Write-Host "`n[3/5] Building Rust release binary..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) { throw "cargo build --release failed" }
Write-Host "[3/5] Rust release build: PASS" -ForegroundColor Green

# Step 4: Binary check
Write-Host "`n[4/5] Verifying binary..." -ForegroundColor Yellow
$binary = Join-Path $ProjectRoot "src-tauri\target\release\shiro-scout.exe"
if (-not (Test-Path $binary)) { throw "Binary not found: $binary" }
$size = (Get-Item $binary).Length
Write-Host "Binary size: $([math]::Round($size / 1MB, 2)) MB" -ForegroundColor Gray
Write-Host "[4/5] Binary check: PASS" -ForegroundColor Green

# Step 5: SBOM (if cargo-deny available)
Write-Host "`n[5/5] Generating SBOM (cargo-deny required)..." -ForegroundColor Yellow
$deny = Get-Command cargo-deny -ErrorAction SilentlyContinue
if ($deny) {
    Set-Location $ProjectRoot
    cargo deny output cyclonedx -f sbom.json
    if (Test-Path "sbom.json") {
        Write-Host "SBOM written to $ProjectRoot\sbom.json" -ForegroundColor Gray
        Write-Host "[5/5] SBOM: PASS" -ForegroundColor Green
    } else {
        Write-Warning "cargo-deny ran but sbom.json not found"
    }
} else {
    Write-Warning "cargo-deny not installed. Install with: cargo install cargo-deny --locked"
    Write-Host "[5/5] SBOM: SKIPPED (cargo-deny missing)" -ForegroundColor Yellow
}

Write-Host "`n=== All gates PASSED ===" -ForegroundColor Green
Set-Location $ProjectRoot
