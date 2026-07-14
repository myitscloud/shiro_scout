# ==============================================================================
# Diagnose-SandboxMount.ps1
# ShiroScout sandbox workspace-mount forensics (PowerShell 7)
#
# Answers, in one run:
#   1. How many aegis-sandbox containers exist (running OR stopped)?
#   2. Which of them actually has the /workspace bind, and what is its Source?
#   3. What does each RUNNING container really see inside /workspace?
#   4. What workspace_path string did the app persist, exactly?
#   5. Verdict: which failure mode matches (stale container / bad bind path /
#      empty setting / file-sharing layer).
#
# Read-only: this script changes nothing. Run:  pwsh .\Diagnose-SandboxMount.ps1
# ==============================================================================

$ErrorActionPreference = 'Continue'
$Verdicts = [System.Collections.Generic.List[string]]::new()

function Write-Section($t) { Write-Host "`n=== $t ===" -ForegroundColor Cyan }

# ------------------------------------------------------------------------------
# 0. Docker daemon sanity
# ------------------------------------------------------------------------------
Write-Section "0. Docker daemon"
$null = docker version --format '{{.Server.Version}}' 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "Docker daemon not reachable. Start Docker Desktop and re-run." -ForegroundColor Red
    exit 1
}
docker version --format 'Client {{.Client.Version}} / Server {{.Server.Version}}'

# ------------------------------------------------------------------------------
# 1. Enumerate ALL sandbox containers, any state
# ------------------------------------------------------------------------------
Write-Section "1. All containers matching 'aegis' (any state)"
$rows = docker ps -a --no-trunc --format '{{json .}}' |
    ForEach-Object { $_ | ConvertFrom-Json } |
    Where-Object { $_.Names -match 'aegis' -or $_.Image -match 'aegis-sandbox' }

if (-not $rows) {
    Write-Host "No aegis containers exist at all." -ForegroundColor Yellow
    $Verdicts.Add("NO-CONTAINER: nothing named/imaged 'aegis' exists. The app either never created one, or create_sandbox errored and the frontend swallowed it. Watch the app's console output during create.")
} else {
    $rows | Format-Table Names, ID, State, Status, CreatedAt -AutoSize
    if (@($rows).Count -gt 1) {
        $Verdicts.Add("MULTIPLE-CONTAINERS: $(@($rows).Count) aegis containers exist. Prime suspect: agents exec into an older one (F6 name-collision zombie). Compare their Created timestamps and binds below.")
    }
}

# ------------------------------------------------------------------------------
# 2. Deep inspect each: binds, mounts, user, network, rootfs
# ------------------------------------------------------------------------------
Write-Section "2. Inspect: HostConfig.Binds + Mounts per container"
$sawBindOnRunning = $false
$runningIds = @()

foreach ($r in $rows) {
    $j = docker inspect $r.ID | ConvertFrom-Json
    $c = $j[0]
    Write-Host "`n--- $($c.Name)  ($($c.Id.Substring(0,12)))" -ForegroundColor Green
    Write-Host ("  State:        {0}   Created: {1}" -f $c.State.Status, $c.Created)
    Write-Host ("  Image:        {0}" -f $c.Config.Image)
    Write-Host ("  User:         '{0}'   ReadonlyRootfs: {1}   Network: {2}" -f `
        $c.Config.User, $c.HostConfig.ReadonlyRootfs, $c.HostConfig.NetworkMode)
    Write-Host ("  Binds (raw):  {0}" -f ($(if ($c.HostConfig.Binds) { $c.HostConfig.Binds -join ' | ' } else { '(NONE)' })))

    $wsMount = $c.Mounts | Where-Object { $_.Destination -eq '/workspace' }
    if ($wsMount) {
        Write-Host ("  /workspace mount -> Type: {0}  RW: {1}" -f $wsMount.Type, $wsMount.RW)
        Write-Host ("                      Source: {0}" -f $wsMount.Source)
        if ($wsMount.Source -match '^\\\\\?\\') {
            $Verdicts.Add("BAD-PATH ($($c.Id.Substring(0,12))): bind Source carries a \\?\ UNC prefix - std::fs::canonicalize artifact. Docker cannot resolve it; container sees an empty dir. Fix: normalize path (dunce::simplified + forward slashes) before building the bind string.")
        }
        if ($c.State.Status -eq 'running') { $sawBindOnRunning = $true }
    } else {
        Write-Host "  /workspace mount:  *** ABSENT ***" -ForegroundColor Yellow
        if ($c.State.Status -eq 'running') {
            $Verdicts.Add("NO-BIND-ON-RUNNING ($($c.Id.Substring(0,12))): the RUNNING container has NO /workspace bind. Agents exec here and see the empty image-layer /workspace (WORKDIR decoy). Cause: created while workspace_path was empty (F7) and kept alive by the 409 name collision (F6). Fix: docker rm -f, then recreate from the app AFTER the workspace is set.")
        }
    }
    if ($c.State.Status -eq 'running') { $runningIds += $c.Id }
}

# ------------------------------------------------------------------------------
# 3. Ground truth from inside each RUNNING container
# ------------------------------------------------------------------------------
Write-Section "3. Inside each running container: id / ls -la /workspace / mount table"
foreach ($id in $runningIds) {
    $short = $id.Substring(0,12)
    Write-Host "`n--- exec into $short" -ForegroundColor Green
    docker exec $id sh -lc 'echo "[id]"; id; echo; echo "[ls -la /workspace]"; ls -la /workspace 2>&1; echo; echo "[mount entries for workspace]"; mount | grep -i workspace || echo "(no workspace entry in mount table => bind not attached)"'
    if ($LASTEXITCODE -ne 0) { Write-Host "exec failed (exit $LASTEXITCODE)" -ForegroundColor Yellow }
}
if ($runningIds.Count -eq 0 -and $rows) {
    $Verdicts.Add("NONE-RUNNING: containers exist but none is running. Whatever the agents exec against, it isn't here - check what container_id the app persisted (section 4/5).")
}

# ------------------------------------------------------------------------------
# 4. What the app persisted: settings.json (exact workspace_path string)
# ------------------------------------------------------------------------------
Write-Section "4. App settings: workspace_path as stored on disk"
$cfgDir = Join-Path $env:APPDATA 'com.shiroscout.app'
$settingsFile = Join-Path $cfgDir 'settings.json'
if (Test-Path $settingsFile) {
    $raw = Get-Content $settingsFile -Raw
    $s = $raw | ConvertFrom-Json
    Write-Host ("  workspace_path (exact): '{0}'" -f $s.workspace_path)
    Write-Host ("  mount_workspace:        {0}    sandbox_air_gapped: {1}" -f $s.mount_workspace, $s.sandbox_air_gapped)
    if ([string]::IsNullOrWhiteSpace($s.workspace_path)) {
        $Verdicts.Add("EMPTY-SETTING: workspace_path is empty in settings.json (F7). build_host_config adds no bind at all. Set the workspace in the app, then recreate the container.")
    } elseif (-not (Test-Path $s.workspace_path)) {
        $Verdicts.Add("PATH-NOT-FOUND: workspace_path '$($s.workspace_path)' does not exist on this host. Docker Desktop will surface an empty auto-created dir instead of your files.")
    } else {
        $n = (Get-ChildItem -LiteralPath $s.workspace_path -Force | Measure-Object).Count
        Write-Host ("  Host-side check:        path exists, {0} items inside" -f $n)
        if ($s.workspace_path -match '^\\\\\?\\') {
            $Verdicts.Add("BAD-PATH-IN-SETTINGS: stored path carries \\?\ prefix. Normalize before persisting (dunce).")
        }
    }
} else {
    Write-Host "  settings.json not found at $settingsFile" -ForegroundColor Yellow
    $Verdicts.Add("NO-SETTINGS: $settingsFile absent - app has never saved settings under this identifier, or a different identifier/profile is in use.")
}

# ------------------------------------------------------------------------------
# 5. Any persisted container_id in app state files?
# ------------------------------------------------------------------------------
Write-Section "5. Persisted state files mentioning a container id"
if (Test-Path $cfgDir) {
    Get-ChildItem $cfgDir -Filter *.json -Recurse | ForEach-Object {
        $hit = Select-String -Path $_.FullName -Pattern 'container' -SimpleMatch -List
        if ($hit) {
            Write-Host "  $($_.FullName):"
            Select-String -Path $_.FullName -Pattern '"[a-z_]*container[a-z_]*"\s*:\s*"[^"]*"' -AllMatches |
                ForEach-Object { $_.Matches.Value } | ForEach-Object { Write-Host "    $_" }
        }
    }
    Write-Host "  (If a persisted container_id here does not match a container with the bind in section 2, the app is exec'ing into a stale/dead id restored on startup.)"
} else {
    Write-Host "  Config dir absent: $cfgDir"
}

# ------------------------------------------------------------------------------
# 6. Verdict
# ------------------------------------------------------------------------------
Write-Section "6. VERDICT"
if ($Verdicts.Count -eq 0) {
    Write-Host @"
No structural fault found: a running container holds the /workspace bind and settings look sane.
If section 3 STILL showed an empty listing while section 4 counted items, the remaining suspect
is Docker Desktop's WSL2 file-sharing layer. Remedy:
    1) Quit Docker Desktop
    2) wsl --shutdown
    3) Start Docker Desktop, recreate the sandbox, re-run this script.
"@
} else {
    $i = 1
    foreach ($v in $Verdicts) { Write-Host ("  {0}. {1}" -f $i++, $v) -ForegroundColor Yellow; Write-Host "" }
}
Write-Host "Done. Paste the full output back for analysis." -ForegroundColor Cyan
