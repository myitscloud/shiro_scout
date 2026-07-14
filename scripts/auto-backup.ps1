<#
.SYNOPSIS
    Auto-backup script for ShiroScout project.
    Mounts network share and copies project to X:\shiroscout-ai-llm-agent via robocopy.
    Designed to run via Windows Task Scheduler with pwsh.exe.
.DESCRIPTION
    - Mounts \\HYPV-SRV01\H-512GBextSSD to drive X: (if not already mounted)
    - Uses robocopy /MIR for incremental mirror backup
    - Only changed files are copied (fast, incremental)
    - Excludes node_modules, target, .git, and other build artifacts
    - Logs to scripts/auto-backup.log
    - Silent operation (no popups)
.NOTES
    Version: 1.1 - Added automatic network share mounting
#>
param(
    [string]$Source = "C:\Users\wayne\agent-zero\Shiro-Scout\usr\projects\shiro_scout",
    [string]$Dest = "X:\shiroscout-ai-llm-agent",
    [string]$Share = "\\HYPV-SRV01\H-512GBextSSD"
)

$LogFile = Join-Path (Split-Path -Parent $PSCommandPath) "auto-backup.log"
$Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

function Write-Log($msg) {
    "[$Timestamp] $msg" | Out-File -FilePath $LogFile -Append -Encoding utf8
}

Write-Log "=== Starting backup ==="

# Check source exists
if (-not (Test-Path $Source)) {
    Write-Log "ERROR: Source path not found: $Source"
    exit 1
}

# Mount X: drive if not already available
if (-not (Test-Path "X:\")) {
    Write-Log "X: drive not found. Mounting $Share as X:..."
    $mount = net use X: $Share /persistent:yes 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Log "ERROR: Failed to mount $Share as X:"
        Write-Log "Details: $mount"
        exit 1
    }
    Write-Log "Mounted $Share as X: successfully."
}

# Create destination if needed
if (-not (Test-Path $Dest)) {
    New-Item -ItemType Directory -Path $Dest -Force | Out-Null
    Write-Log "Created destination directory: $Dest"
}

# Exclude build artifacts and large caches
$exclude = @(
    "node_modules",
    "target",
    ".git",
    "__pycache__",
    ".venv",
    "venv",
    ".npm"
)
$excludeArgs = $exclude | ForEach-Object { "/XD", $_ }

# Run robocopy mirror
$robocopyArgs = @(
    $Source,
    $Dest,
    "/MIR",
    "/R:2",
    "/W:5",
    "/NDL",
    "/NJH",
    "/NJS"
) + $excludeArgs

$result = & robocopy @robocopyArgs 2>&1
$exitCode = $LASTEXITCODE

if ($exitCode -ge 8) {
    Write-Log "ERROR: robocopy failed with exit code $exitCode"
    Write-Log "Details: $result"
    exit 1
} elseif ($exitCode -eq 0) {
    Write-Log "No changes to copy (already in sync)."
} else {
    Write-Log "SUCCESS: Files copied (exit code $exitCode)."
}

Write-Log "=== Backup complete ==="
