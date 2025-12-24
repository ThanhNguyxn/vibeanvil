# VibeAnvil Uninstall Script (PowerShell)
# Usage: irm https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/uninstall.ps1 | iex

$ErrorActionPreference = 'Stop'

$InstallDir = "$HOME\.local\bin"

function Write-InfoMessage($Message) {
    Write-Host "→ " -NoNewline -ForegroundColor Blue
    Write-Host $Message
}

function Write-SuccessMessage($Message) {
    Write-Host "✓ " -NoNewline -ForegroundColor Green
    Write-Host $Message
}

function Write-WarningMessage($Message) {
    Write-Host "⚠ " -NoNewline -ForegroundColor Yellow
    Write-Host $Message
}

function Write-ErrorMessage($Message) {
    Write-Host "✗ " -NoNewline -ForegroundColor Red
    Write-Host $Message
    exit 1
}

Write-Host ""
Write-Host "╔═══════════════════════════════════════════╗" -ForegroundColor Red
Write-Host "║        🗑️  VibeAnvil Uninstaller           ║" -ForegroundColor Red
Write-Host "╚═══════════════════════════════════════════╝" -ForegroundColor Red
Write-Host ""

$InstallPath = Join-Path $InstallDir "vibeanvil.exe"
$BackupPath = Join-Path $InstallDir "vibeanvil.exe.bak"

# Remove main binary
if (Test-Path $InstallPath) {
    Remove-Item -Force $InstallPath
    Write-SuccessMessage "Removed: $InstallPath"
}
else {
    Write-WarningMessage "Binary not found at: $InstallPath"
}

# Remove backup if exists
if (Test-Path $BackupPath) {
    Remove-Item -Force $BackupPath
    Write-SuccessMessage "Removed backup: $BackupPath"
}

# Remove workspace (optional)
$WorkspaceDir = ".vibeanvil"
if (Test-Path $WorkspaceDir) {
    Write-Host ""
    $Confirm = Read-Host "Remove workspace directory (.vibeanvil)? [y/N]"
    if ($Confirm -eq "y" -or $Confirm -eq "Y") {
        Remove-Item -Recurse -Force $WorkspaceDir
        Write-SuccessMessage "Removed workspace: $WorkspaceDir"
    }
    else {
        Write-InfoMessage "Keeping workspace"
    }
}

# Remove global data
$GlobalDir = "$HOME\.vibeanvil"
if (Test-Path $GlobalDir) {
    Write-Host ""
    $Confirm = Read-Host "Remove global data (~/.vibeanvil)? [y/N]"
    if ($Confirm -eq "y" -or $Confirm -eq "Y") {
        Remove-Item -Recurse -Force $GlobalDir
        Write-SuccessMessage "Removed global data: $GlobalDir"
    }
    else {
        Write-InfoMessage "Keeping global data (BrainPack, etc.)"
    }
}

Write-Host ""
Write-Host "╔═══════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║     ✅ VibeAnvil uninstalled successfully  ║" -ForegroundColor Green
Write-Host "╚═══════════════════════════════════════════╝" -ForegroundColor Green
Write-Host ""
Write-Host "Thank you for using VibeAnvil! 👋"
Write-Host ""
